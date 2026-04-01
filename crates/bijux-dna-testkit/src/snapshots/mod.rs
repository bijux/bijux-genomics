mod environment;
mod naming;

use std::env;

use serde_json::Value;

pub use environment::install_snapshot_env;
pub use naming::snapshot_name;

#[must_use]
pub fn stable_json(value: &Value) -> Value {
    stable_json_with_arrays(value)
}

#[must_use]
pub fn sanitize_snapshot_text(input: &str) -> String {
    let mut out = input.replace("\r\n", "\n");
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
    out.replace('\\', "/")
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
            let byte = bytes[seg_end];
            if byte == b'/'
                || byte.is_ascii_whitespace()
                || byte == b','
                || byte == b')'
                || byte == b'"'
                || byte == b'\''
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
        return input.to_string();
    }
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
