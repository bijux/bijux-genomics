//! postprocess observations reports.
use anyhow::Context;
use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub(crate) fn read_json_value(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}
