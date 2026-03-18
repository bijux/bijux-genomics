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
    let raw = load_fixture_text(path);
    serde_json::from_str(&raw).unwrap_or_else(|err| panic!("fixture JSON must parse: {err}"))
}

/// Assert that `value` contains all top-level keys present in `schema_like`.
///
/// # Panics
/// Panics if `schema_like` is not a JSON object or if any expected key is missing.
pub fn assert_json_schema_like(value: &Value, schema_like: &Value) {
    match (value, schema_like) {
        (Value::Object(actual), Value::Object(schema)) => {
            for key in schema.keys() {
                assert!(
                    actual.contains_key(key),
                    "missing expected key '{key}' in json payload"
                );
            }
        }
        _ => panic!("schema_like must be a JSON object"),
    }
}
