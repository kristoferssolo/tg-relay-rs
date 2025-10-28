use crate::{
    comments::global_comments,
    error::{Error, Result},
};
use capitalize::Capitalize;
use std::{
    ffi::OsStr,
    fmt::Display,
    path::{Path, PathBuf},
};
use teloxide::{prelude::*, types::InputFile};
use tokio::{fs::File, io::AsyncReadExt};
use tracing::{error, info, warn};

pub const VIDEO_EXTSTENSIONS: &[&str] = &["mp4", "webm", "mov", "mkv", "avi", "m4v", "3gp"];
pub const IMAGE_EXTSTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "bmp"];

/// Simple media kind enum shared by handlers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Video,
    Image,
    Unknown,
}

impl MediaKind {
    #[must_use]
    #[inline]
    pub const fn to_str(&self) -> &str {
        match self {
            Self::Video => "video",
            Self::Image => "image",
            Self::Unknown => "unknown",
        }
    }
}

/// Detect media kind first by extension, then by content/magic (sync).
pub fn detect_media_kind(path: &Path) -> MediaKind {
    if let Some(ext) = path.extension().and_then(OsStr::to_str) {
        let compare = |e: &&str| e.eq_ignore_ascii_case(ext);
        if VIDEO_EXTSTENSIONS.iter().any(compare) {
            return MediaKind::Video;
        }
        if IMAGE_EXTSTENSIONS.iter().any(compare) {
            return MediaKind::Image;
        }
    }

    // Fallback to MIME type detection
    if let Ok(Some(kind)) = infer::get_from_path(path) {
        let mime_type = kind.mime_type();
        return match mime_type.split('/').next() {
            Some("video") => MediaKind::Video,
            Some("image") => MediaKind::Image,
            _ => MediaKind::Unknown,
        };
    }

    MediaKind::Unknown
}

/// Async/non-blocking detection: check extension first, otherwise read a small
/// sample asynchronously and run `infer::get` on the buffer.
pub async fn detect_media_kind_async(path: &Path) -> MediaKind {
    if let Some(ext) = path.extension().and_then(OsStr::to_str) {
        let compare = |e: &&str| e.eq_ignore_ascii_case(ext);
        if VIDEO_EXTSTENSIONS.iter().any(compare) {
            return MediaKind::Video;
        }
        if IMAGE_EXTSTENSIONS.iter().any(compare) {
            return MediaKind::Image;
        }
    }

    // Read a small prefix (8 KiB) asynchronously and probe
    match File::open(path).await {
        Ok(mut file) => {
            let mut buffer = vec![0u8; 8192];
            if let Ok(n) = file.read(&mut buffer).await
                && n > 0
            {
                buffer.truncate(n);
                if let Some(k) = infer::get(&buffer) {
                    let mt = k.mime_type();
                    if mt.starts_with("video/") {
                        return MediaKind::Video;
                    }
                    if mt.starts_with("image/") {
                        return MediaKind::Image;
                    }
                }
            }
        }
        Err(e) => warn!(path = ?path.display(), "Failed to read file for media detection: {e}"),
    }

    MediaKind::Unknown
}

/// Given a path, send it to chat as photo or video depending on detected kind.
///
/// # Errors
///
/// Returns an `Error::UnknownMediaKind` if sending fails or the media kind is unknown.
pub async fn send_media_from_path(
    bot: &Bot,
    chat_id: ChatId,
    path: PathBuf,
    kind: MediaKind,
) -> Result<()> {
    let caption = global_comments().build_caption();
    let input = InputFile::file(path);

    macro_rules! send_msg {
        ($request_expr:expr) => {{
            let mut request = $request_expr;
            request = request.caption(caption);
            match request.await {
                Ok(message) => info!(message_id = message.id.to_string(), "{} sent", kind),
                Err(e) => {
                    error!("Failed to send {}: {e}", kind.to_str());
                    return Err(Error::Teloxide(e));
                }
            }
        }};
    }

    match kind {
        MediaKind::Video => send_msg!(bot.send_video(chat_id, input)),
        MediaKind::Image => send_msg!(bot.send_photo(chat_id, input)),
        MediaKind::Unknown => {
            bot.send_message(chat_id, "No supported media found")
                .await?;
            error!("No supported media found");
            return Err(Error::UnknownMediaKind);
        }
    }

    Ok(())
}

impl AsRef<str> for MediaKind {
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}

impl Display for MediaKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.capitalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_media_kind_by_extension() {
        assert_eq!(detect_media_kind(Path::new("video.mp4")), MediaKind::Video);
        assert_eq!(detect_media_kind(Path::new("image.jpg")), MediaKind::Image);
        assert_eq!(
            detect_media_kind(Path::new("unknown.txt")),
            MediaKind::Unknown
        );
    }

    #[test]
    fn media_kind_case_insensitive() {
        assert_eq!(detect_media_kind(Path::new("VIDEO.MP4")), MediaKind::Video);
        assert_eq!(detect_media_kind(Path::new("IMAGE.JPG")), MediaKind::Image);
    }
}
