use super::contract::ensure_vcf_qc_ready_contract;
use super::fields::required_metric_names;
use super::model::VcfQcReadyReport;
use super::projection::build_vcf_qc_ready_row;
use super::{COVERAGE_STATUS_COMPLETE, VCF_QC_READY_SCHEMA_VERSION, VCF_QC_STAGE_ID};
use crate::qc::run_local_vcf_qc_smoke;
use anyhow::{anyhow, Result};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
/// Render admitted stage readiness, preserving an incomplete diagnostic before returning failure.
///
/// # Errors
/// Returns identity, acquisition, publication or incomplete-readiness failures.
pub fn render_vcf_qc_ready(
    inputs: &crate::inputs::VariantBenchmarkInputs,
    snapshot: &bijux_dna_bench_vcf_evidence::snapshot::VcfEvidenceSnapshot,
    command_report: &super::super::evidence::ReadinessCommandPaths,
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfQcReadyReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let report =
        build_vcf_qc_ready_report(inputs, snapshot, command_report, repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "vcf.qc active retained callers must stay complete across active-scope, command, output, parser, report, and normalized qc smoke proof"
        ));
    }
    Ok(report)
}
fn build_vcf_qc_ready_report(
    inputs: &crate::inputs::VariantBenchmarkInputs,
    snapshot: &bijux_dna_bench_vcf_evidence::snapshot::VcfEvidenceSnapshot,
    command_report: &super::super::evidence::ReadinessCommandPaths,
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfQcReadyReport> {
    let bindings =
        super::super::evidence::admit_stage(inputs, snapshot, command_report, VCF_QC_STAGE_ID)?
            .to_vec();
    let active_bindings = bindings
        .into_iter()
        .filter(|binding| binding.retained_row.scope_state == "active")
        .collect::<Vec<_>>();

    let mut rows = Vec::with_capacity(active_bindings.len());
    for binding in active_bindings {
        let smoke_report =
            run_local_vcf_qc_smoke(inputs, repo_root, &binding.retained_row.tool_id).ok();
        rows.push(build_vcf_qc_ready_row(command_report, binding, smoke_report.as_ref()));
    }
    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });

    let complete_row_count =
        rows.iter().filter(|row| row.coverage_status == COVERAGE_STATUS_COMPLETE).count();
    let incomplete_row_count = rows.len().saturating_sub(complete_row_count);
    let active_row_count = rows.iter().filter(|row| row.all_domain_active_row_present).count();
    let mut tool_status_counts = BTreeMap::<String, usize>::new();
    let mut coverage_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *tool_status_counts.entry(row.tool_status.clone()).or_default() += 1;
        *coverage_status_counts.entry(row.coverage_status.clone()).or_default() += 1;
    }
    let violations = rows
        .iter()
        .filter(|row| row.coverage_status != COVERAGE_STATUS_COMPLETE)
        .cloned()
        .collect::<Vec<_>>();

    let report = VcfQcReadyReport {
        schema_version: VCF_QC_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        retained_row_count: rows.len(),
        active_row_count,
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: 8,
        required_metric_names: required_metric_names(),
        tool_status_counts,
        coverage_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    };
    ensure_vcf_qc_ready_contract(&report)?;
    Ok(report)
}
fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}
fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
