use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialise tracing with bunyan-style JSON output.
pub fn setup_logger() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    let formatter = BunyanFormattingLayer::new("tg-relay-rs".into(), std::io::stdout);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatter)
        .init();
}
