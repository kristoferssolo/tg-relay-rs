use dotenv::dotenv;
use std::{env, sync::Arc};
use teloxide::{Bot, prelude::Requester, respond, types::Message};
use tg_relay_rs::{
    comments::{Comments, init_global_comments},
    handlers::SocialHandler,
    telemetry::setup_logger,
};
use tracing::{error, info, warn};

macro_rules! add_handler_if_enabled {
    ($handlers:expr, $feature:expr, $handler:expr) => {
        #[cfg(feature = $feature)]
        {
            if is_handler_enabled($feature) {
                $handlers.push(Arc::new($handler));
            }
        }
    };
}

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

    let mut handlers: Vec<Arc<dyn SocialHandler>> = Vec::new();

    add_handler_if_enabled!(
        handlers,
        "instagram",
        tg_relay_rs::handlers::InstagramHandler
    );
    add_handler_if_enabled!(
        handlers,
        "youtube",
        tg_relay_rs::handlers::YouTubeShortsHandler
    );

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
                                    .send_message(chat, "Failed to fetch media, you foking donkey.")
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

fn has_env(key: &str) -> bool {
    !matches!(env::var(key), Ok(val) if val.trim().eq_ignore_ascii_case("false"))
}

fn is_handler_enabled(handler_key: &str) -> bool {
    has_env(&handler_key.to_uppercase())
}
