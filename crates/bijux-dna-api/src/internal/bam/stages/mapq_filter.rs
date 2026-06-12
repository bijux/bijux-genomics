use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::Serialize;

const LOCAL_MAPQ_FILTER_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.bam.mapq_filter.local_smoke.report.v1";
const LOCAL_MAPQ_FILTER_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.mapq_filter.local_smoke.metrics.v1";

#[derive(Debug, Clone, Serialize)]
struct LocalMapqFilterSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    filtered_bam: String,
    filtered_bai: String,
    mapq_threshold: u8,
    input_reads: u64,
    kept_reads: u64,
    removed_reads: u64,
    mapped_reads_removed: Option<u64>,
    mapped_fraction_retained: Option<f64>,
    mapq_filter_summary: String,
    flagstat_before: String,
    flagstat_after: String,
    idxstats_before: String,
    idxstats_after: String,
    stage_metrics: String,
}

/// Materialize the governed local-smoke `bam.mapq_filter` artifacts and top-level report.
///
/// The written report lives at `runs/bench/local-smoke/bam.mapq_filter/mapq_filter.json`
/// under the active repository root, alongside the curated top-level `mapq_filtered.bam`.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_mapq_filter_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_mapq_filter_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "local-smoke bam.mapq_filter expects exactly one governed case, found {}",
            cases.len()
        ));
    };

    let output_root = repo_root.join("runs/bench/local-smoke/bam.mapq_filter");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let report = materialize_local_mapq_filter_smoke_case(&repo_root, case, &output_root)?;
    let report_path = output_root.join("mapq_filter.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(report_path)
}

fn materialize_local_mapq_filter_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalMapqFilterSmokeCasePlan,
    output_root: &Path,
) -> Result<LocalMapqFilterSmokeReport> {
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
    let mut summary = bijux_dna_domain_bam::filter_tiny_bam_by_mapq(
        &input_bam,
        &filtered_bam_path,
        case.mapq_threshold,
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
        && summary.mapped_reads_removed == Some(case.expected_mapped_reads_removed)
        && summary.mapped_fraction_retained.is_some_and(|fraction| {
            (fraction - case.expected_mapped_fraction_retained).abs() <= 1e-9
        });
    bijux_dna_infra::atomic_write_json(&summary_path, &summary)?;
    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": LOCAL_MAPQ_FILTER_SMOKE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.mapq_filter",
            "sample_id": case.sample_id,
            "mapq_threshold": summary.mapq_threshold,
            "input_reads": summary.input_reads,
            "kept_reads": summary.kept_reads,
            "removed_reads": summary.removed_reads,
            "mapped_reads_removed": summary.mapped_reads_removed,
            "mapped_fraction_retained": summary.mapped_fraction_retained,
            "expectation_matched": expectation_matched,
        }),
    )?;
    bijux_dna_infra::atomic_write_bytes(&filtered_bai_path, b"tiny-index\n")?;

    let top_level_filtered_bam = output_root.join("mapq_filtered.bam");
    let top_level_filtered_index = output_root.join("mapq_filtered.bam.bai");
    bijux_dna_infra::atomic_write_bytes(
        &top_level_filtered_bam,
        &std::fs::read(&filtered_bam_path)?,
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &top_level_filtered_index,
        &std::fs::read(&filtered_bai_path)?,
    )?;

    Ok(LocalMapqFilterSmokeReport {
        schema_version: LOCAL_MAPQ_FILTER_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.mapq_filter".to_string(),
        sample_id: case.sample_id.clone(),
        expectation_matched,
        input_bam: path_relative_to_repo(repo_root, &input_bam),
        filtered_bam: path_relative_to_repo(repo_root, &top_level_filtered_bam),
        filtered_bai: path_relative_to_repo(repo_root, &top_level_filtered_index),
        mapq_threshold: summary.mapq_threshold,
        input_reads: summary.input_reads,
        kept_reads: summary.kept_reads,
        removed_reads: summary.removed_reads,
        mapped_reads_removed: summary.mapped_reads_removed,
        mapped_fraction_retained: summary.mapped_fraction_retained,
        mapq_filter_summary: path_relative_to_repo(repo_root, &summary_path),
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
    let mapped_fraction = flagstat.mapped_fraction.map_or_else(
        || "N/A".to_string(),
        |fraction| format!("{fraction:.2}%", fraction = fraction * 100.0),
    );
    format!(
        "{total_reads} + 0 in total (QC-passed reads + QC-failed reads)\n\
{mapped_reads} + 0 mapped ({mapped_fraction} : N/A)\n\
{duplicate_reads} + 0 duplicates\n",
    )
}

fn render_idxstats(summary: &bijux_dna_domain_bam::BamQcPreSummaryV1) -> String {
    use std::fmt::Write as _;

    summary.contig_summary.iter().fold(String::new(), |mut rendered, contig| {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}",
            contig.contig,
            contig.length,
            contig.mapped,
            contig.unmapped
        );
        rendered
    })
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
            anyhow!("bam.mapq_filter local-smoke plan is missing governed output `{output_id}`")
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
