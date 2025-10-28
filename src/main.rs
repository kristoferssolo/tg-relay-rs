use dotenv::dotenv;
use teloxide::{prelude::*, respond};
use tg_relay_rs::{
    comments::{Comments, init_global_comments},
    handler::{Handler, create_handlers},
    telemetry::setup_logger,
};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenv().ok();
    color_eyre::install().expect("color-eyre install");
    setup_logger();

    let comments = Comments::load_from_file("comments.txt")
        .await
        .map_err(|e| {
            warn!("failed to laod comments.txt: {}; using dummy comments", e);
            e
        })
        .unwrap_or_else(|_| Comments::dummy());

    init_global_comments(comments).expect("failed to initialize global comments");

    let bot = Bot::from_env();
    info!("bot starting");

    let handlers = create_handlers();

    // Command::repl(bot.clone(), answer).await;
    teloxide::repl(bot.clone(), move |bot: Bot, msg: Message| {
        let handlers = handlers.clone();
        async move {
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
            }
            return;
        }
    }
}
