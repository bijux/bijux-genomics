use serde_json::Value;

use super::text_normalization::sanitize_snapshot_text;

#[must_use]
pub fn stable_json(value: &Value) -> Value {
    stable_json_with_arrays(value)
}

#[must_use]
pub fn sanitize_snapshot_json(value: &Value) -> Value {
    snapshot_normalize(value)
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

fn strip_unstable_fields(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut next = serde_json::Map::new();
            for (key, nested) in map {
                if is_unstable_key(key) {
                    continue;
                }
                next.insert(key.clone(), strip_unstable_fields(nested));
            }
            Value::Object(next)
        }
        Value::Array(items) => Value::Array(items.iter().map(strip_unstable_fields).collect()),
        Value::String(value) => Value::String(sanitize_snapshot_text(value)),
        _ => value.clone(),
    }
}

fn normalize_json(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(normalize_json).collect()),
        Value::Object(map) => {
            let mut next = serde_json::Map::new();
            for (key, nested) in map {
                next.insert(key.clone(), normalize_json(nested));
            }
            Value::Object(next)
        }
        Value::String(value) => Value::String(normalize_string(value)),
        _ => value.clone(),
    }
}

fn normalize_string(input: &str) -> String {
    let mut out = sanitize_snapshot_text(input);
    out = normalize_artifact_tmp_path(&out);
    if looks_like_timestamp(&out) {
        out = "<TIMESTAMP>".to_string();
    }
    if looks_like_duration(&out) {
        out = "<DURATION>".to_string();
    }
    out
}

fn normalize_artifact_tmp_path(input: &str) -> String {
    if input.chars().any(char::is_whitespace) {
        return input
            .split_inclusive(char::is_whitespace)
            .map(normalize_artifact_tmp_path_token)
            .collect();
    }
    normalize_artifact_tmp_path_token(input)
}

fn normalize_artifact_tmp_path_token(input: &str) -> String {
    let is_artifact_path = input.contains("artifacts/target/")
        || input.contains("artifacts/tmp/")
        || input.contains("artifacts/coverage/profraw-")
        || input.contains("<TMPDIR>/<TMP>/");
    if !is_artifact_path || !input.contains('/') {
        return input.to_string();
    }
    let trimmed = input.trim_end_matches('/');
    let leaf = trimmed.rsplit('/').next().unwrap_or(trimmed);
    leaf.to_string()
}

fn looks_like_timestamp(value: &str) -> bool {
    let value = value.trim();
    let bytes = value.as_bytes();
    if bytes.len() < "2000-01-01T00:00:00".len() {
        return false;
    }
    matches!(
        (
            digit(bytes, 0),
            digit(bytes, 1),
            digit(bytes, 2),
            digit(bytes, 3),
            bytes.get(4),
            digit(bytes, 5),
            digit(bytes, 6),
            bytes.get(7),
            digit(bytes, 8),
            digit(bytes, 9),
            bytes.get(10),
            digit(bytes, 11),
            digit(bytes, 12),
            bytes.get(13),
            digit(bytes, 14),
            digit(bytes, 15),
            bytes.get(16),
            digit(bytes, 17),
            digit(bytes, 18),
        ),
        (
            true,
            true,
            true,
            true,
            Some(b'-'),
            true,
            true,
            Some(b'-'),
            true,
            true,
            Some(b'T'),
            true,
            true,
            Some(b':'),
            true,
            true,
            Some(b':'),
            true,
            true
        )
    )
}

fn digit(bytes: &[u8], index: usize) -> bool {
    bytes.get(index).is_some_and(u8::is_ascii_digit)
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
    if value.is_empty() {
        return false;
    }
    let mut seen_digit = false;
    let mut seen_dot = false;
    for ch in value.chars() {
        if ch.is_ascii_digit() {
            seen_digit = true;
        } else if ch == '.' && !seen_dot {
            seen_dot = true;
        } else {
            return false;
        }
    }
    seen_digit
}

fn is_unstable_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "timestamp"
            | "time"
            | "date"
            | "datetime"
            | "startedat"
            | "started_at"
            | "endedat"
            | "ended_at"
            | "duration"
            | "durationms"
            | "duration_ms"
            | "elapsed"
            | "elapsedms"
            | "elapsed_ms"
    )
}

fn stable_json_with_arrays(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = map
                .iter()
                .map(|(key, nested)| (key.clone(), stable_json_with_arrays(nested)))
                .collect();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            let mut sorted = serde_json::Map::new();
            for (key, nested) in entries {
                sorted.insert(key, nested);
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.iter().map(stable_json_with_arrays).collect()),
        _ => value.clone(),
    }
}
