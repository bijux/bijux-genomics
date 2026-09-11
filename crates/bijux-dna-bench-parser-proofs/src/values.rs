use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn is_json_path(path: &Path) -> bool {
    path.extension().and_then(|value| value.to_str()) == Some("json")
}

pub(crate) fn ensure_governed_keys_present(
    value: &Value,
    proof_path: &Path,
    keys: &[&str],
) -> Result<()> {
    for key in keys {
        if value.get(*key).is_none() {
            return Err(anyhow!("{} is missing `{key}`", proof_path.display()));
        }
    }
    Ok(())
}

pub(crate) fn read_json_document(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

pub(crate) fn top_level_keys(value: &Value) -> Vec<String> {
    value.as_object().map(|object| object.keys().cloned().collect::<Vec<_>>()).unwrap_or_default()
}

pub(crate) fn json_snapshot(value: &Value, keys: &[&str]) -> BTreeMap<String, Value> {
    keys.iter()
        .filter_map(|key| value.get(*key).cloned().map(|entry| ((*key).to_string(), entry)))
        .collect()
}

pub(crate) fn json_pointer_string(value: &Value, pointer: &str) -> Result<String> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("JSON pointer `{pointer}` did not resolve to a string"))
}

pub(crate) fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

pub(crate) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}
