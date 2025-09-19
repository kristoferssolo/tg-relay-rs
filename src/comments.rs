use crate::error::{Error, Result};
use rand::{rng, seq::IndexedRandom};
use std::{
    path::Path,
    sync::{Arc, OnceLock},
};
use tokio::fs::read_to_string;
static DISCLAIMER: &str = "(Roleplay — fictional messages for entertainment.)";

#[derive(Debug)]
pub struct Comments {
    pub disclaimer: String,
    lines: Arc<Vec<String>>,
}

impl Comments {
    /// Create a small dummy/default Comments instance (useful for tests or fallback).
    #[must_use]
    pub fn dummy() -> Self {
        let lines = vec![
            "Oh come on, that's brilliant — and slightly chaotic, like always.".into(),
            "That is a proper bit of craftsmanship — then someone presses the red button.".into(),
            "Nice shot — looks good on the trailer, not so good on the gearbox.".into(),
            "Here you go. Judge for yourself.".into(),
        ];
        Self {
            disclaimer: DISCLAIMER.into(),
            lines: lines.into(),
        }
    }

    /// Load comments from a plaintext file asynchronously.
    ///
    /// # Errors
    ///
    /// - Returns `Error::Io` if reading the file fails (propagated from
    ///   `tokio::fs::read_to_string`).
    /// - Returns `Error::Other` if the file contains no usable lines after
    ///   filtering (empty or all-comment file).
    pub async fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let s = read_to_string(path).await?;

        let lines = s
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        if lines.is_empty() {
            return Err(Error::other("comments file contains no usable lines"));
        }

        Ok(Self {
            disclaimer: DISCLAIMER.into(),
            lines: lines.into(),
        })
    }

    /// Pick a random comment as &str (no allocation). Falls back to a small static
    /// string if the list is unexpectedly empty.
    #[must_use]
    pub fn pick(&self) -> &str {
        let mut rng = rng();
        self.lines
            .choose(&mut rng)
            .map_or("Here you go.", String::as_str)
    }

    #[must_use]
    #[inline]
    pub fn build_caption(&self) -> String {
        self.pick().to_string()
    }
}

static GLOBAL_COMMENTS: OnceLock<Comments> = OnceLock::new();

/// Initialize the global comments (call once at startup).
///
/// # Errors
///
/// - Returns `Error::Other` when the global is already initialized (the
///   underlying `OnceLock::set` fails).
pub fn init_global_comments(comments: Comments) -> Result<()> {
    GLOBAL_COMMENTS
        .set(comments)
        .map_err(|_| Error::other("comments already initialized"))
}

/// Get global comments (if initialized). Returns Option<&'static Comments>.
pub fn global_comments() -> Option<&'static Comments> {
    GLOBAL_COMMENTS.get()
}
