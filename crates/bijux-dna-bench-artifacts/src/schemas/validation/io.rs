use anyhow::{Context, Result};
use serde::Serialize;
use std::{fs, path::Path};
pub(super) fn read_json_value(path: &Path) -> Result<serde_json::Value> {
    serde_json::from_slice(&fs::read(path).with_context(|| format!("read {}", path.display()))?)
        .with_context(|| format!("parse {}", path.display()))
}
pub(super) fn write_validation_report<T: Serialize>(
    repo_root: &Path,
    output_path: &str,
    report: &T,
) -> Result<()> {
    let absolute_output_path = repo_root.join(output_path);
    if let Some(parent) = absolute_output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&absolute_output_path, report)
        .with_context(|| format!("write {}", absolute_output_path.display()))?;
    Ok(())
}
