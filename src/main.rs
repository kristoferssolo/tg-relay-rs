mod telemetry;

use crate::telemetry::setup_logger;
use color_eyre::{
    Result,
    eyre::{Context, eyre},
};
use dotenv::dotenv;
use regex::Regex;
use std::{fs::read_dir, process::Stdio};
use teloxide::{
    Bot,
    prelude::Requester,
    respond,
    types::{ChatId, InputFile, Message},
};
use tempfile::tempdir;
use tokio::process::Command;
use tracing::error;

#[tokio::main]
async fn main() -> Result<()> {
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

    let target = format!("--{}", shortcode);
    let status = Command::new("instaloader")
        .arg(dir_path.to_string_lossy().as_ref())
        .arg("--no-metadate-json")
        .arg("--no-compress-json")
        // .arg("--quiet")
        .arg(&target)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
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

    let first = &media_files[0];
    let input = InputFile::file(first.clone());

    let ext = first.extension().and_then(|s| s.to_str()).unwrap_or("");
    if matches!(ext, "jpg" | "jpeg") {
        bot.send_photo(chat_id, input).await?;
    } else {
        bot.send_video(chat_id, input).await?;
    }
    Ok(())
}
