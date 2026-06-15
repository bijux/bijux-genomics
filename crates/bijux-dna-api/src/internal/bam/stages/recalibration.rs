use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

const LOCAL_RECALIBRATION_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.bam.recalibration.local_smoke.report.v1";
const LOCAL_RECALIBRATION_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.recalibration.local_smoke.metrics.v1";

#[derive(Debug, Clone, Serialize)]
struct LocalRecalibrationSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    reference_fasta: String,
    known_sites: Vec<String>,
    requested_mode: bijux_dna_domain_bam::params::BqsrMode,
    effective_mode: bijux_dna_domain_bam::params::BqsrMode,
    status: String,
    reason: String,
    coverage_gate: bijux_dna_domain_bam::BamRecalibrationCoverageGateV1,
    observed_mean_coverage: f64,
    observed_breadth_1x: f64,
    output_bam_present: bool,
    recalibration_report_present: bool,
    recalibrated_bam: String,
    recalibration_report: String,
    recalibration_summary: String,
    stage_metrics: String,
}

/// Materialize the governed local-smoke `bam.recalibration` artifacts and top-level report.
///
/// The written report lives at `runs/bench/local-smoke/bam.recalibration/recalibration.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_recalibration_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_recalibration_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "local-smoke bam.recalibration expects exactly one governed case, found {}",
            cases.len()
        ));
    };

    let output_root = repo_root.join("runs/bench/local-smoke/bam.recalibration");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let report = materialize_local_recalibration_smoke_case(&repo_root, case)?;
    let report_path = output_root.join("recalibration.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(report_path)
}

/// Write a durable typed `recal.summary.json` artifact beside recalibration outputs.
///
/// # Errors
/// Returns an error if the plan parameters cannot be decoded, the tiny BAM fixture cannot be
/// summarized, or the summary cannot be written.
pub(crate) fn write_stage_recalibration_summary(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let input_bam = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.role == bijux_dna_core::contract::ArtifactRole::Bam)
        .map(|artifact| resolve_plan_path(&repo_root, &artifact.path))
        .ok_or_else(|| anyhow!("bam.recalibration stage summary requires a BAM input"))?;
    let reference_fasta = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.role == bijux_dna_core::contract::ArtifactRole::Reference)
        .map(|artifact| resolve_plan_path(&repo_root, &artifact.path));
    let output_bam = resolve_output_path(&repo_root, plan, "recal_bam")?;
    let recalibration_report = resolve_output_path(&repo_root, plan, "recal_report")?;
    let summary_path = match resolve_output_path(&repo_root, plan, "summary") {
        Ok(path) => path,
        Err(_) => stage_dir.join("recal.summary.json"),
    };
    let effective_params: bijux_dna_domain_bam::params::BqsrEffectiveParams =
        serde_json::from_value(plan.effective_params.clone())
            .context("decode bam.recalibration effective params")?;
    let requested_mode = plan
        .params
        .get("requested_mode")
        .cloned()
        .or_else(|| plan.params.get("mode").cloned())
        .ok_or_else(|| anyhow!("bam.recalibration stage summary requires requested mode"))?;
    let requested_mode: bijux_dna_domain_bam::params::BqsrMode =
        serde_json::from_value(requested_mode).context("decode recalibration requested mode")?;
    let known_sites = effective_params
        .known_sites
        .iter()
        .map(PathBuf::from)
        .map(|path| resolve_plan_path(&repo_root, &path))
        .collect::<Vec<_>>();
    let summary = bijux_dna_domain_bam::summarize_tiny_bam_recalibration(
        &input_bam,
        reference_fasta.as_deref(),
        &known_sites,
        requested_mode,
        effective_params.mode,
        &effective_params.skip_criteria,
        output_bam.exists(),
        recalibration_report.exists(),
    )?;
    bijux_dna_infra::atomic_write_json(&summary_path, &summary)
        .with_context(|| format!("write {}", summary_path.display()))?;
    Ok(summary_path)
}

fn materialize_local_recalibration_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalRecalibrationSmokeCasePlan,
) -> Result<LocalRecalibrationSmokeReport> {
    let case_out_dir = resolve_plan_path(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let recalibrated_bam_path = resolve_output_path(repo_root, &case.plan, "recal_bam")?;
    let recalibrated_bai_path = resolve_output_path(repo_root, &case.plan, "recal_bai")?;
    let recalibration_report_path = resolve_output_path(repo_root, &case.plan, "recal_report")?;
    let recalibration_summary_path = resolve_output_path(repo_root, &case.plan, "summary")?;
    let stage_metrics_path = resolve_output_path(repo_root, &case.plan, "stage_metrics")?;

    let input_bam = repo_root.join(&case.bam);
    let reference_fasta = repo_root.join(&case.reference);
    bijux_dna_infra::atomic_write_bytes(&recalibrated_bam_path, &std::fs::read(&input_bam)?)?;
    bijux_dna_infra::atomic_write_bytes(&recalibrated_bai_path, b"tiny-index\n")?;
    bijux_dna_infra::atomic_write_bytes(
        &recalibration_report_path,
        render_local_recalibration_report(case).as_bytes(),
    )?;
    let _summary_path = write_stage_recalibration_summary(&case_out_dir, &case.plan)?;
    let summary: bijux_dna_domain_bam::BamRecalibrationSummaryV1 = serde_json::from_str(
        &std::fs::read_to_string(&recalibration_summary_path)
            .with_context(|| format!("read {}", recalibration_summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", recalibration_summary_path.display()))?;
    let expectation_matched = summary.status == case.expected_status
        && summary.reason == case.expected_reason
        && summary.requested_mode == case.requested_mode
        && summary.effective_mode == case.effective_mode;
    let mean_coverage_margin = summary.observed_mean_coverage - case.min_mean_coverage;
    let breadth_1x_margin = summary.observed_breadth_1x - case.min_breadth_1x;

    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": LOCAL_RECALIBRATION_SMOKE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.recalibration",
            "sample_id": case.sample_id,
            "expected_requested_mode": case.requested_mode,
            "requested_mode": summary.requested_mode,
            "expected_effective_mode": case.effective_mode,
            "effective_mode": summary.effective_mode,
            "expected_status": case.expected_status,
            "status": summary.status,
            "expected_reason": case.expected_reason,
            "reason": summary.reason,
            "expected_known_sites": case
                .known_sites
                .iter()
                .map(|path| path_relative_to_repo(repo_root, &repo_root.join(path)))
                .collect::<Vec<_>>(),
            "known_sites": summary
                .known_sites
                .iter()
                .map(|path| path_relative_to_repo(repo_root, path))
                .collect::<Vec<_>>(),
            "expected_coverage_gate": {
                "min_mean_coverage": case.min_mean_coverage,
                "min_breadth_1x": case.min_breadth_1x,
            },
            "coverage_gate": summary.coverage_gate,
            "observed_mean_coverage": summary.observed_mean_coverage,
            "mean_coverage_margin": mean_coverage_margin,
            "observed_breadth_1x": summary.observed_breadth_1x,
            "breadth_1x_margin": breadth_1x_margin,
            "output_bam_present": summary.output_bam_present,
            "recalibration_report_present": summary.recalibration_report_present,
            "expectation_matched": expectation_matched,
        }),
    )?;

    Ok(LocalRecalibrationSmokeReport {
        schema_version: LOCAL_RECALIBRATION_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.recalibration".to_string(),
        sample_id: case.sample_id.clone(),
        expectation_matched,
        input_bam: path_relative_to_repo(repo_root, &input_bam),
        reference_fasta: path_relative_to_repo(repo_root, &reference_fasta),
        known_sites: summary
            .known_sites
            .iter()
            .map(|path| path_relative_to_repo(repo_root, path))
            .collect(),
        requested_mode: summary.requested_mode,
        effective_mode: summary.effective_mode,
        status: summary.status.clone(),
        reason: summary.reason.clone(),
        coverage_gate: summary.coverage_gate,
        observed_mean_coverage: summary.observed_mean_coverage,
        observed_breadth_1x: summary.observed_breadth_1x,
        output_bam_present: summary.output_bam_present,
        recalibration_report_present: summary.recalibration_report_present,
        recalibrated_bam: path_relative_to_repo(repo_root, &recalibrated_bam_path),
        recalibration_report: path_relative_to_repo(repo_root, &recalibration_report_path),
        recalibration_summary: path_relative_to_repo(repo_root, &recalibration_summary_path),
        stage_metrics: path_relative_to_repo(repo_root, &stage_metrics_path),
    })
}

fn render_local_recalibration_report(
    case: &bijux_dna_planner_bam::stage_api::LocalRecalibrationSmokeCasePlan,
) -> String {
    format!(
        "status={status}\nreason={reason}\nrequested_mode={requested_mode}\neffective_mode={effective_mode}\nknown_sites={known_sites}\ncoverage_gate.min_mean_coverage={min_mean_coverage}\ncoverage_gate.min_breadth_1x={min_breadth_1x}\nobserved_mean_coverage={observed_mean_coverage}\nobserved_breadth_1x={observed_breadth_1x}\n",
        status = case.expected_status,
        reason = case.expected_reason,
        requested_mode = mode_name(case.requested_mode),
        effective_mode = mode_name(case.effective_mode),
        known_sites = case
            .known_sites
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(","),
        min_mean_coverage = case.min_mean_coverage,
        min_breadth_1x = case.min_breadth_1x,
        observed_mean_coverage = case.observed_mean_coverage,
        observed_breadth_1x = case.observed_breadth_1x,
    )
}

fn mode_name(mode: bijux_dna_domain_bam::params::BqsrMode) -> &'static str {
    match mode {
        bijux_dna_domain_bam::params::BqsrMode::Standard => "standard",
        bijux_dna_domain_bam::params::BqsrMode::Skip => "skip",
        bijux_dna_domain_bam::params::BqsrMode::EmitOnly => "emit_only",
    }
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
            anyhow!("bam.recalibration local-smoke plan is missing governed output `{output_id}`")
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
