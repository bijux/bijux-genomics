//! Owner: bijux-dna-bench
//! Optional observed FASTQ score report IO.

use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub(crate) fn load_optional_fastq_score_report(path: &Path) -> Result<Option<Value>> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).with_context(|| format!("read {}", path.display())),
    };
    let value =
        serde_json::from_str::<Value>(&raw).with_context(|| format!("parse {}", path.display()))?;
    if matches!(value, Value::Object(ref map) if map.is_empty()) {
        return Ok(None);
    }
    Ok(Some(value))
}
