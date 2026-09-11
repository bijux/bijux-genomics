use super::model::VcfAdapterMissingInputTestsReport;
use super::{
    coverage::ensure_required_role_coverage, VCF_ADAPTER_MISSING_INPUT_TESTS_SCHEMA_VERSION,
};
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
/// Exercise shared required-file admission and publish complete governed role coverage.
///
/// # Errors
/// Rejects adapter acquisition, role coverage, failed probe and publication errors.
pub fn render_vcf_adapter_missing_input_audit(
    inputs: &crate::inputs::VcfAdapterInputs,
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfAdapterMissingInputTestsReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let rows = super::acquisition::collect_vcf_adapter_missing_input_test_rows(inputs, repo_root)?;
    ensure_required_role_coverage(&rows)?;
    let passed_row_count = rows.iter().filter(|row| row.passed).count();
    let failed_row_count = rows.len().saturating_sub(passed_row_count);
    let adapter_row_count =
        rows.iter().filter(|row| row.contract_surface == "adapter_contract").count();
    let support_row_count =
        rows.iter().filter(|row| row.contract_surface != "adapter_contract").count();
    if failed_row_count != 0 {
        let failed_roles = rows
            .iter()
            .filter(|row| !row.passed)
            .map(|row| format!("{}:{}:{}", row.stage_id, row.tool_id, row.missing_input_role))
            .collect::<Vec<_>>();
        return Err(anyhow!(
            "VCF missing-input tests must pass for every governed role, failed rows: {}",
            failed_roles.join(", ")
        ));
    }

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let report = VcfAdapterMissingInputTestsReport {
        schema_version: VCF_ADAPTER_MISSING_INPUT_TESTS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        passed_row_count,
        failed_row_count,
        adapter_row_count,
        support_row_count,
        rows,
    };
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    Ok(report)
}
fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}
fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
