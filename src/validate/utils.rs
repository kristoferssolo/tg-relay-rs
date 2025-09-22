use crate::error::{Error, Result};

/// Trims whitespace and rejects empty strings.
pub fn validate_non_empty(input: &str) -> Result<&str> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(Error::validation_falied("input cannot be empty"));
    }
    Ok(trimmed)
}
