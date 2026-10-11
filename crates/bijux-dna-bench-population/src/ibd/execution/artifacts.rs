//! Owner: bijux-dna-bench-population
//! Ibd execution artifacts.
use anyhow::Context;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub(crate) fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

pub(crate) fn resolve_manifest_relative_path(
    manifest_dir: &Path,
    path: &Path,
) -> std::path::PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest_dir.join(path)
    }
}
