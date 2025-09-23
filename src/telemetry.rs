#[cfg(feature = "bunyan")]
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialise tracing with bunyan-style JSON output.
#[cfg(feature = "bunyan")]
pub fn setup_logger() {
    let env_filter = create_env_filter();
    let formatter = BunyanFormattingLayer::new("tg-relay-rs".into(), std::io::stdout);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(formatter)
        .with(JsonStorageLayer)
        .init()
}

#[cfg(not(feature = "bunyan"))]
pub fn setup_logger() {
    let env_filter = create_env_filter();
    let formatter = tracing_subscriber::fmt::Layer::default();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(formatter)
        .init()
}

fn create_env_filter() -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into())
}
