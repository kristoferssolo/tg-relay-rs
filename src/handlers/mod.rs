#[cfg(feature = "instagram")]
mod instagram;
#[cfg(feature = "tiktok")]
mod tiktok;
#[cfg(feature = "twitter")]
mod twitter;
#[cfg(feature = "youtube")]
mod youtube;

use crate::error::Result;
use teloxide::{Bot, types::ChatId};

#[cfg(feature = "instagram")]
pub use instagram::InstagramHandler;
#[cfg(feature = "tiktok")]
pub use tiktok::TiktokHandler;
#[cfg(feature = "twitter")]
pub use twitter::TwitterHandler;
#[cfg(feature = "youtube")]
pub use youtube::YouTubeShortsHandler;

#[macro_export]
macro_rules! lazy_regex {
    ($name:ident, $pattern:expr) => {
        static $name: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

        fn regex() -> &'static regex::Regex {
            $name.get_or_init(|| regex::Regex::new($pattern).expect("failed to compile regex"))
        }
    };
}

#[async_trait::async_trait]
pub trait SocialHandler: Send + Sync {
    /// Short name used for logging etc.
    fn name(&self) -> &'static str;

    /// Try to extract a platform-specific identifier (shortcode, id, url)
    /// from arbitrary text. Return `Some` if the handler should handle this message.
    fn try_extract(&self, text: &str) -> Option<String>;

    /// Do the heavy-lifting: fetch media and send to `chat_id`.
    async fn handle(&self, bot: &Bot, chat_id: ChatId, id: String) -> Result<()>;

    /// Clone a boxed handler.
    fn box_clone(&self) -> Box<dyn SocialHandler>;
}

impl Clone for Box<dyn SocialHandler> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}
