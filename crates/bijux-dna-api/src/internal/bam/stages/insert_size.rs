use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

const LOCAL_INSERT_SIZE_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.bam.insert_size.local_smoke.report.v1";
const LOCAL_INSERT_SIZE_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.insert_size.local_smoke.metrics.v1";

#[derive(Debug, Clone, Serialize)]
struct LocalInsertSizeSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    method: String,
    read_pairs: u64,
    median_insert_size: Option<f64>,
    mean_insert_size: Option<f64>,
    standard_deviation: Option<f64>,
    min_insert_size: Option<u64>,
    max_insert_size: Option<u64>,
    insufficient_pairs_reason: Option<String>,
    insert_size_report: String,
    insert_size_histogram: String,
    insert_size_summary: String,
    stage_metrics: String,
}

/// Materialize the governed local-smoke `bam.insert_size` artifacts and top-level report.
///
/// The written report lives at `runs/bench/local-smoke/bam.insert_size/insert_size.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_insert_size_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_insert_size_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "local-smoke bam.insert_size expects exactly one governed case, found {}",
            cases.len()
        ));
    };

    let output_root = repo_root.join("runs/bench/local-smoke/bam.insert_size");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let report = materialize_local_insert_size_smoke_case(&repo_root, case)?;
    let report_path = output_root.join("insert_size.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(report_path)
}

/// Write a durable typed `insert_size.summary.json` artifact beside BAM insert-size outputs.
///
/// # Errors
/// Returns an error if the raw Picard metrics cannot be parsed or the summary cannot be written.
pub(crate) fn write_stage_insert_size_summary(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<PathBuf> {
    let input_bam = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.role == bijux_dna_core::contract::ArtifactRole::Bam)
        .map_or_else(|| stage_dir.join("in.bam"), |artifact| artifact.path.clone());
    let summary = summarize_stage_insert_size_outputs(stage_dir, &input_bam)?;
    let path = stage_dir.join("insert_size.summary.json");
    bijux_dna_infra::atomic_write_json(&path, &summary)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

fn materialize_local_insert_size_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalInsertSizeSmokeCasePlan,
) -> Result<LocalInsertSizeSmokeReport> {
    let case_out_dir = resolve_plan_path(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let insert_size_report_path = resolve_output_path(repo_root, &case.plan, "insert_size_report")?;
    let insert_size_histogram_path =
        resolve_output_path(repo_root, &case.plan, "insert_size_histogram")?;
    let insert_size_summary_path = resolve_output_path(repo_root, &case.plan, "summary")?;
    let stage_metrics_path = resolve_output_path(repo_root, &case.plan, "stage_metrics")?;

    let input_bam = repo_root.join(&case.bam);
    let mut summary = bijux_dna_domain_bam::summarize_tiny_bam_insert_size(&input_bam)?;
    summary.input_bam = relative_path(repo_root, &summary.input_bam);

    bijux_dna_infra::atomic_write_bytes(
        &insert_size_report_path,
        render_picard_insert_size_metrics(&summary).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &insert_size_histogram_path,
        b"%PDF-1.4\n% bijux bam insert size local smoke histogram placeholder\n",
    )?;
    let _summary_path = write_stage_insert_size_summary(&case_out_dir, &case.plan)?;
    let written_summary: bijux_dna_domain_bam::BamInsertSizeSummaryV1 = serde_json::from_str(
        &std::fs::read_to_string(&insert_size_summary_path)
            .with_context(|| format!("read {}", insert_size_summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", insert_size_summary_path.display()))?;

    let expectation_matched = written_summary.read_pairs == case.expected_read_pairs
        && float_matches(
            written_summary.median_insert_size,
            Some(case.expected_median_insert_size),
        )
        && float_matches(written_summary.mean_insert_size, Some(case.expected_mean_insert_size))
        && written_summary.min_insert_size == Some(case.expected_min_insert_size)
        && written_summary.max_insert_size == Some(case.expected_max_insert_size)
        && written_summary.insufficient_pairs_reason.is_none();

    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": LOCAL_INSERT_SIZE_SMOKE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.insert_size",
            "sample_id": case.sample_id,
            "method": case.plan.tool_id.as_str(),
            "read_pairs": written_summary.read_pairs,
            "median_insert_size": written_summary.median_insert_size,
            "mean_insert_size": written_summary.mean_insert_size,
            "standard_deviation": written_summary.standard_deviation,
            "min_insert_size": written_summary.min_insert_size,
            "max_insert_size": written_summary.max_insert_size,
            "insufficient_pairs_reason": written_summary.insufficient_pairs_reason,
            "expectation_matched": expectation_matched,
        }),
    )?;

    Ok(LocalInsertSizeSmokeReport {
        schema_version: LOCAL_INSERT_SIZE_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.insert_size".to_string(),
        sample_id: case.sample_id.clone(),
        expectation_matched,
        input_bam: path_relative_to_repo(repo_root, &input_bam),
        method: case.plan.tool_id.as_str().to_string(),
        read_pairs: written_summary.read_pairs,
        median_insert_size: written_summary.median_insert_size,
        mean_insert_size: written_summary.mean_insert_size,
        standard_deviation: written_summary.standard_deviation,
        min_insert_size: written_summary.min_insert_size,
        max_insert_size: written_summary.max_insert_size,
        insufficient_pairs_reason: written_summary.insufficient_pairs_reason.clone(),
        insert_size_report: path_relative_to_repo(repo_root, &insert_size_report_path),
        insert_size_histogram: path_relative_to_repo(repo_root, &insert_size_histogram_path),
        insert_size_summary: path_relative_to_repo(repo_root, &insert_size_summary_path),
        stage_metrics: path_relative_to_repo(repo_root, &stage_metrics_path),
    })
}

fn summarize_stage_insert_size_outputs(
    stage_dir: &Path,
    input_bam: &Path,
) -> Result<bijux_dna_domain_bam::BamInsertSizeSummaryV1> {
    let report_path = stage_dir.join("insert_size.metrics.txt");
    let metrics = if report_path.exists() {
        Some(bijux_dna_domain_bam::metrics::parse_picard_insert_size_metrics(&report_path)?)
    } else {
        None
    };
    Ok(bijux_dna_domain_bam::summarize_bam_insert_size(
        "bam.insert_size",
        input_bam,
        report_path.exists(),
        stage_dir.join("insert_size.histogram.pdf").exists(),
        metrics.as_ref(),
    ))
}

fn render_picard_insert_size_metrics(
    summary: &bijux_dna_domain_bam::BamInsertSizeSummaryV1,
) -> String {
    let orientation =
        if summary.pair_orientation_fr_fraction.unwrap_or(0.0) > 0.0 { "FR" } else { "TANDEM" };
    format!(
        "## htsjdk.samtools.metrics.StringHeader\n\
# picard CollectInsertSizeMetrics bijux local smoke fixture\n\
## METRICS CLASS\tpicard.analysis.InsertSizeMetrics\n\
MEDIAN_INSERT_SIZE\tMODE_INSERT_SIZE\tMEDIAN_ABSOLUTE_DEVIATION\tMIN_INSERT_SIZE\tMAX_INSERT_SIZE\tMEAN_INSERT_SIZE\tSTANDARD_DEVIATION\tREAD_PAIRS\tPAIR_ORIENTATION\tWIDTH_OF_10_PERCENT\tWIDTH_OF_20_PERCENT\tWIDTH_OF_30_PERCENT\tWIDTH_OF_40_PERCENT\tWIDTH_OF_50_PERCENT\tWIDTH_OF_60_PERCENT\tWIDTH_OF_70_PERCENT\tWIDTH_OF_80_PERCENT\tWIDTH_OF_90_PERCENT\tWIDTH_OF_99_PERCENT\tSAMPLE\tLIBRARY\tREAD_GROUP\n\
{median}\t{mode}\t{mad}\t{min}\t{max}\t{mean}\t{stddev}\t{pairs}\t{orientation}\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\tNA\tNA\tNA\n\
\n\
## HISTOGRAM\tjava.lang.Integer\n\
insert_size\tAll_Reads.fr_count\n\
{histogram_insert_size}\t{pairs}\n",
        median = summary.median_insert_size.unwrap_or(0.0),
        mode = summary.median_insert_size.unwrap_or(0.0).round() as u64,
        mad = summary.median_absolute_deviation.unwrap_or(0.0),
        min = summary.min_insert_size.unwrap_or(0),
        max = summary.max_insert_size.unwrap_or(0),
        mean = summary.mean_insert_size.unwrap_or(0.0),
        stddev = summary.standard_deviation.unwrap_or(0.0),
        pairs = summary.read_pairs,
        orientation = orientation,
        histogram_insert_size = summary
            .median_insert_size
            .map(|value| value.round() as u64)
            .unwrap_or(0),
    )
}

fn float_matches(left: Option<f64>, right: Option<f64>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => (left - right).abs() <= 1e-9,
        (None, None) => true,
        _ => false,
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
            anyhow!("bam.insert_size local-smoke plan is missing governed output `{output_id}`")
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
