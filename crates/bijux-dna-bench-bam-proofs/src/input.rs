use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use std::path::Path;
/// Decode the required native record while preserving path-specific IO and schema diagnostics.
pub(crate) fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    serde_json::from_str(
        &std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))
}
#[cfg(test)]
mod tests;
