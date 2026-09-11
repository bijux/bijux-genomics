use super::model::VcfMatrixRegistryConsistencyReport;
use crate::inputs::VcfAdapterInputs;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
/// Publish the audit from admitted catalog bindings and registry observations.
///
/// # Errors
/// Rejects malformed declarations, audit contract failures and publication errors.
pub fn render_vcf_matrix_registry_consistency(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfMatrixRegistryConsistencyReport> {
    let registry = super::super::load_vcf_registry_snapshot(repo_root)?;
    let output_path = repo_relative_path(repo_root, output_path);
    let report = super::projection::build_report(inputs, &registry, repo_root, &output_path)?;
    let payload = serde_json::to_string_pretty(&report)
        .context("render VCF matrix/registry consistency gate to JSON")?;
    bijux_dna_infra::atomic_write_bytes(&output_path, payload.as_bytes())?;
    Ok(report)
}
fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}
