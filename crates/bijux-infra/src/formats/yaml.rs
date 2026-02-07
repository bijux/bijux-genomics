//! YAML parsing/rendering helpers for config compatibility.
//!
//! YAML is intentionally constrained; see docs/WHY_YAML.md for the
//! allowlist rationale.

#[cfg(feature = "yaml")]
use serde::de::DeserializeOwned;
#[cfg(feature = "yaml")]
use serde::Serialize;

#[cfg(feature = "yaml")]
use crate::{IoError, IoErrorKind};

/// Parse YAML data (optional feature).
///
/// # Errors
/// Returns an error if parsing fails.
#[cfg(feature = "yaml")]
pub fn parse_yaml<T: DeserializeOwned>(contents: &str) -> Result<T, IoError> {
    serde_yaml::from_str(contents)
        .map_err(|err| IoError::new(IoErrorKind::Corruption, format!("yaml parse error: {err}")))
}

/// Render YAML data (optional feature).
///
/// # Errors
/// Returns an error if serialization fails.
#[cfg(feature = "yaml")]
pub fn to_yaml_string<T: Serialize>(value: &T) -> Result<String, IoError> {
    serde_yaml::to_string(value)
        .map_err(|err| IoError::new(IoErrorKind::Other, format!("yaml encode error: {err}")))
}
