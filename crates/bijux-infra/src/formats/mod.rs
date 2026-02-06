use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{IoError, IoErrorKind};

pub mod json;
pub mod yaml;

pub use json::*;
pub use yaml::*;

/// Parse TOML configuration.
///
/// # Errors
/// Returns an error if parsing fails.
pub fn parse_toml<T: DeserializeOwned>(contents: &str) -> Result<T, IoError> {
    toml::from_str(contents)
        .map_err(|err| IoError::new(IoErrorKind::Corruption, format!("toml parse error: {err}")))
}

/// Render TOML configuration.
///
/// # Errors
/// Returns an error if serialization fails.
pub fn to_toml_string<T: Serialize>(value: &T) -> Result<String, IoError> {
    toml::to_string(value)
        .map_err(|err| IoError::new(IoErrorKind::Other, format!("toml encode error: {err}")))
}
