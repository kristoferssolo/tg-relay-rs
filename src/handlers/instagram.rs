use crate::error::{Error, Result};
use crate::handlers::SocialHandler;
use crate::utils::{MediaKind, detect_media_kind_async, send_media_from_path};
use futures::{StreamExt, stream};
use regex::Regex;
use std::path::PathBuf;
use std::{process::Stdio, sync::OnceLock};
use teloxide::{Bot, types::ChatId};
use tempfile::tempdir;
use tokio::fs::read_dir;
use tokio::process::Command;
use tracing::{error, info};

static SHORTCODE_RE: OnceLock<Regex> = OnceLock::new();

fn shortcode_regex() -> &'static Regex {
    SHORTCODE_RE.get_or_init(|| {
        Regex::new(
            r"https?://(?:www\.)?(?:instagram\.com|instagr\.am)/(?:p|reel|tv)/([A-Za-z0-9_-]+)",
        )
        .expect("filed to compile regex")
    })
}

/// Handler for Instagram posts / reels / tv
#[derive(Clone, Default)]
pub struct InstagramHandler;

impl InstagramHandler {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl SocialHandler for InstagramHandler {
    fn name(&self) -> &'static str {
        "instagram"
    }

    fn try_extract(&self, text: &str) -> Option<String> {
        shortcode_regex()
            .captures(text)
            .and_then(|c| c.get(1).map(|m| m.as_str().to_owned()))
    }

    async fn handle(&self, bot: &Bot, chat_id: ChatId, shortcode: String) -> Result<()> {
        info!(handler = %self.name(), shortcode = %shortcode, "handling instagram code");
        let tmp = tempdir().map_err(Error::from)?;
        let cwd = tmp.path().to_path_buf();
        let target = format!("-{shortcode}");

        let status = Command::new("instaloader")
            .current_dir(&cwd)
            .args([
                "--dirname-pattern=.",
                "--no-metadata-json",
                "--no-compress-json",
                "--quiet",
                "--",
                &target,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map_err(Error::from)?;

        if !status.success() {
            return Err(Error::InstaloaderFaileled(status.to_string()));
        }

        let mut dir = read_dir(&cwd).await?;
        let mut paths = Vec::new();

        while let Some(entry) = dir.next_entry().await? {
            if entry.file_type().await?.is_file() {
                paths.push(entry.path());
            }
        }

        let concurrency = 8;
        let results = stream::iter(paths)
            .map(|path| async move {
                let kind = detect_media_kind_async(&path).await;
                match kind {
                    MediaKind::Unknown => None,
                    k => Some((path, k)),
                }
            })
            .buffer_unordered(concurrency)
            .collect::<Vec<Option<(PathBuf, MediaKind)>>>()
            .await;

        let mut media = results
            .into_iter()
            .flatten()
            .collect::<Vec<(PathBuf, MediaKind)>>();

        if media.is_empty() {
            error!("no media found in tmp dir after instaloader");
            return Err(Error::NoMediaFound);
        }

        // deterministic ordering
        media.sort_by_key(|(p, _)| p.clone());

        // prefer video over image
        if let Some((path, MediaKind::Video)) = media.iter().find(|(_, k)| *k == MediaKind::Video) {
            return send_media_from_path(bot, chat_id, path.clone(), Some(MediaKind::Video)).await;
        }

        if let Some((path, MediaKind::Image)) = media.iter().find(|(_, k)| *k == MediaKind::Image) {
            return send_media_from_path(bot, chat_id, path.clone(), Some(MediaKind::Image)).await;
        }

        error!("no supported media kind found after scanning");
        Err(Error::NoMediaFound)
    }

    fn box_clone(&self) -> Box<dyn SocialHandler> {
        Box::new(self.clone())
    }
}
