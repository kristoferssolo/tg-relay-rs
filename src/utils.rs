use crate::{
    comments::global_comments,
    error::{Error, Result},
};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use teloxide::{
    Bot,
    payloads::{SendPhotoSetters, SendVideoSetters},
    prelude::Requester,
    types::{ChatId, InputFile},
};
use tokio::{fs::File, io::AsyncReadExt};

const TELEGRAM_CAPTION_LIMIT: usize = 1024;
static VIDEO_EXTS: &[&str] = &["mp4", "webm", "mov", "mkv", "avi"];
static IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp"];

/// Simple media kind enum shared by handlers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Video,
    Image,
    Unknown,
}

/// Detect media kind first by extension, then by content/magic (sync).
/// NOTE: `infer::get_from_path` is blocking — use `detect_media_kind_async` in
/// async contexts to avoid blocking the Tokio runtime.
pub fn detect_media_kind(path: &Path) -> MediaKind {
    if let Some(ext) = path.extension().and_then(OsStr::to_str) {
        if VIDEO_EXTS.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
            return MediaKind::Video;
        }
        if IMAGE_EXTS.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
            return MediaKind::Image;
        }
    }

    if let Ok(Some(kind)) = infer::get_from_path(path) {
        let mt = kind.mime_type();
        if mt.starts_with("video/") {
            return MediaKind::Video;
        }
        if mt.starts_with("image/") {
            return MediaKind::Image;
        }
    }

    MediaKind::Unknown
}

/// Async/non-blocking detection: check extension first, otherwise read a small
/// sample asynchronously and run `infer::get` on the buffer.
pub async fn detect_media_kind_async(path: &Path) -> MediaKind {
    if let Some(ext) = path.extension().and_then(OsStr::to_str) {
        if VIDEO_EXTS.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
            return MediaKind::Video;
        }
        if IMAGE_EXTS.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
            return MediaKind::Image;
        }
    }

    // Read a small prefix (8 KiB) asynchronously and probe
    if let Ok(mut f) = File::open(path).await {
        let mut buf = vec![0u8; 8192];
        match f.read(&mut buf).await {
            Ok(n) if n > 0 => {
                buf.truncate(n);
                if let Some(k) = infer::get(&buf) {
                    let mt = k.mime_type();
                    if mt.starts_with("video/") {
                        return MediaKind::Video;
                    }
                    if mt.starts_with("image/") {
                        return MediaKind::Image;
                    }
                }
            }
            _ => {}
        }
    }

    MediaKind::Unknown
}

/// Given a path, send it to chat as photo or video depending on detected kind.
///
/// # Errors
///
/// Returns an error if sending fails or the media kind is unknown.
pub async fn send_media_from_path(
    bot: &Bot,
    chat_id: ChatId,
    path: PathBuf,
    kind: Option<MediaKind>,
) -> Result<()> {
    let kind = kind.unwrap_or_else(|| detect_media_kind(&path));

    let caption_opt = global_comments().map(|c| {
        let mut caption = c.build_caption();
        if caption.chars().count() > TELEGRAM_CAPTION_LIMIT {
            caption = caption.chars().take(TELEGRAM_CAPTION_LIMIT - 1).collect();
            caption.push_str("...");
        }
        caption
    });

    match kind {
        MediaKind::Video => {
            let video = InputFile::file(path);
            let mut req = bot.send_video(chat_id, video);
            if let Some(c) = caption_opt {
                req = req.caption(c);
            }
            req.await.map_err(Error::from)?;
        }
        MediaKind::Image => {
            let photo = InputFile::file(path);
            let mut req = bot.send_photo(chat_id, photo);
            if let Some(c) = caption_opt {
                req = req.caption(c);
            }
            req.await.map_err(Error::from)?;
        }
        MediaKind::Unknown => {
            bot.send_message(chat_id, "No supported media found")
                .await
                .map_err(Error::from)?;
            return Err(Error::UnknownMediaKind);
        }
    }
    Ok(())
}
