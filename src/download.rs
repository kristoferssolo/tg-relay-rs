use crate::{
    error::{Error, Result},
    utils::{MediaKind, detect_media_kind_async, send_media_from_path},
};
use futures::{StreamExt, stream};
use std::{path::PathBuf, process::Stdio};
use teloxide::{Bot, types::ChatId};
use tempfile::{TempDir, tempdir};
use tokio::{fs::read_dir, process::Command};

/// `TempDir` guard + downloaded files. Keep this value alive until you're
/// done sending files so the temporary directory is not deleted.
#[derive(Debug)]
pub struct DownloadResult {
    pub tempdir: TempDir,
    pub files: Vec<PathBuf>,
}

/// Run a command in a freshly created temporary directory and collect
/// regular files produced there.
///
/// `cmd` is the command name (e.g. "yt-dlp" or "instaloader").
/// `args` are the command arguments (owned Strings so callers can build dynamic args).
///
/// # Errors
///
/// - `Error::Io` for filesystem / spawn errors (propagated).
/// - `Error::Other` for non-zero exit code (with stderr).
/// - `Error::NoMediaFound` if no files were produced.
#[allow(clippy::similar_names)]
async fn run_command_in_tempdir(cmd: &str, args: &[&str]) -> Result<DownloadResult> {
    let tmp = tempdir().map_err(Error::from)?;
    let cwd = tmp.path().to_path_buf();

    let output = Command::new(cmd)
        .current_dir(&cwd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(Error::from)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(Error::Other(format!("{cmd} failed: {stderr}")));
    }

    // collect files produced in tempdir (async)
    let mut rd = read_dir(&cwd).await?;
    let mut files = Vec::new();
    while let Some(entry) = rd.next_entry().await? {
        if entry.file_type().await?.is_file() {
            files.push(entry.path());
        }
    }

    if files.is_empty() {
        return Err(Error::NoMediaFound);
    }

    Ok(DownloadResult {
        tempdir: tmp,
        files,
    })
}

/// Download an Instagram shortcode using instaloader (wrapper).
///
/// # Errors
///
/// - Propagates `run_command_in_tempdir` errors.
pub async fn download_instaloader(shortcode: &str) -> Result<DownloadResult> {
    let args = [
        "--no-metadata-json",
        "--no-compress-json",
        "--quiet",
        "--",
        &format!("-{shortcode}"),
    ];
    run_command_in_tempdir("instaloader", &args).await
}

/// Download a URL with yt-dlp. `format` can be "best" or a merged selector
/// like "bestvideo[ext=mp4]+bestaudio/best".
///
/// # Errors
///
/// - Propagates `run_command_in_tempdir` errors.
pub async fn download_ytdlp(url: &str, format: &str) -> Result<DownloadResult> {
    let args = [
        "--no-playlist",
        "-f",
        format,
        "--merge-output-format",
        "mp4",
        "--restrict-filenames",
        "-o",
        "%(id)s.%(ext)s",
        url,
    ];
    run_command_in_tempdir("yt-dlp", &args).await
}

/// Post-process a `DownloadResult`.
///
/// Detect media kinds (async), prefer video, then image, then call `send_media_from_path`.
/// Keeps the tempdir alive while sending because `DownloadResult` is passed by value.
///
/// # Errors
///
/// - Propagates `send_media_from_path` errors or returns NoMediaFound/UnknownMediaKind.
pub async fn process_download_result(bot: &Bot, chat_id: ChatId, dr: DownloadResult) -> Result<()> {
    // detect kinds in parallel
    let concurrency = 8;
    let results = stream::iter(dr.files.into_iter().map(|path| async move {
        let kind = detect_media_kind_async(&path).await;
        match kind {
            MediaKind::Unknown => None,
            k => Some((path, k)),
        }
    }))
    .buffer_unordered(concurrency)
    .collect::<Vec<Option<(PathBuf, MediaKind)>>>()
    .await;

    let mut media = results
        .into_iter()
        .flatten()
        .collect::<Vec<(PathBuf, MediaKind)>>();

    if media.is_empty() {
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

    Err(Error::NoMediaFound)
}
