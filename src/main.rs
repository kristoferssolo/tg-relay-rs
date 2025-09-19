use dotenv::dotenv;
use std::sync::Arc;
use teloxide::{Bot, prelude::Requester, respond, types::Message};
use tg_relay_rs::{
    comments::{Comments, init_global_comments},
    handlers::{InstagramHandler, SocialHandler, YouTubeShortsHandler},
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

    let handlers: Vec<Arc<dyn SocialHandler>> =
        vec![Arc::new(InstagramHandler), Arc::new(YouTubeShortsHandler)];

    teloxide::repl(bot.clone(), move |bot: Bot, msg: Message| {
        // clone the handlers vector into the closure
        let handlers = handlers.clone();
        async move {
            if let Some(text) = msg.text() {
                for handler in handlers {
                    if let Some(id) = handler.try_extract(text) {
                        let handler = handler.clone();
                        let bot_for_task = bot.clone();
                        let chat = msg.chat.id;

                        tokio::spawn(async move {
                            if let Err(err) = handler.handle(&bot_for_task, chat, id).await {
                                error!(%err, "handler failed");

                                let _ = bot_for_task
                                    .send_message(chat, "Failed to fetch media.")
                                    .await;
                            }
                        });
                        // if one handler matcher, stop checking
                        break;
                    }
                }
            }
            respond(())
        }
    })
    .await;

    Ok(())
}
