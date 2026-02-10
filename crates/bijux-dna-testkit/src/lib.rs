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
        if let Ok(pwd) = env::current_dir() {
            out = out.replace(&pwd.display().to_string(), "<ROOT>");
        }
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            out = out.replace(&manifest_dir, "<ROOT>");
        }
        if let Ok(tmpdir) = env::var("TMPDIR") {
            out = out.replace(&tmpdir, "<TMPDIR>");
        }
        if let Ok(tmp) = env::var("TMP") {
            out = out.replace(&tmp, "<TMPDIR>");
        }
        if let Ok(temp) = env::var("TEMP") {
            out = out.replace(&temp, "<TMPDIR>");
        }
        out = normalize_tmp_subdir(&out);
        if let Ok(home) = env::var("HOME") {
            out = out.replace(&home, "<HOME>");
        }
        if let Ok(user) = env::var("USER") {
            out = out.replace(&user, "<USER>");
        }
        if let Ok(logname) = env::var("LOGNAME") {
            out = out.replace(&logname, "<USER>");
        }
        if let Ok(hostname) = env::var("HOSTNAME") {
            out = out.replace(&hostname, "<HOSTNAME>");
        }
        if let Ok(hostname) = env::var("COMPUTERNAME") {
            out = out.replace(&hostname, "<HOSTNAME>");
        }
        out
    }

    fn normalize_tmp_subdir(input: &str) -> String {
        let marker = "<TMPDIR>";
        let mut out = String::with_capacity(input.len());
        let mut idx = 0;
        while let Some(pos) = input[idx..].find(marker) {
            let start = idx + pos;
            out.push_str(&input[idx..start]);
            let after_marker = start + marker.len();
            let mut seg_start = after_marker;
            let bytes = input.as_bytes();
            if seg_start < bytes.len() && bytes[seg_start] == b'/' {
                seg_start += 1;
            }
            let mut seg_end = seg_start;
            while seg_end < bytes.len() {
                let b = bytes[seg_end];
                if b == b'/'
                    || b.is_ascii_whitespace()
                    || b == b','
                    || b == b')'
                    || b == b'"'
                    || b == b'\''
                {
                    break;
                }
                seg_end += 1;
            }
            if seg_end > seg_start {
                out.push_str("<TMPDIR>/<TMP>");
            } else {
                out.push_str("<TMPDIR>");
            }
            idx = seg_end;
        }
        out.push_str(&input[idx..]);
        out
    }

    #[must_use]
    pub fn sanitize_snapshot_json(value: &Value) -> Value {
        snapshot_normalize(value)
    }

    #[must_use]
    pub fn snapshot_normalize_text(input: &str) -> String {
        sanitize_snapshot_text(input)
    }

    #[must_use]
    pub fn snapshot_normalize(value: &Value) -> Value {
        let scrubbed = strip_unstable_fields(value);
        let normalized = normalize_json(&scrubbed);
        stable_json_with_arrays(&normalized)
    }

    #[must_use]
    pub fn snapshot_normalize_json(value: &Value) -> Value {
        snapshot_normalize(value)
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

    fn strip_unstable_fields(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut next = serde_json::Map::new();
                for (k, v) in map {
                    if is_unstable_key(k) {
                        continue;
                    }
                    next.insert(k.clone(), strip_unstable_fields(v));
                }
                Value::Object(next)
            }
            Value::Array(items) => Value::Array(items.iter().map(strip_unstable_fields).collect()),
            Value::String(s) => Value::String(sanitize_snapshot_text(s)),
            _ => value.clone(),
        }
    }

    fn normalize_json(value: &Value) -> Value {
        match value {
            Value::Array(items) => Value::Array(items.iter().map(normalize_json).collect()),
            Value::Object(map) => {
                let mut next = serde_json::Map::new();
                for (k, v) in map {
                    next.insert(k.clone(), normalize_json(v));
                }
                Value::Object(next)
            }
            Value::String(s) => Value::String(normalize_string(s)),
            _ => value.clone(),
        }
    }

    fn normalize_string(input: &str) -> String {
        let mut out = sanitize_snapshot_text(input);
        out = normalize_isolate_tmp_path(&out);
        if looks_like_timestamp(&out) {
            out = "<TIMESTAMP>".to_string();
        }
        if looks_like_duration(&out) {
            out = "<DURATION>".to_string();
        }
        out
    }

    fn normalize_isolate_tmp_path(input: &str) -> String {
        if input.chars().any(char::is_whitespace) {
            return input.to_string();
        }
        let is_isolate_path = input.contains("artifacts/isolates/")
            || input.contains("artifacts/target-isolate-")
            || input.contains("/target-test/tmp/")
            || input.contains("/target-cov/tmp/")
            || input.contains("/target-isolate-")
            || input.contains("<TMPDIR>/<TMP>/");
        if !is_isolate_path || !input.contains('/') {
            return input.to_string();
        }
        let trimmed = input.trim_end_matches('/');
        let leaf = trimmed.rsplit('/').next().unwrap_or(trimmed);
        leaf.to_string()
    }

    fn looks_like_timestamp(value: &str) -> bool {
        value.contains('T') && value.contains(':') && value.contains('-')
    }

    fn looks_like_duration(value: &str) -> bool {
        let trimmed = value.trim();
        if let Some(prefix) = trimmed.strip_suffix("ms") {
            return is_number(prefix);
        }
        if let Some(prefix) = trimmed.strip_suffix("sec") {
            return is_number(prefix);
        }
        if let Some(prefix) = trimmed.strip_suffix('s') {
            return is_number(prefix);
        }
        false
    }

    fn is_number(value: &str) -> bool {
        let value = value.trim();
        !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit() || ch == '.')
    }

    fn is_unstable_key(key: &str) -> bool {
        matches!(
            key,
            "timestamp"
                | "time"
                | "date"
                | "datetime"
                | "started_at"
                | "ended_at"
                | "duration"
                | "duration_ms"
                | "elapsed"
                | "elapsed_ms"
        )
    }

    fn stable_json_with_arrays(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut entries: Vec<(String, Value)> = map
                    .iter()
                    .map(|(k, v)| (k.clone(), stable_json_with_arrays(v)))
                    .collect();
                entries.sort_by(|a, b| a.0.cmp(&b.0));
                let mut sorted = serde_json::Map::new();
                for (k, v) in entries {
                    sorted.insert(k, v);
                }
                Value::Object(sorted)
            }
            Value::Array(items) => {
                Value::Array(items.iter().map(stable_json_with_arrays).collect())
            }
            _ => value.clone(),
        }
    }
}

pub use determinism::{assert_json_stable, assert_stable_ordering, strip_timestamp_fields};
pub use fixtures::{assert_json_schema_like, load_fixture_json, load_fixture_text};
pub use snapshots::{
    install_snapshot_env, sanitize_snapshot_json, sanitize_snapshot_text, snapshot_name,
    snapshot_normalize_json, snapshot_normalize_text, stable_json,
};
