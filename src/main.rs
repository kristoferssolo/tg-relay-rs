use dotenv::dotenv;
use teloxide::{prelude::*, respond, utils::command::BotCommands};
use tg_relay_rs::{
    commands::{Command, answer},
    comments::Comments,
    config::{Config, global_config},
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
            warn!("failed to laod comments.txt: {e}; using dummy comments");
            e
        })
        .unwrap_or_else(|_| Comments::dummy())
        .init()
        .expect("failed to initialize comments");

    Config::from_env()
        .init()
        .expect("failed to initialize config");

    let bot = Bot::from_env();
    info!("bot starting");

    let handlers = create_handlers();

    teloxide::repl(bot.clone(), move |bot: Bot, msg: Message| {
        let handlers = handlers.clone();
        async move {
            process_cmd(&bot, &msg).await;
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
                    .send_message(msg.chat.id, "Failed to fetch media, you foking donkey.")
                    .await;
                if let Some(chat_id) = global_config().chat_id {
                    let _ = bot.send_message(chat_id, format!("{err}")).await;
                }
            }
            return;
        }
    }
}

async fn process_cmd(bot: &Bot, msg: &Message) {
    if let Some(text) = msg.text()
        && let Ok(cmd) = Command::parse(text, "guenter")
    {
        let _ = answer(bot, msg, cmd).await;
    }
}
