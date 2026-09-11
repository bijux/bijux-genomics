use super::model::VcfParserFailureTestsReport;
use super::projection::collect_vcf_parser_failure_rows;
use super::VCF_PARSER_FAILURE_TESTS_SCHEMA_VERSION;
use anyhow::Context;
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
/// Execute governed malformed-output probes and publish their checked report.
///
/// # Errors
/// Rejects unavailable or unsafe fixtures, unexpected parser acceptance and publication failures.
pub fn render_vcf_parser_failure_audit(
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfParserFailureTestsReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let rows = collect_vcf_parser_failure_rows(repo_root)?;
    let passed_row_count = rows.iter().filter(|row| row.passed).count();
    let failed_row_count = rows.len().saturating_sub(passed_row_count);
    let mut failure_reason_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *failure_reason_counts.entry(row.failure_reason.clone()).or_default() += 1;
    }
    if failed_row_count != 0 {
        let failed_cases = rows
            .iter()
            .filter(|row| !row.passed)
            .map(|row| format!("{}:{}:{}", row.stage_id, row.tool_id, row.failure_reason))
            .collect::<Vec<_>>();
        return Err(anyhow!(
            "VCF parser failure tests must pass for every governed case, failed rows: {}",
            failed_cases.join(", ")
        ));
    }

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let report = VcfParserFailureTestsReport {
        schema_version: VCF_PARSER_FAILURE_TESTS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        passed_row_count,
        failed_row_count,
        failure_reason_counts,
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
