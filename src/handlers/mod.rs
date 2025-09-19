mod instagram;

use crate::error::Result;
use teloxide::{Bot, types::ChatId};

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

pub use instagram::InstagramHandler;
