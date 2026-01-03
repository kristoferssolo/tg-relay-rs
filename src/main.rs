use dotenv::dotenv;
use teloxide::{prelude::*, respond, utils::command::BotCommands};
use tg_relay_rs::{
    commands::{Command, answer},
    comments::Comments,
    config::{Config, FAILED_FETCH_MEDIA_MESSAGE, global_config},
    handler::{Handler, create_handlers},
    telemetry::setup_logger,
};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenv().ok();
    color_eyre::install().expect("color-eyre install");
    setup_logger();

    Comments::load_from_file("comments.txt")
        .await
        .map_err(|e| {
            warn!("failed to load comments.txt: {e}; using dummy comments");
            e
        })
        .unwrap_or_else(|_| Comments::dummy())
        .init()?;

    Config::from_env().init()?;

    let bot = Bot::from_env();
    let bot_name = bot.get_me().await?.username().to_owned();

    info!(name = bot_name, "bot starting");

    let handlers = create_handlers();

    teloxide::repl(bot.clone(), move |bot: Bot, msg: Message| {
        let handlers = handlers.clone();
        let bot_name_cloned = bot_name.clone();
        async move {
            relay_message(&bot, &msg).await;
            process_cmd(&bot, &msg, &bot_name_cloned).await;
            process_message(&bot, &msg, &handlers).await;
            respond(())
        }
    })
    .await;

    Ok(())
}

async fn process_message(bot: &Bot, msg: &Message, handlers: &[Handler]) {
    let Some(text) = msg.text() else {
        return;
    };

    for handler in handlers {
        if let Some(url) = handler.try_extract(text) {
            if let Err(err) = handler.handle(bot, msg.chat.id, url).await {
                error!(%err, "handler failed");
                let _ = bot
                    .send_message(msg.chat.id, FAILED_FETCH_MEDIA_MESSAGE)
                    .await;
                if let Some(chat_id) = global_config().chat_id {
                    let _ = bot.send_message(chat_id, format!("{err}")).await;
                }
            }
            return;
        }
    }
}

async fn process_cmd(bot: &Bot, msg: &Message, bot_name: &str) {
    if let Some(text) = msg.text()
        && let Ok(cmd) = Command::parse(text, bot_name)
        && let Err(e) = answer(bot, msg, cmd).await
    {
        error!(%e, "failed to answer command");
    }
}

async fn relay_message(bot: &Bot, msg: &Message) {
    let Some(chat_id) = global_config().chat_id else {
        return;
    };

    // Don't relay messages from the relay target itself
    if msg.chat.id == chat_id {
        return;
    }

    let author = msg.from.as_ref().map_or_else(
        || "Unknown".to_string(),
        |u| {
            u.username
                .as_ref()
                .map_or_else(|| u.full_name(), |un| format!("@{un}"))
        },
    );

    let chat_name = msg.chat.title().unwrap_or("Private chat");

    let text = msg.text().or_else(|| msg.caption()).unwrap_or("");

    let relay_text = format!("[{chat_name}] {author}:\n{text}");

    if let Err(e) = bot.send_message(chat_id, relay_text).await {
        error!(%e, "failed to relay message");
    }
}
