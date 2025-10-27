use crate::handlers::SocialHandler;
use crate::lazy_regex;
use crate::{
    download::{download_ytdlp, process_download_result},
    error::Result,
};
use teloxide::{Bot, types::ChatId};
use tracing::info;

lazy_regex!(
    URL_RE,
    r#"https?:\/\/(?:www\.)?youtube\.com\/shorts\/[A-Za-z0-9_-]+(?:\?[^\s]*)?"#
);

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
        regex().find(text).map(|m| m.as_str().to_owned())
    }

    async fn handle(&self, bot: &Bot, chat_id: ChatId, url: String) -> Result<()> {
        info!(handler = %self.name(), url = %url, "handling youtube url");
        let dr = download_ytdlp(&url).await?;
        process_download_result(bot, chat_id, dr).await
    }

    fn box_clone(&self) -> Box<dyn SocialHandler> {
        Box::new(self.clone())
    }
}
