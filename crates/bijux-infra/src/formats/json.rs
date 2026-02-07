//! JSON parsing/rendering helpers for non-contract payloads.
//!
//! Contract JSON canonicalization lives in bijux-core; do not add
//! canonicalizers here.

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{IoError, IoErrorKind};

/// Parse JSON data.
///
/// # Errors
/// Returns an error if parsing fails.
pub fn parse_json<T: DeserializeOwned>(contents: &str) -> Result<T, IoError> {
    serde_json::from_str(contents)
        .map_err(|err| IoError::new(IoErrorKind::Corruption, format!("json parse error: {err}")))
}

/// Render JSON with stable pretty formatting.
///
/// # Errors
/// Returns an error if serialization fails.
pub fn to_json_pretty<T: Serialize>(value: &T) -> Result<String, IoError> {
    serde_json::to_string_pretty(value)
        .map_err(|err| IoError::new(IoErrorKind::Other, format!("json encode error: {err}")))
}
