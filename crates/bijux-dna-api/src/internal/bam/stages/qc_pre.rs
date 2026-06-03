use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

const LOCAL_QC_PRE_SMOKE_REPORT_SCHEMA_VERSION: &str = "bijux.bam.qc_pre.local_smoke.report.v1";
const LOCAL_QC_PRE_SMOKE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.qc_pre.local_smoke.metrics.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalQcPreContigSummary {
    contig: String,
    length: u64,
    mapped: u64,
    unmapped: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalQcPreSmokeCaseReport {
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    total_reads: u64,
    mapped_reads: u64,
    unmapped_reads: u64,
    duplicate_flagged_reads: u64,
    contig_summary: Vec<LocalQcPreContigSummary>,
    reference_mismatch: bool,
    qc_pre_summary: String,
    flagstat: String,
    idxstats: String,
    samtools_stats: String,
    stage_metrics: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalQcPreSmokeReport {
    schema_version: String,
    stage_id: String,
    case_count: u64,
    all_cases_matched: bool,
    cases: Vec<LocalQcPreSmokeCaseReport>,
}

/// Materialize the governed local-smoke `bam.qc_pre` artifacts and summary report.
///
/// The written summary artifact lives at `target/local-smoke/bam.qc_pre/qc_pre.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_qc_pre_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_qc_pre_smoke_plans(&repo_root)?;
    let output_root = repo_root.join("target/local-smoke/bam.qc_pre");
    bijux_dna_infra::ensure_dir(&output_root)?;

    let case_reports = cases
        .iter()
        .map(|case| materialize_local_qc_pre_smoke_case(&repo_root, case))
        .collect::<Result<Vec<_>>>()?;

    let summary = LocalQcPreSmokeReport {
        schema_version: LOCAL_QC_PRE_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.qc_pre".to_string(),
        case_count: case_reports.len() as u64,
        all_cases_matched: case_reports.iter().all(|case| case.expectation_matched),
        cases: case_reports,
    };

    let report_path = output_root.join("qc_pre.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    Ok(report_path)
}

/// Write a durable `qc_pre.summary.json` artifact beside BAM qc_pre stage outputs.
///
/// # Errors
/// Returns an error if the stage artifacts cannot be parsed or the summary cannot be written.
pub(crate) fn write_stage_qc_pre_summary(stage_dir: &Path, input_bam: &Path) -> Result<PathBuf> {
    let summary = summarize_qc_pre_outputs(stage_dir, input_bam)?;
    let path = stage_dir.join("qc_pre.summary.json");
    bijux_dna_infra::atomic_write_json(&path, &summary)?;
    Ok(path)
}

fn materialize_local_qc_pre_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalQcPreSmokeCasePlan,
) -> Result<LocalQcPreSmokeCaseReport> {
    let case_out_dir = resolve_plan_dir(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let bam = repo_root.join(&case.bam);
    let mut summary = bijux_dna_domain_bam::summarize_tiny_bam_qc_pre(&bam)?;
    normalize_summary_paths(repo_root, &mut summary);

    let flagstat_path = resolve_output_path(repo_root, &case.plan, "flagstat")?;
    let idxstats_path = resolve_output_path(repo_root, &case.plan, "idxstats")?;
    let stats_path = resolve_output_path(repo_root, &case.plan, "stats")?;
    let stage_metrics_path = resolve_output_path(repo_root, &case.plan, "stage_metrics")?;
    let qc_pre_summary_path = case_out_dir.join("qc_pre.summary.json");
    let observed_contigs =
        summary.contig_summary.iter().map(|contig| contig.contig.clone()).collect::<Vec<_>>();
    let expectation_matched = summary.total_reads == case.expected_total_reads
        && summary.mapped_reads == case.expected_mapped_reads
        && summary.unmapped_reads == case.expected_unmapped_reads
        && summary.duplicate_flagged_reads == case.expected_duplicate_flagged_reads
        && observed_contigs == case.expected_contigs;

    bijux_dna_infra::atomic_write_bytes(&flagstat_path, render_flagstat(&summary).as_bytes())?;
    bijux_dna_infra::atomic_write_bytes(&idxstats_path, render_idxstats(&summary).as_bytes())?;
    bijux_dna_infra::atomic_write_bytes(&stats_path, render_stats(&summary).as_bytes())?;
    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": LOCAL_QC_PRE_SMOKE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.qc_pre",
            "sample_id": case.sample_id,
            "total_reads": summary.total_reads,
            "mapped_reads": summary.mapped_reads,
            "unmapped_reads": summary.unmapped_reads,
            "duplicate_flagged_reads": summary.duplicate_flagged_reads,
            "reference_mismatch": summary.reference_mismatch,
            "expectation_matched": expectation_matched,
            "contig_summary": summary
                .contig_summary
                .iter()
                .map(|contig| serde_json::json!({
                    "contig": contig.contig,
                    "length": contig.length,
                    "mapped": contig.mapped,
                    "unmapped": contig.unmapped,
                }))
                .collect::<Vec<_>>(),
        }),
    )?;
    bijux_dna_infra::atomic_write_json(&qc_pre_summary_path, &summary)?;

    let contig_summary = summary
        .contig_summary
        .iter()
        .map(|contig| LocalQcPreContigSummary {
            contig: contig.contig.clone(),
            length: contig.length,
            mapped: contig.mapped,
            unmapped: contig.unmapped,
        })
        .collect::<Vec<_>>();

    Ok(LocalQcPreSmokeCaseReport {
        sample_id: case.sample_id.clone(),
        expectation_matched,
        input_bam: summary.input_bam.display().to_string(),
        total_reads: summary.total_reads,
        mapped_reads: summary.mapped_reads,
        unmapped_reads: summary.unmapped_reads,
        duplicate_flagged_reads: summary.duplicate_flagged_reads,
        contig_summary,
        reference_mismatch: summary.reference_mismatch,
        qc_pre_summary: path_relative_to_repo(repo_root, &qc_pre_summary_path),
        flagstat: path_relative_to_repo(repo_root, &flagstat_path),
        idxstats: path_relative_to_repo(repo_root, &idxstats_path),
        samtools_stats: path_relative_to_repo(repo_root, &stats_path),
        stage_metrics: path_relative_to_repo(repo_root, &stage_metrics_path),
    })
}

fn summarize_qc_pre_outputs(
    stage_dir: &Path,
    input_bam: &Path,
) -> Result<bijux_dna_domain_bam::BamQcPreSummaryV1> {
    let flagstat =
        bijux_dna_domain_bam::metrics::parse_samtools_flagstat(&stage_dir.join("flagstat.txt"))?;
    let idxstats =
        bijux_dna_domain_bam::metrics::parse_samtools_idxstats(&stage_dir.join("idxstats.txt"))?;
    let (fragment_length, mapq) =
        bijux_dna_domain_bam::metrics::parse_samtools_stats(&stage_dir.join("samtools_stats.txt"))?;
    Ok(bijux_dna_domain_bam::BamQcPreSummaryV1 {
        schema_version: bijux_dna_domain_bam::BAM_QC_PRE_SUMMARY_SCHEMA_VERSION.to_string(),
        stage_id: "bam.qc_pre".to_string(),
        input_bam: input_bam.to_path_buf(),
        total_reads: flagstat.total,
        mapped_reads: flagstat.mapped,
        unmapped_reads: flagstat.total.saturating_sub(flagstat.mapped),
        duplicate_flagged_reads: flagstat.duplicates,
        contig_summary: idxstats.contigs.clone(),
        reference_mismatch: idxstats.reference_mismatch,
        fragment_length,
        mapq,
    })
}

fn render_flagstat(summary: &bijux_dna_domain_bam::BamQcPreSummaryV1) -> String {
    let mapped_fraction = if summary.total_reads > 0 {
        format!("{:.2}%", (summary.mapped_reads as f64 / summary.total_reads as f64) * 100.0)
    } else {
        "N/A".to_string()
    };
    format!(
        "{total} + 0 in total (QC-passed reads + QC-failed reads)\n\
{mapped} + 0 mapped ({mapped_fraction} : N/A)\n\
{duplicates} + 0 duplicates\n",
        total = summary.total_reads,
        mapped = summary.mapped_reads,
        duplicates = summary.duplicate_flagged_reads,
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

fn render_stats(summary: &bijux_dna_domain_bam::BamQcPreSummaryV1) -> String {
    let mut payload = String::new();
    if summary.mapped_reads > 0 {
        let representative_length = summary.fragment_length.median.round().max(0.0) as u64;
        if representative_length > 0 {
            use std::fmt::Write as _;
            let _ = writeln!(
                payload,
                "RL\t{length}\t{count}",
                length = representative_length,
                count = summary.mapped_reads
            );
        }
    }
    for (mapq, count) in &summary.mapq.histogram {
        use std::fmt::Write as _;
        let _ = writeln!(payload, "MQ\t{mapq}\t{count}");
    }
    payload
}

fn normalize_summary_paths(
    repo_root: &Path,
    summary: &mut bijux_dna_domain_bam::BamQcPreSummaryV1,
) {
    summary.input_bam = relative_path(repo_root, &summary.input_bam);
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
            anyhow!("bam.qc_pre local-smoke plan is missing governed output `{output_id}`")
        })?;
    Ok(resolve_plan_dir(repo_root, &path))
}

fn resolve_plan_dir(repo_root: &Path, path: &Path) -> PathBuf {
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
