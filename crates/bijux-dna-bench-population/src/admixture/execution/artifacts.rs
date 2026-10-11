//! Owner: bijux-dna-bench-population
//! Admixture execution artifacts.
use anyhow::Context;
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub(crate) fn resolve_manifest_relative_path(manifest_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest_dir.join(path)
    }
}

pub(crate) fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}
