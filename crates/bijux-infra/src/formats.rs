use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{IoError, IoErrorKind};

/// Parse TOML configuration.
///
/// # Errors
/// Returns an error if parsing fails.
pub fn parse_toml<T: DeserializeOwned>(contents: &str) -> Result<T, IoError> {
    toml::from_str(contents)
        .map_err(|err| IoError::new(IoErrorKind::Corruption, format!("toml parse error: {err}")))
}

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

/// Render TOML configuration.
///
/// # Errors
/// Returns an error if serialization fails.
pub fn to_toml_string<T: Serialize>(value: &T) -> Result<String, IoError> {
    toml::to_string(value)
        .map_err(|err| IoError::new(IoErrorKind::Other, format!("toml encode error: {err}")))
}

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
