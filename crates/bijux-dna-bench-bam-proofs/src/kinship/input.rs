use crate::downstream::observations::ParserObservation;
use anyhow::anyhow;
use anyhow::Result;
use std::path::Path;
#[cfg(feature = "bam_downstream")]
pub(super) fn read_json(path: &Path) -> Result<serde_json::Value> {
    crate::input::read_json(path)
}

pub(super) fn required_string(payload: &serde_json::Value, key: &str) -> Result<String> {
    payload
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("missing string field `{key}`"))
}

pub(super) fn required_u64(payload: &serde_json::Value, key: &str) -> Result<u64> {
    payload
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("missing integer field `{key}`"))
}

pub(super) fn required_bool(payload: &serde_json::Value, key: &str) -> Result<bool> {
    payload
        .get(key)
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("missing boolean field `{key}`"))
}

pub(super) fn optional_string(payload: &serde_json::Value, key: &str) -> Result<Option<String>> {
    let Some(value) = payload.get(key) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    value
        .as_str()
        .map(|item| Some(item.to_string()))
        .ok_or_else(|| anyhow!("field `{key}` must be string or null"))
}

pub(super) fn required_pair_array_len(payload: &serde_json::Value) -> Result<usize> {
    payload
        .get("pairwise_results")
        .and_then(serde_json::Value::as_array)
        .map(std::vec::Vec::len)
        .ok_or_else(|| anyhow!("missing array field `pairwise_results`"))
}

pub(super) fn required_pair_field(
    payload: &serde_json::Value,
    index: usize,
    key: &str,
) -> Result<String> {
    payload
        .get("pairwise_results")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.get(index))
        .and_then(|item| item.get(key))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("missing string pairwise field `{key}` at index {index}"))
}

pub(super) fn required_pair_u64(
    payload: &serde_json::Value,
    index: usize,
    key: &str,
) -> Result<u64> {
    payload
        .get("pairwise_results")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.get(index))
        .and_then(|item| item.get(key))
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("missing integer pairwise field `{key}` at index {index}"))
}

pub(super) fn required_pair_f64(
    payload: &serde_json::Value,
    index: usize,
    key: &str,
) -> Result<f64> {
    payload
        .get("pairwise_results")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.get(index))
        .and_then(|item| item.get(key))
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("missing float pairwise field `{key}` at index {index}"))
}

#[cfg(feature = "bam_downstream")]
pub(super) fn required_snapshot_string(row: &ParserObservation, key: &str) -> Result<String> {
    row.normalized_snapshot
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("missing parser-smoke snapshot string `{key}`"))
}

#[cfg(feature = "bam_downstream")]
pub(super) fn required_snapshot_u64(row: &ParserObservation, key: &str) -> Result<u64> {
    row.normalized_snapshot
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("missing parser-smoke snapshot integer `{key}`"))
}
