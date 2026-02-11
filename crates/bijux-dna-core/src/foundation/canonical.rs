use std::path::{Component, Path};

pub(crate) fn canonicalize_json_value(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut ordered = serde_json::Map::new();
            for key in keys {
                let val = map.get(key).unwrap_or(&serde_json::Value::Null);
                ordered.insert(key.clone(), canonicalize_json_value(val));
            }
            serde_json::Value::Object(ordered)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(canonicalize_json_value).collect())
        }
        _ => value.clone(),
    }
}

pub(crate) fn parameters_json_canonicalization(value: &serde_json::Value) -> serde_json::Value {
    normalize_numbers_and_paths(&canonicalize_json_value(value))
}

pub(crate) fn canonicalize_truth_json(value: &serde_json::Value) -> serde_json::Value {
    normalize_numbers_and_paths(&canonicalize_json_value(value))
}

fn looks_like_path(value: &str) -> bool {
    value.contains('/') || value.contains('\\')
}

fn looks_like_hostname(value: &str) -> bool {
    value.contains('.') && !value.contains(' ')
}

fn normalize_sensitive_string(value: &str) -> String {
    let username = std::env::var("USER")
        .ok()
        .or_else(|| std::env::var("USERNAME").ok())
        .unwrap_or_default();
    let hostname = std::env::var("HOSTNAME").unwrap_or_default();
    let mut normalized = value.to_string();
    if !username.is_empty() {
        normalized = normalized.replace(&username, "<user>");
    }
    if !hostname.is_empty() {
        normalized = normalized.replace(&hostname, "<host>");
    }
    if looks_like_hostname(&normalized) && normalized.ends_with(".local") {
        return "<host>".to_string();
    }
    normalized
}

fn normalize_path_string(value: &str) -> String {
    let path = Path::new(value);
    let mut components: Vec<String> = Vec::new();
    let mut prefix: Option<String> = None;
    for comp in path.components() {
        match comp {
            Component::Prefix(prefix_component) => {
                prefix = Some(prefix_component.as_os_str().to_string_lossy().to_string());
            }
            Component::RootDir => {
                if components.is_empty() {
                    components.push(String::new());
                }
            }
            Component::CurDir => {}
            Component::ParentDir => {
                components.pop();
            }
            Component::Normal(part) => {
                components.push(part.to_string_lossy().to_string());
            }
        }
    }
    let mut normalized = components.join("/");
    if let Some(prefix) = prefix {
        normalized = format!("{prefix}/{normalized}");
    }
    if path.is_absolute() {
        if let Some(stripped) = strip_to_stable_tail(&components) {
            return stripped;
        }
        return components
            .last()
            .cloned()
            .unwrap_or_else(|| normalized.clone());
    }
    normalized
}

fn normalize_numbers_and_paths(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Number(num) => {
            if num.is_i64() || num.is_u64() {
                serde_json::Value::Number(num.clone())
            } else if let Some(f) = num.as_f64() {
                serde_json::Number::from_f64(f).map_or_else(
                    || serde_json::Value::Number(num.clone()),
                    serde_json::Value::Number,
                )
            } else {
                serde_json::Value::Number(num.clone())
            }
        }
        serde_json::Value::String(s) => {
            if looks_like_path(s) {
                serde_json::Value::String(normalize_path_string(s))
            } else {
                serde_json::Value::String(normalize_sensitive_string(s))
            }
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(normalize_numbers_and_paths).collect())
        }
        serde_json::Value::Object(map) => {
            let mut ordered = serde_json::Map::new();
            for (key, val) in map {
                ordered.insert(key.clone(), normalize_numbers_and_paths(val));
            }
            serde_json::Value::Object(ordered)
        }
        _ => value.clone(),
    }
}

fn strip_to_stable_tail(components: &[String]) -> Option<String> {
    let markers = ["bench", "run_artifacts", "artifacts", "runs"];
    for (idx, part) in components.iter().enumerate() {
        if markers.iter().any(|marker| marker == part) {
            let tail = components[idx..].join("/");
            return if tail.is_empty() { None } else { Some(tail) };
        }
    }
    None
}
