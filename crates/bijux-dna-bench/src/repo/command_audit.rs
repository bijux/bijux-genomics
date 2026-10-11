//! Owner: recorded command diagnostic report publication.
use anyhow::{Context, Result};
use bijux_dna_bench_model::AllDomainNoPlaceholderCommandCheckReport;
use std::path::{Path, PathBuf};

pub(crate) fn output_path(repo_root: &Path, requested: &Path) -> PathBuf {
    if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        repo_root.join(requested)
    }
}
pub(crate) fn display_path(repo_root: &Path, output: &Path) -> String {
    output.strip_prefix(repo_root).unwrap_or(output).to_string_lossy().replace('\\', "/")
}
pub(crate) fn publish(
    path: &Path,
    report: &AllDomainNoPlaceholderCommandCheckReport,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let payload = serde_json::to_vec_pretty(report).context("serialize recorded command audit")?;
    bijux_dna_infra::atomic_write_bytes(path, &payload)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}
