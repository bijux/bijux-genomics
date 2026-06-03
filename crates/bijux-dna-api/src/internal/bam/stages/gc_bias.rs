use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

const LOCAL_GC_BIAS_SMOKE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.gc_bias.local_smoke.metrics.v1";

/// Materialize the governed local-smoke `bam.gc_bias` artifacts and TSV summary.
///
/// The written summary artifact lives at `target/local-smoke/bam.gc_bias/gc_bias.tsv`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_gc_bias_smoke_summary() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_gc_bias_smoke_plans(&repo_root)?;
    let output_root = repo_root.join("target/local-smoke/bam.gc_bias");
    bijux_dna_infra::ensure_dir(&output_root)?;

    let mut body = String::from(
        "sample_id\tgc_bin\tnormalized_coverage\twindows\tread_starts\tinsufficient_reference_reason\trow_expectation_matched\tcase_expectation_matched\tinput_bam\treference_fasta\tgc_bias_tsv\tgc_bias_summary_json\tgc_bias_metrics\tgc_bias_plot\tstage_metrics\n",
    );
    for case in &cases {
        for row in materialize_local_gc_bias_smoke_case(&repo_root, case)? {
            writeln!(
                body,
                "{sample_id}\t{gc_bin}\t{normalized_coverage}\t{windows}\t{read_starts}\t{insufficient_reference_reason}\t{row_expectation_matched}\t{case_expectation_matched}\t{input_bam}\t{reference_fasta}\t{gc_bias_tsv}\t{gc_bias_summary_json}\t{gc_bias_metrics}\t{gc_bias_plot}\t{stage_metrics}",
                sample_id = row.sample_id,
                gc_bin = row.gc_bin,
                normalized_coverage = row.normalized_coverage,
                windows = row.windows,
                read_starts = row.read_starts,
                insufficient_reference_reason = row.insufficient_reference_reason,
                row_expectation_matched = row.row_expectation_matched,
                case_expectation_matched = row.case_expectation_matched,
                input_bam = row.input_bam,
                reference_fasta = row.reference_fasta,
                gc_bias_tsv = row.gc_bias_tsv,
                gc_bias_summary_json = row.gc_bias_summary_json,
                gc_bias_metrics = row.gc_bias_metrics,
                gc_bias_plot = row.gc_bias_plot,
                stage_metrics = row.stage_metrics,
            )
            .map_err(|error| anyhow!("write bam.gc_bias local-smoke TSV row: {error}"))?;
        }
    }

    let summary_path = output_root.join("gc_bias.tsv");
    bijux_dna_infra::atomic_write_bytes(&summary_path, body.as_bytes())?;
    Ok(summary_path)
}

/// Write a durable typed `gc_bias.summary.json` artifact beside BAM gc-bias stage outputs.
///
/// # Errors
/// Returns an error if the raw Picard metrics cannot be parsed or the summary cannot be written.
pub(crate) fn write_stage_gc_bias_summary(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<PathBuf> {
    let input_bam = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.role == bijux_dna_core::contract::ArtifactRole::Bam)
        .map_or_else(|| stage_dir.join("in.bam"), |artifact| artifact.path.clone());
    let reference_fasta = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.role == bijux_dna_core::contract::ArtifactRole::Reference)
        .map_or_else(|| stage_dir.join("reference.fasta"), |artifact| artifact.path.clone());
    let window_size = plan
        .params
        .get("window_size")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok());
    let report_path = stage_dir.join("gc_bias.metrics.txt");
    let metrics = if report_path.exists() {
        Some(bijux_dna_domain_bam::metrics::parse_picard_gc_bias_metrics(&report_path)?)
    } else {
        None
    };
    let summary = bijux_dna_domain_bam::summarize_bam_gc_bias(
        "bam.gc_bias",
        &input_bam,
        &reference_fasta,
        window_size,
        report_path.exists(),
        stage_dir.join("gc_bias.plot.pdf").exists(),
        metrics.as_ref(),
        None,
    );
    let path = stage_dir.join("gc_bias.summary.json");
    bijux_dna_infra::atomic_write_json(&path, &summary)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

struct LocalGcBiasSmokeRow {
    sample_id: String,
    gc_bin: String,
    normalized_coverage: String,
    windows: String,
    read_starts: String,
    insufficient_reference_reason: String,
    row_expectation_matched: bool,
    case_expectation_matched: bool,
    input_bam: String,
    reference_fasta: String,
    gc_bias_tsv: String,
    gc_bias_summary_json: String,
    gc_bias_metrics: String,
    gc_bias_plot: String,
    stage_metrics: String,
}

fn materialize_local_gc_bias_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalGcBiasSmokeCasePlan,
) -> Result<Vec<LocalGcBiasSmokeRow>> {
    let case_out_dir = resolve_plan_path(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let gc_bias_metrics = resolve_output_path(repo_root, &case.plan, "gc_bias_report")?;
    let gc_bias_plot = resolve_output_path(repo_root, &case.plan, "gc_bias_plot")?;
    let gc_bias_summary_json = resolve_output_path(repo_root, &case.plan, "summary")?;
    let stage_metrics = resolve_output_path(repo_root, &case.plan, "stage_metrics")?;
    let gc_bias_tsv = case_out_dir.join("gc_bias.tsv");

    let input_bam = repo_root.join(&case.bam);
    let reference_fasta = repo_root.join(&case.reference);
    let (mut summary, rows) =
        bijux_dna_domain_bam::summarize_tiny_bam_gc_bias(&input_bam, &reference_fasta, case.window_size)?;
    summary.input_bam = relative_path(repo_root, &summary.input_bam);
    summary.reference_fasta = relative_path(repo_root, &summary.reference_fasta);

    bijux_dna_infra::atomic_write_bytes(
        &gc_bias_metrics,
        render_picard_gc_bias_metrics(&summary, &rows).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &gc_bias_plot,
        b"%PDF-1.4\n% bijux bam gc-bias local smoke plot placeholder\n",
    )?;
    bijux_dna_infra::atomic_write_json(&gc_bias_summary_json, &summary)?;
    bijux_dna_infra::atomic_write_bytes(&gc_bias_tsv, render_case_gc_bias_tsv(&rows, &summary).as_bytes())?;

    let expected_by_gc_bin = case
        .expected_rows
        .iter()
        .map(|row| (row.gc_bin, row))
        .collect::<HashMap<_, _>>();
    let row_expectation_matched = rows.iter().all(|row| {
        expected_by_gc_bin.get(&row.gc_bin).is_some_and(|expected| gc_bias_row_matches(row, expected))
    });
    let case_expectation_matched =
        summary.insufficient_reference_reason.is_none() && row_expectation_matched;

    bijux_dna_infra::atomic_write_json(
        &stage_metrics,
        &serde_json::json!({
            "schema_version": LOCAL_GC_BIAS_SMOKE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.gc_bias",
            "sample_id": case.sample_id,
            "window_size": case.window_size,
            "expected_row_count": case.expected_rows.len(),
            "observed_row_count": rows.len(),
            "gc_bias_score": summary.gc_bias_score,
            "at_dropout": summary.at_dropout,
            "gc_dropout": summary.gc_dropout,
            "insufficient_reference_reason": summary.insufficient_reference_reason,
            "observed_gc_bins": rows.iter().map(|row| row.gc_bin).collect::<Vec<_>>(),
            "row_expectation_matched": row_expectation_matched,
            "case_expectation_matched": case_expectation_matched,
        }),
    )?;

    if rows.is_empty() {
        return Ok(vec![LocalGcBiasSmokeRow {
            sample_id: case.sample_id.clone(),
            gc_bin: String::new(),
            normalized_coverage: String::new(),
            windows: String::new(),
            read_starts: String::new(),
            insufficient_reference_reason: summary
                .insufficient_reference_reason
                .clone()
                .unwrap_or_else(|| "reference_gc_windows_unavailable".to_string()),
            row_expectation_matched: false,
            case_expectation_matched,
            input_bam: path_relative_to_repo(repo_root, &input_bam),
            reference_fasta: path_relative_to_repo(repo_root, &reference_fasta),
            gc_bias_tsv: path_relative_to_repo(repo_root, &gc_bias_tsv),
            gc_bias_summary_json: path_relative_to_repo(repo_root, &gc_bias_summary_json),
            gc_bias_metrics: path_relative_to_repo(repo_root, &gc_bias_metrics),
            gc_bias_plot: path_relative_to_repo(repo_root, &gc_bias_plot),
            stage_metrics: path_relative_to_repo(repo_root, &stage_metrics),
        }]);
    }

    Ok(rows
        .into_iter()
        .map(|row| {
            let row_match =
                expected_by_gc_bin.get(&row.gc_bin).is_some_and(|expected| gc_bias_row_matches(&row, expected));
            LocalGcBiasSmokeRow {
                sample_id: case.sample_id.clone(),
                gc_bin: row.gc_bin.to_string(),
                normalized_coverage: format!("{:.6}", row.normalized_coverage),
                windows: row.windows.to_string(),
                read_starts: row.read_starts.to_string(),
                insufficient_reference_reason: String::new(),
                row_expectation_matched: row_match,
                case_expectation_matched,
                input_bam: path_relative_to_repo(repo_root, &input_bam),
                reference_fasta: path_relative_to_repo(repo_root, &reference_fasta),
                gc_bias_tsv: path_relative_to_repo(repo_root, &gc_bias_tsv),
                gc_bias_summary_json: path_relative_to_repo(repo_root, &gc_bias_summary_json),
                gc_bias_metrics: path_relative_to_repo(repo_root, &gc_bias_metrics),
                gc_bias_plot: path_relative_to_repo(repo_root, &gc_bias_plot),
                stage_metrics: path_relative_to_repo(repo_root, &stage_metrics),
            }
        })
        .collect())
}

fn gc_bias_row_matches(
    observed: &bijux_dna_domain_bam::BamGcBiasBinSummaryV1,
    expected: &bijux_dna_planner_bam::stage_api::LocalGcBiasSmokeExpectedRow,
) -> bool {
    observed.gc_bin == expected.gc_bin
        && observed.windows == expected.windows
        && observed.read_starts == expected.read_starts
        && (observed.normalized_coverage - expected.normalized_coverage).abs() <= 1e-9
}

fn render_case_gc_bias_tsv(
    rows: &[bijux_dna_domain_bam::BamGcBiasBinSummaryV1],
    summary: &bijux_dna_domain_bam::BamGcBiasSummaryV1,
) -> String {
    let mut body =
        String::from("gc_bin\tnormalized_coverage\twindows\tread_starts\tinsufficient_reference_reason\n");
    if rows.is_empty() {
        let _ = writeln!(
            body,
            "\t\t\t\t{}",
            summary
                .insufficient_reference_reason
                .as_deref()
                .unwrap_or("reference_gc_windows_unavailable")
        );
        return body;
    }

    for row in rows {
        let _ = writeln!(
            body,
            "{gc_bin}\t{normalized_coverage:.6}\t{windows}\t{read_starts}\t",
            gc_bin = row.gc_bin,
            normalized_coverage = row.normalized_coverage,
            windows = row.windows,
            read_starts = row.read_starts,
        );
    }
    body
}

fn render_picard_gc_bias_metrics(
    summary: &bijux_dna_domain_bam::BamGcBiasSummaryV1,
    rows: &[bijux_dna_domain_bam::BamGcBiasBinSummaryV1],
) -> String {
    let mut body = format!(
        "## htsjdk.samtools.metrics.StringHeader\n\
# picard CollectGcBiasMetrics bijux local smoke fixture\n\
## METRICS CLASS\tpicard.analysis.GcBiasMetrics\n\
ACCUMULATION_LEVEL\tREADS_USED\tWINDOW_SIZE\tTOTAL_CLUSTERS\tALIGNED_READS\tAT_DROPOUT\tGC_DROPOUT\tWINDOWS\tREAD_STARTS\n\
ALL_READS\tALL\t{window_size}\t{total_clusters}\t{aligned_reads}\t{at_dropout}\t{gc_dropout}\t{windows}\t{read_starts}\n\
\n\
## HISTOGRAM\tjava.lang.Integer\n\
GC\tNORMALIZED_COVERAGE\tWINDOWS\tERROR_BAR_WIDTH\n",
        window_size = summary.window_size.unwrap_or(0),
        total_clusters = summary.total_clusters,
        aligned_reads = summary.aligned_reads,
        at_dropout = summary.at_dropout,
        gc_dropout = summary.gc_dropout,
        windows = summary.windows,
        read_starts = summary.read_starts,
    );
    for row in rows {
        let _ = writeln!(
            body,
            "{gc_bin}\t{normalized_coverage:.6}\t{windows}\t0.0",
            gc_bin = row.gc_bin,
            normalized_coverage = row.normalized_coverage,
            windows = row.windows,
        );
    }
    body
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
        .ok_or_else(|| anyhow!("bam.gc_bias local-smoke plan is missing governed output `{output_id}`"))?;
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
