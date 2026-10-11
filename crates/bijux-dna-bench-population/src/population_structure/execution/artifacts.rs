//! Owner: bijux-dna-bench-population
//! Population structure execution artifacts.
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub(crate) fn ensure_file_exists(path: &Path, label: &str) -> Result<()> {
    if !path.is_file() {
        bail!("{label} is missing: {}", path.display());
    }
    Ok(())
}

pub(crate) fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}
