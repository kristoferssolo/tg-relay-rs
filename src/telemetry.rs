use color_eyre::Result;
use std::{fs::create_dir_all, path::PathBuf};
use tracing_appender::rolling;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// # Errors
pub fn setup_logger() -> Result<()> {
    let log_dir_path = PathBuf::from(".logs");
    create_dir_all(&log_dir_path)?;

    let logfile = if cfg!(debug_assertions) {
        rolling::daily(log_dir_path, "traxor.log")
    } else {
        rolling::never(log_dir_path, "traxor.log")
    };

    let formatter = BunyanFormattingLayer::new("traxor".into(), logfile);

    tracing_subscriber::registry()
        .with(JsonStorageLayer)
        .with(formatter)
        .init();

    Ok(())
}
