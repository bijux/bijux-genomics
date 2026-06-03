use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

const LOCAL_ENDOGENOUS_CONTENT_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.bam.endogenous_content.local_smoke.report.v1";
const LOCAL_ENDOGENOUS_CONTENT_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.endogenous_content.local_smoke.metrics.v1";
const ENDOGENOUS_CONTENT_METHOD: &str = "mapped_fraction_from_flagstat";

#[derive(Debug, Clone, Serialize)]
struct LocalEndogenousContentSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    method: String,
    host_reference_scope: Option<String>,
    mapped_reads: u64,
    endogenous_reads: u64,
    total_reads: u64,
    endogenous_fraction: f64,
    prealignment_fraction: Option<f64>,
    endogenous_report: String,
    endogenous_summary: String,
    stage_metrics: String,
}

/// Materialize the governed local-smoke `bam.endogenous_content` artifacts and top-level report.
///
/// The written report lives at
/// `target/local-smoke/bam.endogenous_content/endogenous_content.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_endogenous_content_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_endogenous_content_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "local-smoke bam.endogenous_content expects exactly one governed case, found {}",
            cases.len()
        ));
    };

    let output_root = repo_root.join("target/local-smoke/bam.endogenous_content");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let report = materialize_local_endogenous_content_smoke_case(&repo_root, case)?;
    let report_path = output_root.join("endogenous_content.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(report_path)
}

/// Write durable typed endogenous-content artifacts beside BAM stage outputs.
///
/// # Errors
/// Returns an error if the flagstat report cannot be parsed or the summary cannot be written.
pub(crate) fn write_stage_endogenous_content_artifacts(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<PathBuf> {
    let input_bam = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.role == bijux_dna_core::contract::ArtifactRole::Bam)
        .map_or_else(|| stage_dir.join("in.bam"), |artifact| artifact.path.clone());
    let host_reference_scope =
        plan.params.get("host_reference_scope").and_then(serde_json::Value::as_str);
    let prealignment_fraction =
        plan.params.get("prealignment_fraction").and_then(serde_json::Value::as_f64);
    let summary = summarize_stage_endogenous_content_outputs(
        stage_dir,
        &input_bam,
        host_reference_scope,
        prealignment_fraction,
    )?;

    let report_path = stage_dir.join("endogenous.content.json");
    let summary_path = stage_dir.join("endogenous.summary.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)
        .with_context(|| format!("write {}", report_path.display()))?;
    bijux_dna_infra::atomic_write_json(&summary_path, &summary)
        .with_context(|| format!("write {}", summary_path.display()))?;
    Ok(summary_path)
}

fn materialize_local_endogenous_content_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalEndogenousContentSmokeCasePlan,
) -> Result<LocalEndogenousContentSmokeReport> {
    let case_out_dir = resolve_plan_path(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let endogenous_report_path = resolve_output_path(repo_root, &case.plan, "endogenous_report")?;
    let endogenous_summary_path = resolve_output_path(repo_root, &case.plan, "summary")?;
    let stage_metrics_path = resolve_output_path(repo_root, &case.plan, "stage_metrics")?;

    let input_bam = repo_root.join(&case.bam);
    let summary = bijux_dna_domain_bam::summarize_tiny_bam_endogenous_content(
        &input_bam,
        ENDOGENOUS_CONTENT_METHOD,
        &case.host_reference_scope,
        None,
    )?;

    bijux_dna_infra::atomic_write_bytes(
        &case_out_dir.join("flagstat.txt"),
        render_flagstat(summary.total_reads, summary.mapped_reads).as_bytes(),
    )?;
    let _summary_path = write_stage_endogenous_content_artifacts(&case_out_dir, &case.plan)?;
    let written_summary: bijux_dna_domain_bam::BamEndogenousContentEstimateV1 =
        serde_json::from_str(
            &std::fs::read_to_string(&endogenous_summary_path)
                .with_context(|| format!("read {}", endogenous_summary_path.display()))?,
        )
        .with_context(|| format!("parse {}", endogenous_summary_path.display()))?;

    let expectation_matched = written_summary.method == case.expected_method
        && written_summary.total_reads == case.expected_total_reads
        && written_summary.mapped_reads == case.expected_mapped_reads
        && written_summary.host_reference_scope.as_deref()
            == Some(case.host_reference_scope.as_str())
        && float_matches(written_summary.endogenous_fraction, case.expected_endogenous_fraction);
    let mapped_read_delta = i64::try_from(written_summary.mapped_reads).unwrap_or(i64::MAX)
        - i64::try_from(case.expected_mapped_reads).unwrap_or(i64::MAX);
    let total_read_delta = i64::try_from(written_summary.total_reads).unwrap_or(i64::MAX)
        - i64::try_from(case.expected_total_reads).unwrap_or(i64::MAX);
    let endogenous_fraction_delta =
        written_summary.endogenous_fraction - case.expected_endogenous_fraction;

    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": LOCAL_ENDOGENOUS_CONTENT_SMOKE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.endogenous_content",
            "sample_id": case.sample_id,
            "expected_method": case.expected_method,
            "method": written_summary.method,
            "expected_host_reference_scope": case.host_reference_scope,
            "host_reference_scope": written_summary.host_reference_scope,
            "expected_mapped_reads": case.expected_mapped_reads,
            "mapped_reads": written_summary.mapped_reads,
            "mapped_read_delta": mapped_read_delta,
            "expected_endogenous_reads": case.expected_mapped_reads,
            "endogenous_reads": written_summary.endogenous_reads,
            "expected_total_reads": case.expected_total_reads,
            "total_reads": written_summary.total_reads,
            "total_read_delta": total_read_delta,
            "expected_endogenous_fraction": case.expected_endogenous_fraction,
            "endogenous_fraction": written_summary.endogenous_fraction,
            "endogenous_fraction_delta": endogenous_fraction_delta,
            "prealignment_fraction": written_summary.prealignment_fraction,
            "expectation_matched": expectation_matched,
        }),
    )?;

    Ok(LocalEndogenousContentSmokeReport {
        schema_version: LOCAL_ENDOGENOUS_CONTENT_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.endogenous_content".to_string(),
        sample_id: case.sample_id.clone(),
        expectation_matched,
        input_bam: path_relative_to_repo(repo_root, &input_bam),
        method: written_summary.method.clone(),
        host_reference_scope: written_summary.host_reference_scope.clone(),
        mapped_reads: written_summary.mapped_reads,
        endogenous_reads: written_summary.endogenous_reads,
        total_reads: written_summary.total_reads,
        endogenous_fraction: written_summary.endogenous_fraction,
        prealignment_fraction: written_summary.prealignment_fraction,
        endogenous_report: path_relative_to_repo(repo_root, &endogenous_report_path),
        endogenous_summary: path_relative_to_repo(repo_root, &endogenous_summary_path),
        stage_metrics: path_relative_to_repo(repo_root, &stage_metrics_path),
    })
}

fn summarize_stage_endogenous_content_outputs(
    stage_dir: &Path,
    _input_bam: &Path,
    host_reference_scope: Option<&str>,
    prealignment_fraction: Option<f64>,
) -> Result<bijux_dna_domain_bam::BamEndogenousContentEstimateV1> {
    let (total_reads, mapped_reads) = parse_flagstat_counts(&stage_dir.join("flagstat.txt"))?;
    Ok(bijux_dna_domain_bam::summarize_bam_endogenous_content(
        "bam.endogenous_content",
        ENDOGENOUS_CONTENT_METHOD,
        total_reads,
        mapped_reads,
        prealignment_fraction,
        host_reference_scope,
    ))
}

fn parse_flagstat_counts(path: &Path) -> Result<(u64, u64)> {
    let body = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut total_reads = None;
    let mut mapped_reads = None;
    for line in body.lines() {
        if total_reads.is_none() && line.contains(" in total ") {
            total_reads =
                line.split_whitespace().next().and_then(|value| value.parse::<u64>().ok());
        }
        if mapped_reads.is_none() && line.contains(" mapped ") {
            mapped_reads =
                line.split_whitespace().next().and_then(|value| value.parse::<u64>().ok());
        }
    }
    let total_reads = total_reads
        .ok_or_else(|| anyhow!("flagstat report missing total read count: {}", path.display()))?;
    let mapped_reads = mapped_reads
        .ok_or_else(|| anyhow!("flagstat report missing mapped read count: {}", path.display()))?;
    Ok((total_reads, mapped_reads))
}

fn render_flagstat(total_reads: u64, mapped_reads: u64) -> String {
    format!(
        "{total_reads} + 0 in total (QC-passed reads + QC-failed reads)\n{mapped_reads} + 0 mapped ({fraction:.2}% : N/A)\n",
        fraction = if total_reads > 0 {
            mapped_reads as f64 * 100.0 / total_reads as f64
        } else {
            0.0
        },
    )
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
            anyhow!(
                "bam.endogenous_content local-smoke plan is missing governed output `{output_id}`"
            )
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
