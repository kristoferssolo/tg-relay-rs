use crate::error::{Error, Result};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use teloxide::{
    Bot,
    prelude::Requester,
    types::{ChatId, InputFile},
};
use tokio::{fs::File, io::AsyncReadExt};

/// Simple media kind enum shared by handlers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Video,
    Image,
    Unknown,
}

static VIDEO_EXTS: &[&str] = &["mp4", "webm", "mov", "mkv", "avi"];
static IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp"];

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
    match kind {
        MediaKind::Video => {
            let video = InputFile::file(path);
            bot.send_video(chat_id, video).await.map_err(Error::from)?;
        }
        MediaKind::Image => {
            let photo = InputFile::file(path);
            bot.send_photo(chat_id, photo).await.map_err(Error::from)?;
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
