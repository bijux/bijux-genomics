use std::fs;
use std::path::Path;

use serde_json::Value;

/// Load a UTF-8 fixture file.
///
/// # Panics
/// Panics if the file cannot be read.
#[must_use]
pub fn load_fixture_text(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|err| panic!("failed to read fixture {}: {err}", path.as_ref().display()))
}

/// Load and parse a JSON fixture file.
///
/// # Panics
/// Panics if the file cannot be read or parsed as JSON.
#[must_use]
pub fn load_fixture_json(path: impl AsRef<Path>) -> Value {
    let path = path.as_ref();
    let raw = load_fixture_text(path);
    serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("fixture JSON {} must parse: {err}", path.display()))
}
