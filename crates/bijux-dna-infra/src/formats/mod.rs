//! Format helpers for configuration parsing and rendering.
//!
//! Invariants:
//! - No contract schema ownership.
//! - Stable ordering for rendered outputs.
//! - Lightweight dependencies only.

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{IoError, IoErrorKind};

pub mod json;
mod stable_surface;
#[cfg(feature = "yaml")]
pub mod yaml;

pub use stable_surface::*;

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
