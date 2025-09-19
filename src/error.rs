use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("instaloader failed: {0}")]
    InstaloaderFaileled(String),

    #[error("no media found")]
    NoMediaFound,

    #[error("unknown media kind")]
    UnknownMediaKind,

    #[error("teloxide error: {0}")]
    Teloxide(#[from] teloxide::RequestError),

    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("other: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
