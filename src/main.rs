mod telemetry;

use crate::telemetry::setup_logger;
use color_eyre::{
    Result,
    eyre::{Context, eyre},
};
use dotenv::dotenv;
use regex::Regex;
use std::{
    fs::{File, read_dir},
    io::Read,
    path::Path,
};
use teloxide::{
    Bot,
    prelude::Requester,
    respond,
    types::{ChatId, InputFile, Message},
};
use tempfile::tempdir;
use tokio::process::Command;
use tracing::error;

static VIDEO_EXTS: &[&str] = &["mp4", "webm"];
static IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MediaKind {
    Video,
    Image,
    Unknown,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenv().ok();
    color_eyre::install()?;
    setup_logger()?;

    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        if let Some(text) = msg.text() {
            if let Some(shortcode) = extract_instagram_shortcode(text) {
                let bot_cloned = bot.clone();
                let chat = msg.chat.id;

                tokio::spawn(async move {
                    if let Err(e) = fetch_and_send(&bot_cloned, chat, &shortcode).await {
                        error!("error fetching/sending: {:?}", e);
                        let _ = bot_cloned
                            .send_message(chat, "Failed to fetch Instagram media.")
                            .await;
                    }
                });
            }
        }
        respond(())
    })
    .await;

    Ok(())
}

fn extract_instagram_shortcode(text: &str) -> Option<String> {
    let re = Regex::new(
        r"https?://(?:www\.)?(?:instagram\.com|instagr\.am)/(?:p|reel|tv)/([A-Za-z0-9_-]+)",
    )
    .unwrap();
    re.captures(text)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
}

async fn fetch_and_send(bot: &Bot, chat_id: ChatId, shortcode: &str) -> Result<()> {
    let dir = tempdir().context("create tempdir")?;
    let dir_path = dir.path().to_path_buf();

    dbg!(&dir_path);
    let target = format!("-{}", shortcode);
    dbg!(&target);
    let status = Command::new("instaloader")
        .arg("--dirname-pattern")
        .arg(dir_path.to_string_lossy().as_ref())
        .arg("--no-metadata-json")
        .arg("--no-compress-json")
        .arg("--quiet")
        .arg("--")
        .arg(&target)
        .status()
        .await
        .context("runnning instaloader")?;

    if !status.success() {
        error!("instaloader exit: {:?}", status);
        return Err(eyre!("instaloader failed"));
    }
    let mut media_files = Vec::new();

    for entry in read_dir(&dir_path)? {
        let p = entry?.path();
        if p.is_file() {
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
            if matches!(ext, "jpg" | "jpeg" | "mp4" | "webm") {
                media_files.push(p);
            }
        }
    }

    if media_files.is_empty() {
        return Err(eyre!("no media found"));
    }

    dbg!(&media_files);

    if let Some(video_path) = media_files.iter().find(|p| is_video(p)) {
        let input = InputFile::file(video_path.clone());
        bot.send_video(chat_id, input).await?;
        return Ok(());
    }

    if let Some(image_path) = media_files.iter().find(|p| is_image(p)) {
        let input = InputFile::file(image_path.clone());
        bot.send_photo(chat_id, input).await?;
        return Ok(());
    }

    bot.send_message(chat_id, "No supported media found")
        .await?;

    Ok(())
}

fn ext_lower(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
}

fn kind_by_magic(path: &Path) -> Option<MediaKind> {
    let mut f = File::open(path).ok()?;
    let mut buf = [0u8; 8192];

    let n = f.read(&mut buf).ok()?;
    if n == 0 {
        return None;
    }

    if let Some(kind) = infer::get(&buf[..n]) {
        let mt = kind.mime_type();
        if mt.starts_with("video/") {
            return Some(MediaKind::Video);
        }
        if mt.starts_with("image/") {
            return Some(MediaKind::Image);
        }
    }
    None
}

fn detect_media_kind(path: &Path) -> MediaKind {
    if let Some(ext) = ext_lower(path) {
        if VIDEO_EXTS.iter().any(|e| e.eq_ignore_ascii_case(&ext)) {
            return MediaKind::Video;
        }
        if IMAGE_EXTS.iter().any(|e| e.eq_ignore_ascii_case(&ext)) {
            return MediaKind::Image;
        }
    }
    kind_by_magic(path).unwrap_or(MediaKind::Unknown)
}

fn is_video(path: &Path) -> bool {
    detect_media_kind(path) == MediaKind::Video
}

fn is_image(path: &Path) -> bool {
    detect_media_kind(path) == MediaKind::Image
}
