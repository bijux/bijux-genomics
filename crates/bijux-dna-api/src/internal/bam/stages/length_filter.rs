use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::Serialize;

const LOCAL_LENGTH_FILTER_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.bam.length_filter.local_smoke.report.v1";
const LOCAL_LENGTH_FILTER_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.length_filter.local_smoke.metrics.v1";

#[derive(Debug, Clone, Serialize)]
struct LocalLengthFilterSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    filtered_bam: String,
    filtered_bai: String,
    min_length_threshold: u32,
    input_reads: u64,
    kept_reads: u64,
    removed_reads: u64,
    observed_min_length: Option<u32>,
    observed_max_length: Option<u32>,
    length_filter_summary: String,
    flagstat_before: String,
    flagstat_after: String,
    idxstats_before: String,
    idxstats_after: String,
    stage_metrics: String,
}

/// Materialize the governed local-smoke `bam.length_filter` artifacts and top-level report.
///
/// The written report lives at `runs/bench/local-smoke/bam.length_filter/length_filter.json`
/// under the active repository root, alongside the curated top-level `length_filtered.bam`.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_length_filter_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_length_filter_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "local-smoke bam.length_filter expects exactly one governed case, found {}",
            cases.len()
        ));
    };

    let output_root = repo_root.join("runs/bench/local-smoke/bam.length_filter");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let report = materialize_local_length_filter_smoke_case(&repo_root, case, &output_root)?;
    let report_path = output_root.join("length_filter.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(report_path)
}

fn materialize_local_length_filter_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalLengthFilterSmokeCasePlan,
    output_root: &Path,
) -> Result<LocalLengthFilterSmokeReport> {
    let case_out_dir = resolve_plan_path(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let filtered_bam_path = resolve_output_path(repo_root, &case.plan, "filtered_bam")?;
    let filtered_bai_path = resolve_output_path(repo_root, &case.plan, "filtered_bai")?;
    let flagstat_before_path = resolve_output_path(repo_root, &case.plan, "flagstat_before")?;
    let flagstat_after_path = resolve_output_path(repo_root, &case.plan, "flagstat_after")?;
    let idxstats_before_path = resolve_output_path(repo_root, &case.plan, "idxstats_before")?;
    let idxstats_after_path = resolve_output_path(repo_root, &case.plan, "idxstats_after")?;
    let summary_path = resolve_output_path(repo_root, &case.plan, "summary")?;
    let stage_metrics_path = resolve_output_path(repo_root, &case.plan, "stage_metrics")?;

    let input_bam = repo_root.join(&case.bam);
    let mut summary = bijux_dna_domain_bam::filter_tiny_bam_by_length(
        &input_bam,
        &filtered_bam_path,
        case.min_length,
    )?;
    summary.input_bam = relative_path(repo_root, &summary.input_bam);
    summary.output_bam = relative_path(repo_root, &summary.output_bam);

    let input_qc_pre = bijux_dna_domain_bam::summarize_tiny_bam_qc_pre(&input_bam)?;
    let output_qc_pre = bijux_dna_domain_bam::summarize_tiny_bam_qc_pre(&filtered_bam_path)?;

    bijux_dna_infra::atomic_write_bytes(
        &flagstat_before_path,
        render_flagstat(&summary.flagstat_before).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &flagstat_after_path,
        render_flagstat(&summary.flagstat_after).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &idxstats_before_path,
        render_idxstats(&input_qc_pre).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &idxstats_after_path,
        render_idxstats(&output_qc_pre).as_bytes(),
    )?;
    let expectation_matched = summary.input_reads == case.expected_input_reads
        && summary.kept_reads == case.expected_kept_reads
        && summary.removed_reads == case.expected_removed_reads
        && summary.observed_min_length == Some(case.expected_observed_min_length)
        && summary.observed_max_length == Some(case.expected_observed_max_length);
    bijux_dna_infra::atomic_write_json(&summary_path, &summary)?;
    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": LOCAL_LENGTH_FILTER_SMOKE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.length_filter",
            "sample_id": case.sample_id,
            "min_length_threshold": summary.min_length_threshold,
            "input_reads": summary.input_reads,
            "kept_reads": summary.kept_reads,
            "removed_reads": summary.removed_reads,
            "observed_min_length": summary.observed_min_length,
            "observed_max_length": summary.observed_max_length,
            "expectation_matched": expectation_matched,
        }),
    )?;
    bijux_dna_infra::atomic_write_bytes(&filtered_bai_path, b"tiny-index\n")?;

    let top_level_filtered_bam = output_root.join("length_filtered.bam");
    let top_level_filtered_bai = output_root.join("length_filtered.bam.bai");
    bijux_dna_infra::atomic_write_bytes(
        &top_level_filtered_bam,
        &std::fs::read(&filtered_bam_path)?,
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &top_level_filtered_bai,
        &std::fs::read(&filtered_bai_path)?,
    )?;

    Ok(LocalLengthFilterSmokeReport {
        schema_version: LOCAL_LENGTH_FILTER_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.length_filter".to_string(),
        sample_id: case.sample_id.clone(),
        expectation_matched,
        input_bam: path_relative_to_repo(repo_root, &input_bam),
        filtered_bam: path_relative_to_repo(repo_root, &top_level_filtered_bam),
        filtered_bai: path_relative_to_repo(repo_root, &top_level_filtered_bai),
        min_length_threshold: summary.min_length_threshold,
        input_reads: summary.input_reads,
        kept_reads: summary.kept_reads,
        removed_reads: summary.removed_reads,
        observed_min_length: summary.observed_min_length,
        observed_max_length: summary.observed_max_length,
        length_filter_summary: path_relative_to_repo(repo_root, &summary_path),
        flagstat_before: path_relative_to_repo(repo_root, &flagstat_before_path),
        flagstat_after: path_relative_to_repo(repo_root, &flagstat_after_path),
        idxstats_before: path_relative_to_repo(repo_root, &idxstats_before_path),
        idxstats_after: path_relative_to_repo(repo_root, &idxstats_after_path),
        stage_metrics: path_relative_to_repo(repo_root, &stage_metrics_path),
    })
}

fn render_flagstat(flagstat: &bijux_dna_domain_bam::BamFlagstatCountsV1) -> String {
    let total_reads = flagstat.total_reads.unwrap_or(0);
    let mapped_reads = flagstat.mapped_reads.unwrap_or(0);
    let duplicate_reads = flagstat.duplicate_reads.unwrap_or(0);
    let mapped_fraction = flagstat
        .mapped_fraction
        .map(|fraction| format!("{:.2}%", fraction * 100.0))
        .unwrap_or_else(|| "N/A".to_string());
    format!(
        "{total_reads} + 0 in total (QC-passed reads + QC-failed reads)\n\
{mapped_reads} + 0 mapped ({mapped_fraction} : N/A)\n\
{duplicate_reads} + 0 duplicates\n",
    )
}

fn render_idxstats(summary: &bijux_dna_domain_bam::BamQcPreSummaryV1) -> String {
    summary
        .contig_summary
        .iter()
        .map(|contig| {
            format!(
                "{contig}\t{length}\t{mapped}\t{unmapped}\n",
                contig = contig.contig,
                length = contig.length,
                mapped = contig.mapped,
                unmapped = contig.unmapped
            )
        })
        .collect()
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
            anyhow!("bam.length_filter local-smoke plan is missing governed output `{output_id}`")
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
