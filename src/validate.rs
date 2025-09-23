use crate::error::Result;
use regex::Regex;
use std::sync::OnceLock;

/// Trait for validating platform-specific identifiers (e.g., shortcodes, URLs)
/// extracted from user input.
///
/// Implementors should:
/// - Check format (e.g., length, characters).
/// - Canonicalize if needed (e.g., trim query params from a URL).
/// - Return `Ok(canonical_id)` on success or `Err(Error::Other(...))` on failure.
pub trait Validate {
    /// Validate the input and return a canonicalized String (e.g., cleaned shortcode or URL).
    fn validate(&self, input: &str) -> Result<String>;
}

/// Helper function to create a lazy static Regex (reused across impls).
///
/// # Panics
///
/// If no pattern found
pub fn lazy_regex(pattern: &str) -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(pattern).expect("failed to compile validation regex"))
}
