use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

const LOCAL_SEX_SMOKE_REPORT_SCHEMA_VERSION: &str = "bijux.bam.sex.local_smoke.report.v1";
const LOCAL_SEX_SMOKE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.sex.local_smoke.metrics.v1";
const SEX_TOOL_REPORT_SCHEMA_VERSION: &str = "bijux.bam.sex.v1";

#[derive(Debug, Clone, Serialize)]
struct LocalSexSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    reference_fasta: String,
    method: String,
    chromosome_system: Option<String>,
    minimum_y_sites: Option<u32>,
    x_coverage: f64,
    y_coverage: f64,
    autosomal_coverage: f64,
    x_to_y_ratio: Option<f64>,
    call: bijux_dna_domain_bam::metrics::SexConfidenceClass,
    confidence: f64,
    status: String,
    insufficiency_reason: Option<String>,
    sex_report: String,
    sex_summary: String,
    stage_metrics: String,
}

/// Materialize the governed local-smoke `bam.sex` artifacts and top-level report.
///
/// The written report lives at `target/local-smoke/bam.sex/sex.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_sex_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_sex_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "local-smoke bam.sex expects exactly one governed case, found {}",
            cases.len()
        ));
    };

    let output_root = repo_root.join("target/local-smoke/bam.sex");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let report = materialize_local_sex_smoke_case(&repo_root, case)?;
    let report_path = output_root.join("sex.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(report_path)
}

/// Write durable `bam.sex` report and summary artifacts beside stage outputs.
///
/// # Errors
/// Returns an error if the BAM fixture cannot be summarized or artifacts cannot be written.
pub(crate) fn write_stage_sex_artifacts(
    stage_dir: &Path,
    input_bam: &Path,
    reference_fasta: &Path,
    method: &str,
    chromosome_system: Option<&str>,
    minimum_y_sites: Option<u32>,
) -> Result<bijux_dna_domain_bam::BamSexSummaryV1> {
    let summary = bijux_dna_domain_bam::summarize_tiny_bam_sex(
        input_bam,
        reference_fasta,
        method,
        chromosome_system,
        minimum_y_sites,
    )?;
    let report_path = stage_dir.join("sex.json");
    let summary_path = stage_dir.join("sex.summary.json");
    bijux_dna_infra::atomic_write_json(&report_path, &sex_tool_report(&summary))
        .with_context(|| format!("write {}", report_path.display()))?;
    bijux_dna_infra::atomic_write_json(&summary_path, &summary)
        .with_context(|| format!("write {}", summary_path.display()))?;
    Ok(summary)
}

fn materialize_local_sex_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalSexSmokeCasePlan,
) -> Result<LocalSexSmokeReport> {
    let case_out_dir = resolve_plan_path(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let sex_report_path = resolve_output_path(repo_root, &case.plan, "sex_report")?;
    let sex_summary_path = resolve_output_path(repo_root, &case.plan, "summary")?;
    let stage_metrics_path = resolve_output_path(repo_root, &case.plan, "stage_metrics")?;
    let input_bam = repo_root.join(&case.bam);
    let reference_fasta = repo_root.join(&case.reference);

    let summary = write_stage_sex_artifacts(
        &case_out_dir,
        &input_bam,
        &reference_fasta,
        &case.expected_method,
        Some(case.chromosome_system.as_str()),
        Some(case.minimum_y_sites),
    )?;
    let expectation_matched = summary.method == case.expected_method
        && summary.chromosome_system.as_deref() == Some(case.chromosome_system.as_str())
        && summary.minimum_y_sites == Some(case.minimum_y_sites)
        && float_matches(summary.x_coverage, case.expected_x_coverage)
        && float_matches(summary.y_coverage, case.expected_y_coverage)
        && float_matches(summary.autosomal_coverage, case.expected_autosomal_coverage)
        && summary.call == case.expected_call
        && float_matches(summary.confidence, case.expected_confidence)
        && summary.status == case.expected_status;
    let x_coverage_delta = summary.x_coverage - case.expected_x_coverage;
    let y_coverage_delta = summary.y_coverage - case.expected_y_coverage;
    let autosomal_coverage_delta =
        summary.autosomal_coverage - case.expected_autosomal_coverage;
    let confidence_delta = summary.confidence - case.expected_confidence;

    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": LOCAL_SEX_SMOKE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.sex",
            "sample_id": case.sample_id,
            "expected_method": case.expected_method,
            "method": summary.method,
            "expected_chromosome_system": case.chromosome_system,
            "chromosome_system": summary.chromosome_system,
            "expected_minimum_y_sites": case.minimum_y_sites,
            "minimum_y_sites": summary.minimum_y_sites,
            "expected_x_coverage": case.expected_x_coverage,
            "x_coverage": summary.x_coverage,
            "x_coverage_delta": x_coverage_delta,
            "expected_y_coverage": case.expected_y_coverage,
            "y_coverage": summary.y_coverage,
            "y_coverage_delta": y_coverage_delta,
            "expected_autosomal_coverage": case.expected_autosomal_coverage,
            "autosomal_coverage": summary.autosomal_coverage,
            "autosomal_coverage_delta": autosomal_coverage_delta,
            "x_to_y_ratio": summary.x_to_y_ratio,
            "expected_call": case.expected_call,
            "call": summary.call,
            "expected_confidence": case.expected_confidence,
            "confidence": summary.confidence,
            "confidence_delta": confidence_delta,
            "expected_status": case.expected_status,
            "status": summary.status,
            "insufficiency_reason": summary.insufficiency_reason,
            "expectation_matched": expectation_matched,
        }),
    )?;

    Ok(LocalSexSmokeReport {
        schema_version: LOCAL_SEX_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.sex".to_string(),
        sample_id: case.sample_id.clone(),
        expectation_matched,
        input_bam: path_relative_to_repo(repo_root, &input_bam),
        reference_fasta: path_relative_to_repo(repo_root, &reference_fasta),
        method: summary.method.clone(),
        chromosome_system: summary.chromosome_system.clone(),
        minimum_y_sites: summary.minimum_y_sites,
        x_coverage: summary.x_coverage,
        y_coverage: summary.y_coverage,
        autosomal_coverage: summary.autosomal_coverage,
        x_to_y_ratio: summary.x_to_y_ratio,
        call: summary.call,
        confidence: summary.confidence,
        status: summary.status.clone(),
        insufficiency_reason: summary.insufficiency_reason.clone(),
        sex_report: path_relative_to_repo(repo_root, &sex_report_path),
        sex_summary: path_relative_to_repo(repo_root, &sex_summary_path),
        stage_metrics: path_relative_to_repo(repo_root, &stage_metrics_path),
    })
}

fn sex_tool_report(summary: &bijux_dna_domain_bam::BamSexSummaryV1) -> serde_json::Value {
    serde_json::json!({
        "schema_version": SEX_TOOL_REPORT_SCHEMA_VERSION,
        "method": summary.method,
        "chromosome_system": summary.chromosome_system,
        "minimum_y_sites": summary.minimum_y_sites,
        "x_coverage": summary.x_coverage,
        "y_coverage": summary.y_coverage,
        "autosomal_coverage": summary.autosomal_coverage,
        "x_to_y_ratio": summary.x_to_y_ratio,
        "classification": summary.call,
        "call": summary.call,
        "confidence": summary.confidence,
        "status": summary.status,
        "insufficiency_reason": summary.insufficiency_reason,
    })
}

fn float_matches(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}

fn resolve_output_path(
    repo_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    output_id: &str,
) -> Result<PathBuf> {
    let path = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| {
            anyhow!("bam.sex local-smoke plan is missing governed output `{output_id}`")
        })?;
    Ok(resolve_plan_path(repo_root, &path))
}

fn resolve_plan_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    relative_path(repo_root, path).display().to_string()
}
