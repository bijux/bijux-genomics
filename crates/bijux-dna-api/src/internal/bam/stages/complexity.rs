use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

const LOCAL_COMPLEXITY_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.bam.complexity.local_smoke.report.v1";
const LOCAL_COMPLEXITY_SMOKE_OBSERVATION_SCHEMA_VERSION: &str =
    "bijux.bam.complexity.local_smoke.observation.v1";
const LOCAL_COMPLEXITY_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.complexity.local_smoke.metrics.v1";
const INSUFFICIENT_COMPLEXITY_REASON: &str =
    "insufficient_observed_unique_reads_for_complexity_extrapolation";
const MISSING_PRESEQ_POINTS_REASON: &str = "missing_preseq_projection_points";

#[derive(Debug, Clone, Serialize)]
struct LocalComplexitySmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    method: String,
    observed_total_reads: u64,
    observed_unique_reads: u64,
    estimated_unique_reads: Option<u64>,
    estimated_library_size: Option<u64>,
    saturation_estimate: Option<f64>,
    insufficient_data_reason: Option<String>,
    complexity_report: String,
    complexity_curve: String,
    complexity_summary: String,
    stage_metrics: String,
}

#[derive(Debug, Clone, Serialize)]
struct LocalComplexityObservation {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    method: String,
    observed_total_reads: u64,
    observed_unique_reads: u64,
    projected_unique_reads: Vec<(u64, u64)>,
    estimated_unique_reads: Option<u64>,
    estimated_library_size: Option<u64>,
    saturation_estimate: Option<f64>,
    min_reads: u64,
    insufficient_data_reason: Option<String>,
}

/// Materialize the governed local-smoke `bam.complexity` artifacts and top-level report.
///
/// The written report lives at `target/local-smoke/bam.complexity/complexity.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_complexity_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_complexity_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "local-smoke bam.complexity expects exactly one governed case, found {}",
            cases.len()
        ));
    };

    let output_root = repo_root.join("target/local-smoke/bam.complexity");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let report = materialize_local_complexity_smoke_case(&repo_root, case)?;
    let report_path = output_root.join("complexity.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(report_path)
}

/// Write a durable typed `complexity.summary.json` artifact beside BAM complexity stage outputs.
///
/// # Errors
/// Returns an error if the stage artifacts cannot be parsed or the summary cannot be written.
pub(crate) fn write_stage_complexity_summary(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<PathBuf> {
    let input_bam = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.role == bijux_dna_core::contract::ArtifactRole::Bam)
        .map_or_else(|| stage_dir.join("in.bam"), |artifact| artifact.path.clone());
    let summary = summarize_stage_complexity_outputs(stage_dir, plan, &input_bam)?;
    let path = stage_dir.join("complexity.summary.json");
    bijux_dna_infra::atomic_write_json(&path, &summary)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

fn materialize_local_complexity_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalComplexitySmokeCasePlan,
) -> Result<LocalComplexitySmokeReport> {
    let case_out_dir = resolve_plan_path(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let complexity_report_path = resolve_output_path(repo_root, &case.plan, "complexity_report")?;
    let complexity_curve_path = resolve_output_path(repo_root, &case.plan, "complexity_curve")?;
    let complexity_summary_path = resolve_output_path(repo_root, &case.plan, "summary")?;
    let stage_metrics_path = resolve_output_path(repo_root, &case.plan, "stage_metrics")?;

    let input_bam = repo_root.join(&case.bam);
    let mut summary = bijux_dna_domain_bam::summarize_tiny_bam_complexity(
        &input_bam,
        case.plan.tool_id.as_str(),
        case.min_reads,
        &case.projection_points,
    )?;
    summary.input_bam = relative_path(repo_root, &summary.input_bam);

    let observation = LocalComplexityObservation {
        schema_version: LOCAL_COMPLEXITY_SMOKE_OBSERVATION_SCHEMA_VERSION.to_string(),
        stage_id: "bam.complexity".to_string(),
        sample_id: case.sample_id.clone(),
        method: summary.method.clone(),
        observed_total_reads: summary.observed_total_reads,
        observed_unique_reads: summary.observed_unique_reads,
        projected_unique_reads: summary.projected_unique_reads.clone(),
        estimated_unique_reads: summary.estimated_unique_reads,
        estimated_library_size: summary.estimated_library_size,
        saturation_estimate: summary.saturation_estimate,
        min_reads: summary.min_reads,
        insufficient_data_reason: summary.insufficient_data_reason.clone(),
    };

    bijux_dna_infra::atomic_write_json(&complexity_report_path, &observation)?;
    bijux_dna_infra::atomic_write_bytes(
        &complexity_curve_path,
        render_complexity_curve(&summary).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_json(&complexity_summary_path, &summary)?;
    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": LOCAL_COMPLEXITY_SMOKE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.complexity",
            "sample_id": case.sample_id,
            "method": summary.method,
            "observed_total_reads": summary.observed_total_reads,
            "observed_unique_reads": summary.observed_unique_reads,
            "estimated_unique_reads": summary.estimated_unique_reads,
            "estimated_library_size": summary.estimated_library_size,
            "saturation_estimate": summary.saturation_estimate,
            "min_reads": summary.min_reads,
            "insufficient_data_reason": summary.insufficient_data_reason,
        }),
    )?;

    let expectation_matched = summary.observed_total_reads == case.expected_observed_total_reads
        && summary.observed_unique_reads == case.expected_observed_unique_reads
        && summary.estimated_unique_reads == case.expected_estimated_unique_reads
        && summary.insufficient_data_reason == case.expected_insufficient_data_reason;

    Ok(LocalComplexitySmokeReport {
        schema_version: LOCAL_COMPLEXITY_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.complexity".to_string(),
        sample_id: case.sample_id.clone(),
        expectation_matched,
        input_bam: path_relative_to_repo(repo_root, &input_bam),
        method: summary.method.clone(),
        observed_total_reads: summary.observed_total_reads,
        observed_unique_reads: summary.observed_unique_reads,
        estimated_unique_reads: summary.estimated_unique_reads,
        estimated_library_size: summary.estimated_library_size,
        saturation_estimate: summary.saturation_estimate,
        insufficient_data_reason: summary.insufficient_data_reason.clone(),
        complexity_report: path_relative_to_repo(repo_root, &complexity_report_path),
        complexity_curve: path_relative_to_repo(repo_root, &complexity_curve_path),
        complexity_summary: path_relative_to_repo(repo_root, &complexity_summary_path),
        stage_metrics: path_relative_to_repo(repo_root, &stage_metrics_path),
    })
}

fn summarize_stage_complexity_outputs(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    input_bam: &Path,
) -> Result<bijux_dna_domain_bam::BamComplexitySummaryV1> {
    let complexity_curve_path = stage_dir.join("complexity_curve.tsv");
    let complexity = bijux_dna_domain_bam::metrics::parse_preseq_estimates(&complexity_curve_path)?;
    let observed_total_reads =
        complexity.projected_reads.first().map(|(reads, _)| *reads).unwrap_or(0);
    let min_reads = plan.params.get("min_reads").and_then(serde_json::Value::as_u64).unwrap_or(0);
    let insufficient_data_reason = if complexity.projected_reads.is_empty() {
        Some(MISSING_PRESEQ_POINTS_REASON)
    } else if complexity.observed_reads < min_reads {
        Some(INSUFFICIENT_COMPLEXITY_REASON)
    } else {
        None
    };

    Ok(bijux_dna_domain_bam::summarize_bam_complexity(
        "bam.complexity",
        plan.tool_id.as_str(),
        input_bam,
        observed_total_reads,
        &complexity,
        min_reads,
        insufficient_data_reason,
    ))
}

fn render_complexity_curve(summary: &bijux_dna_domain_bam::BamComplexitySummaryV1) -> String {
    let mut rendered = String::new();
    for (reads, projected_unique_reads) in &summary.projected_unique_reads {
        use std::fmt::Write as _;
        let _ = writeln!(rendered, "{reads}\t{projected_unique_reads}");
    }
    rendered
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
            anyhow!("bam.complexity local-smoke plan is missing governed output `{output_id}`")
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
