use anyhow::Result;
use sha2::Digest;
use std::path::{Component, Path};

/// Canonical params hash for run identity.
///
/// # Errors
/// Returns an error if serialization fails.
pub fn params_hash(params: &serde_json::Value) -> Result<String> {
    let bytes = serde_json::to_vec(&parameters_json_canonicalization(params))?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Deterministic parameters fingerprint for cache keys.
///
/// # Errors
/// Returns an error if serialization fails.
pub fn parameters_fingerprint(params: &serde_json::Value) -> Result<String> {
    params_hash(params)
}

/// Deterministic run id derived from pipeline identity and hashes.
#[must_use]
pub fn run_id_from_hashes(
    pipeline_id: &str,
    sample_id: &str,
    params_hash: &str,
    input_hashes: &[String],
    reference_genome: Option<&str>,
) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(pipeline_id.as_bytes());
    hasher.update(b"|");
    hasher.update(sample_id.as_bytes());
    hasher.update(b"|");
    hasher.update(params_hash.as_bytes());
    hasher.update(b"|");
    for hash in input_hashes {
        hasher.update(hash.as_bytes());
        hasher.update(b",");
    }
    hasher.update(b"|");
    if let Some(reference) = reference_genome {
        hasher.update(reference.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

#[must_use]
pub fn input_fingerprint(input_hashes: &[String]) -> String {
    let mut input_hashes_sorted = input_hashes.to_vec();
    input_hashes_sorted.sort();
    input_hashes_sorted.dedup();
    let mut hasher = sha2::Sha256::new();
    for hash in input_hashes_sorted {
        hasher.update(hash.as_bytes());
        hasher.update(b",");
    }
    format!("{:x}", hasher.finalize())
}

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

#[must_use]
pub fn parameters_json_canonicalization(value: &serde_json::Value) -> serde_json::Value {
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

    let canonical = canonicalize_json_value(value);
    normalize_numbers_and_paths(&canonical)
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
                // Preserve root with empty marker.
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
