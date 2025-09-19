use crate::{
    download::{download_ytdlp, process_download_result},
    error::Result,
};
use regex::Regex;
use std::sync::OnceLock;
use teloxide::{Bot, types::ChatId};
use tracing::info;

use crate::handlers::SocialHandler;

static SHORTCODE_RE: OnceLock<Regex> = OnceLock::new();

fn shortcode_regex() -> &'static Regex {
    SHORTCODE_RE.get_or_init(|| {
        Regex::new(
            r"https?://(?:www\.)?(?:youtube\.com/shorts/[A-Za-z0-9_-]+(?:\?[^\s]*)?|youtu\.be/[A-Za-z0-9_-]+(?:\?[^\s]*)?)",
        )
        .expect("filed to compile regex")
    })
}

/// Handler for `YouTube Shorts` (and short youtu.be links)
#[derive(Clone, Default)]
pub struct YouTubeShortsHandler;

impl YouTubeShortsHandler {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl SocialHandler for YouTubeShortsHandler {
    fn name(&self) -> &'static str {
        "youtube"
    }

    fn try_extract(&self, text: &str) -> Option<String> {
        shortcode_regex().find(text).map(|m| m.as_str().to_owned())
    }

    async fn handle(&self, bot: &Bot, chat_id: ChatId, url: String) -> Result<()> {
        info!(handler = %self.name(), url = %url, "handling youtube code");
        let format = "bestvideo[ext=mp4]+bestaudio/best";
        let dr = download_ytdlp(&url, format).await?;
        process_download_result(bot, chat_id, dr).await
    }

    fn box_clone(&self) -> Box<dyn SocialHandler> {
        Box::new(self.clone())
    }
}
