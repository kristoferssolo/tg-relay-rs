use teloxide::{prelude::*, utils::command::BotCommands};

use crate::comments::{Comments, global_comments};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    /// Display this text.
    #[command(aliases = ["h", "?"])]
    Help,
    /// Send a random comment
    #[command()]
    Curse,
}

/// Handle a command from the user.
///
/// # Errors
///
/// Returns a Teloxide error if the message fails to send.
pub async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Curse => {
            let comment = global_comments().map(Comments::build_caption);
            bot.send_message(msg.chat.id, comment.unwrap_or_else(|| "To comment".into()))
                .await?
        }
    };

    Ok(())
}
