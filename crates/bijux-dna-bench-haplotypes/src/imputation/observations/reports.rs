//! imputation observations reports.
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub(crate) fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

pub(crate) fn read_u64(value: &serde_json::Value, pointer: &str) -> Result<u64> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("missing `{pointer}` in imputation QC payload"))
}
