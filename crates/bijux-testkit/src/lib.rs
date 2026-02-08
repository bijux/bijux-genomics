//! Shared test fixtures/helpers for bijux crates.
//! This crate must stay small and test-only.

pub mod fixtures {
    use std::fs;
    use std::path::Path;

    use serde_json::Value;

    #[must_use]
    pub fn load_fixture_text(path: impl AsRef<Path>) -> String {
        fs::read_to_string(path.as_ref()).unwrap_or_else(|err| {
            panic!("failed to read fixture {}: {err}", path.as_ref().display())
        })
    }

    #[must_use]
    pub fn load_fixture_json(path: impl AsRef<Path>) -> Value {
        let raw = load_fixture_text(path);
        serde_json::from_str(&raw).expect("fixture JSON must parse")
    }

    pub fn assert_json_schema_like(value: &Value, schema_like: &Value) {
        match (value, schema_like) {
            (Value::Object(actual), Value::Object(schema)) => {
                for key in schema.keys() {
                    assert!(
                        actual.contains_key(key),
                        "missing expected key '{}' in json payload",
                        key
                    );
                }
            }
            _ => panic!("schema_like must be a JSON object"),
        }
    }
}

pub mod determinism {
    use serde_json::Value;

    use crate::snapshots::stable_json;

    #[must_use]
    pub fn strip_timestamp_fields(value: &Value, fields: &[&str]) -> Value {
        match value {
            Value::Object(map) => {
                let mut next = serde_json::Map::new();
                for (k, v) in map {
                    if fields.iter().any(|field| field == k) {
                        continue;
                    }
                    next.insert(k.clone(), strip_timestamp_fields(v, fields));
                }
                Value::Object(next)
            }
            Value::Array(items) => Value::Array(
                items
                    .iter()
                    .map(|v| strip_timestamp_fields(v, fields))
                    .collect(),
            ),
            _ => value.clone(),
        }
    }

    pub fn assert_stable_ordering<T: Ord + std::fmt::Debug + Clone>(items: &[T]) {
        let mut sorted = items.to_vec();
        sorted.sort();
        assert_eq!(items, sorted, "items must be sorted deterministically");
    }

    pub fn assert_json_stable(expected: &Value, actual: &Value) {
        assert_eq!(
            stable_json(expected),
            stable_json(actual),
            "JSON must be deterministically ordered"
        );
    }
}

pub mod snapshots {
    use serde_json::Value;
    use std::env;

    #[must_use]
    pub fn snapshot_name(bucket: &str, test_name: &str) -> String {
        let pkg =
            std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| env!("CARGO_PKG_NAME").to_string());
        format!("{pkg}__{bucket}__{test_name}")
    }

    #[must_use]
    pub fn stable_json(value: &Value) -> Value {
        sort_value(value)
    }

    #[must_use]
    pub fn sanitize_snapshot_text(input: &str) -> String {
        let mut out = input.to_string();
        if let Ok(home) = env::var("HOME") {
            out = out.replace(&home, "<HOME>");
        }
        if let Ok(tmpdir) = env::var("TMPDIR") {
            out = out.replace(&tmpdir, "<TMPDIR>");
        }
        out
    }

    #[must_use]
    pub fn sanitize_snapshot_json(value: &Value) -> Value {
        match value {
            Value::String(s) => Value::String(sanitize_snapshot_text(s)),
            Value::Array(items) => Value::Array(items.iter().map(sanitize_snapshot_json).collect()),
            Value::Object(map) => {
                let mut next = serde_json::Map::new();
                for (k, v) in map {
                    next.insert(k.clone(), sanitize_snapshot_json(v));
                }
                Value::Object(next)
            }
            _ => value.clone(),
        }
    }

    pub fn install_snapshot_env() {
        if env::var("TZ").is_err() {
            env::set_var("TZ", "UTC");
        }
        if env::var("LC_ALL").is_err() {
            env::set_var("LC_ALL", "C");
        }
    }

    fn sort_value(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut entries: Vec<(String, Value)> = map
                    .iter()
                    .map(|(k, v)| (k.clone(), sort_value(v)))
                    .collect();
                entries.sort_by(|a, b| a.0.cmp(&b.0));
                let mut sorted = serde_json::Map::new();
                for (k, v) in entries {
                    sorted.insert(k, v);
                }
                Value::Object(sorted)
            }
            Value::Array(items) => Value::Array(items.iter().map(sort_value).collect()),
            _ => value.clone(),
        }
    }
}

pub use determinism::{assert_json_stable, assert_stable_ordering, strip_timestamp_fields};
pub use fixtures::{assert_json_schema_like, load_fixture_json, load_fixture_text};
pub use snapshots::{
    install_snapshot_env, sanitize_snapshot_json, sanitize_snapshot_text, snapshot_name,
    stable_json,
};
