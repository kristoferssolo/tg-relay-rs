use crate::{
    download::{download_twitter, process_download_result},
    error::Result,
    lazy_regex,
};
use teloxide::{Bot, types::ChatId};
use tracing::info;

use crate::handlers::SocialHandler;

lazy_regex!(
    URL_RE,
    r#"https?://(?:www\.)?twitter\.com/([A-Za-z0-9_]+(?:/[A-Za-z0-9_]+)?)/status/(\d{1,20})"#
);

/// Handler for Tiktok
#[derive(Clone, Default)]
pub struct TwitterHandler;

impl TwitterHandler {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl SocialHandler for TwitterHandler {
    fn name(&self) -> &'static str {
        "twitter"
    }

    fn try_extract(&self, text: &str) -> Option<String> {
        regex()
            .captures(text)
            .and_then(|c| c.get(0).map(|m| m.as_str().to_owned()))
    }

    async fn handle(&self, bot: &Bot, chat_id: ChatId, url: String) -> Result<()> {
        info!(handler = %self.name(), url = %url, "handling twitter url");
        let dr = download_twitter(&url).await?;
        process_download_result(bot, chat_id, dr).await
    }

    fn box_clone(&self) -> Box<dyn SocialHandler> {
        Box::new(self.clone())
    }
}
