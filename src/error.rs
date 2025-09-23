use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] tokio::io::Error),

    #[error("instaloader failed: {0}")]
    InstaloaderFailed(String),

    #[error("yt-dpl failed: {0}")]
    YTDLPFailed(String),

    #[error("no media found")]
    NoMediaFound,

    #[error("unknown media kind")]
    UnknownMediaKind,

    #[error("validation failed: {0}")]
    ValidationFailed(String),

    #[error("teloxide error: {0}")]
    Teloxide(#[from] teloxide::RequestError),

    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("rate limit exceeded")]
    RateLimit,

    #[error("other: {0}")]
    Other(String),
}

impl Error {
    #[inline]
    pub fn other(text: impl Into<String>) -> Self {
        Self::Other(text.into())
    }

    #[inline]
    pub fn instaloader_failed(text: impl Into<String>) -> Self {
        Self::InstaloaderFailed(text.into())
    }

    #[inline]
    pub fn ytdlp_failed(text: impl Into<String>) -> Self {
        Self::YTDLPFailed(text.into())
    }

    #[inline]
    pub fn validation_falied(text: impl Into<String>) -> Self {
        Self::ValidationFailed(text.into())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
