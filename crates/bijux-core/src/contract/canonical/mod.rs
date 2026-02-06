//! Canonical serialization helpers for contract artifacts.

use serde::Serialize;
use std::path::{Component, Path};

use crate::foundation::Result;

pub const CANONICAL_JSON_VERSION: &str = "v1";

/// Canonicalize a JSON value by sorting keys and normalizing numbers/paths.
#[must_use]
pub fn canonicalize_json_value(value: &serde_json::Value) -> serde_json::Value {
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

/// Canonicalize parameters for stable hashing (sorted keys + normalized values).
#[must_use]
pub fn parameters_json_canonicalization(value: &serde_json::Value) -> serde_json::Value {
    normalize_numbers_and_paths(&canonicalize_json_value(value))
}

/// Canonicalize JSON for truth artifacts (manifests, records, reports).
#[must_use]
pub fn canonicalize_truth_json(value: &serde_json::Value) -> serde_json::Value {
    normalize_numbers_and_paths(&canonicalize_json_value(value))
}

fn looks_like_path(value: &str) -> bool {
    value.contains('/') || value.contains('\\')
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
    if path.is_absolute() && !normalized.starts_with('/') {
        normalized = format!("/{normalized}");
    }
    normalized
}

fn normalize_numbers_and_paths(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Number(num) => {
            if let Some(f) = num.as_f64() {
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
                serde_json::Value::String(s.clone())
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

/// Canonical serializer for truth artifacts.
///
/// # Errors
/// Returns an error if serialization fails.
pub fn to_canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let json = serde_json::to_value(value)?;
    let canonical = canonicalize_truth_json(&json);
    Ok(serde_json::to_vec(&canonical)?)
}
