//! imputation metrics observations reports.
use anyhow::Context;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub(crate) fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}
