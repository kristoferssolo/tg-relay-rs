use crate::error::{Error, Result};
use std::{env, fmt::Debug, path::PathBuf, sync::OnceLock};

static GLOBAL_CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub youtube: YoutubeConfig,
    pub instagram: InstagramConfig,
    pub tiktok: TiktokConfig,
    pub twitter: TwitterConfig,
}

#[derive(Debug, Clone)]
pub struct YoutubeConfig {
    pub cookies_path: Option<PathBuf>,
    pub postprocessor_args: String,
}

#[derive(Debug, Clone, Default)]
pub struct InstagramConfig {
    pub cookies_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct TiktokConfig {
    pub cookies_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct TwitterConfig {
    pub cookies_path: Option<PathBuf>,
}

impl Config {
    /// Load configuration from environment variables.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            youtube: YoutubeConfig::from_env(),
            instagram: InstagramConfig::from_env(),
            tiktok: TiktokConfig::from_env(),
            twitter: TwitterConfig::from_env(),
        }
    }

    /// Initialize the global config (call once at startup).
    ///
    /// # Errors
    ///
    /// Returns error if config is already initialized.
    pub fn init(self) -> Result<()> {
        GLOBAL_CONFIG
            .set(self)
            .map_err(|_| Error::other("config already initialized"))
    }
}
/// Get global config (initialized by `Config::init(self)`).
#[must_use]
pub fn global_config() -> Config {
    GLOBAL_CONFIG.get().cloned().unwrap_or_default()
}

impl YoutubeConfig {
    const DEFAULT_POSTPROCESSOR_ARGS: &'static str = "ffmpeg:-vf setsar=1 -c:v libx264 -crf 20 -preset ultrafast -c:a aac -b:a 128k -movflags +faststart";

    fn from_env() -> Self {
        Self {
            cookies_path: get_path_from_env("YOUTUBE_SESSION_COOKIE_PATH"),
            postprocessor_args: env::var("YOUTUBE_POSTPROCESSOR_ARGS")
                .unwrap_or_else(|_| Self::DEFAULT_POSTPROCESSOR_ARGS.to_string()),
        }
    }
}

impl InstagramConfig {
    fn from_env() -> Self {
        Self {
            cookies_path: get_path_from_env("IG_SESSION_COOKIE_PATH"),
        }
    }
}

impl TiktokConfig {
    fn from_env() -> Self {
        Self {
            cookies_path: get_path_from_env("TIKTOK_SESSION_COOKIE_PATH"),
        }
    }
}

impl TwitterConfig {
    fn from_env() -> Self {
        Self {
            cookies_path: get_path_from_env("TWITTER_SESSION_COOKIE_PATH"),
        }
    }
}

fn get_path_from_env(key: &str) -> Option<PathBuf> {
    env::var(key).ok().map(PathBuf::from)
}

impl Default for YoutubeConfig {
    fn default() -> Self {
        Self {
            cookies_path: None,
            postprocessor_args: Self::DEFAULT_POSTPROCESSOR_ARGS.into(),
        }
    }
}
