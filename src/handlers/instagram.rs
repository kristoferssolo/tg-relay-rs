use crate::download::{download_instagram, process_download_result};
use crate::error::Result;
use crate::handlers::SocialHandler;
use crate::lazy_regex;
use teloxide::{Bot, types::ChatId};
use tracing::info;

lazy_regex!(
    URL_RE,
    r#"https?://(?:www\.)?(?:instagram\.com|instagr\.am)/(?:p|reel|tv)/([A-Za-z0-9_-]+)"#
);

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
        regex()
            .captures(text)
            .and_then(|c| c.get(0).map(|m| m.as_str().to_owned()))
    }

    async fn handle(&self, bot: &Bot, chat_id: ChatId, url: String) -> Result<()> {
        info!(handler = %self.name(), url = %url, "handling instagram url");
        let dr = download_instagram(&url).await?;
        process_download_result(bot, chat_id, dr).await
    }

    fn box_clone(&self) -> Box<dyn SocialHandler> {
        Box::new(self.clone())
    }
}
