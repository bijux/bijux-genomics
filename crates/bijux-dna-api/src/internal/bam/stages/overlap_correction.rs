use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

const LOCAL_OVERLAP_CORRECTION_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.bam.overlap_correction.local_smoke.report.v1";
const LOCAL_OVERLAP_CORRECTION_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.overlap_correction.local_smoke.metrics.v1";
const OVERLAP_CORRECTION_STAGE_ID: &str = "bam.overlap_correction";
const OVERLAP_CORRECTION_PAIR_METRICS_UNAVAILABLE: &str =
    "pair_metrics_not_available_from_runtime_outputs";

#[derive(Debug, Clone, Serialize)]
struct LocalOverlapCorrectionSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    overlap_corrected_bam: String,
    method: String,
    pair_count: u64,
    corrected_pairs: u64,
    corrected_overlap_bases: u64,
    insufficiency_reason: Option<String>,
    overlap_correction_summary: String,
    flagstat_before: String,
    flagstat_after: String,
    idxstats_before: String,
    idxstats_after: String,
    stage_metrics: String,
}

/// Materialize the governed local-smoke `bam.overlap_correction` artifacts and top-level report.
///
/// The written report lives at
/// `runs/bench/local-smoke/bam.overlap_correction/overlap_correction.json`
/// under the active repository root, alongside the curated top-level
/// `overlap_corrected.bam`.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_overlap_correction_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_overlap_correction_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "local-smoke bam.overlap_correction expects exactly one governed case, found {}",
            cases.len()
        ));
    };

    let output_root = repo_root.join("runs/bench/local-smoke/bam.overlap_correction");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let report = materialize_local_overlap_correction_smoke_case(&repo_root, case, &output_root)?;
    let report_path = output_root.join("overlap_correction.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(report_path)
}

/// Write a durable typed `overlap_correction.summary.json` artifact beside overlap-correction
/// outputs.
///
/// # Errors
/// Returns an error if the overlap-corrected BAM is missing or the summary cannot be written.
pub(crate) fn write_stage_overlap_correction_summary(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<PathBuf> {
    let input_bam = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.role == bijux_dna_core::contract::ArtifactRole::Bam)
        .map_or_else(|| stage_dir.join("in.bam"), |artifact| artifact.path.clone());
    let input_bam = resolve_stage_input_path(&input_bam);
    let output_bam = stage_dir.join("overlap.corrected.bam");
    if !output_bam.exists() {
        return Err(anyhow!(
            "bam.overlap_correction hard failure: missing output {}",
            output_bam.display()
        ));
    }
    let summary = summarize_stage_overlap_correction_outputs(
        stage_dir,
        &input_bam,
        &output_bam,
        plan.tool_id.as_str(),
    )?;
    let path = stage_dir.join("overlap_correction.summary.json");
    bijux_dna_infra::atomic_write_json(&path, &summary)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

fn materialize_local_overlap_correction_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalOverlapCorrectionSmokeCasePlan,
    output_root: &Path,
) -> Result<LocalOverlapCorrectionSmokeReport> {
    let case_out_dir = resolve_plan_path(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let corrected_bam_path = resolve_output_path(repo_root, &case.plan, "overlap_corrected_bam")?;
    let corrected_bai_path = resolve_output_path(repo_root, &case.plan, "overlap_corrected_bai")?;
    let flagstat_before_path = resolve_output_path(repo_root, &case.plan, "flagstat_before")?;
    let flagstat_after_path = resolve_output_path(repo_root, &case.plan, "flagstat_after")?;
    let idxstats_before_path = resolve_output_path(repo_root, &case.plan, "idxstats_before")?;
    let idxstats_after_path = resolve_output_path(repo_root, &case.plan, "idxstats_after")?;
    let summary_path = resolve_output_path(repo_root, &case.plan, "summary")?;
    let stage_metrics_path = resolve_output_path(repo_root, &case.plan, "stage_metrics")?;

    let input_bam = repo_root.join(&case.bam);
    let input_qc_pre = bijux_dna_domain_bam::summarize_tiny_bam_qc_pre(&input_bam)?;
    let mut summary = bijux_dna_domain_bam::correct_tiny_bam_overlaps(
        &input_bam,
        &corrected_bam_path,
        case.plan.tool_id.as_str(),
    )?;
    let output_qc_pre = bijux_dna_domain_bam::summarize_tiny_bam_qc_pre(&corrected_bam_path)?;

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
    bijux_dna_infra::atomic_write_bytes(&corrected_bai_path, b"tiny-index\n")?;

    let _summary_path = write_stage_overlap_correction_summary(&case_out_dir, &case.plan)?;
    let mut written_summary: bijux_dna_domain_bam::BamOverlapCorrectionSummaryV1 =
        serde_json::from_str(
            &std::fs::read_to_string(&summary_path)
                .with_context(|| format!("read {}", summary_path.display()))?,
        )
        .with_context(|| format!("parse {}", summary_path.display()))?;
    written_summary.input_bam = relative_path(repo_root, &written_summary.input_bam);
    written_summary.output_bam = relative_path(repo_root, &written_summary.output_bam);
    bijux_dna_infra::atomic_write_json(&summary_path, &written_summary)?;

    let expectation_matched = written_summary.pair_count == Some(case.expected_pair_count)
        && written_summary.corrected_pairs == Some(case.expected_corrected_pairs)
        && written_summary.corrected_overlap_bases == Some(case.expected_corrected_overlap_bases)
        && written_summary.insufficiency_reason.is_none();
    let observed_pair_count = written_summary.pair_count.unwrap_or(0);
    let observed_corrected_pairs = written_summary.corrected_pairs.unwrap_or(0);
    let observed_corrected_overlap_bases = written_summary.corrected_overlap_bases.unwrap_or(0);
    let pair_count_delta = i64::try_from(observed_pair_count).unwrap_or(i64::MAX)
        - i64::try_from(case.expected_pair_count).unwrap_or(i64::MAX);
    let corrected_pair_delta = i64::try_from(observed_corrected_pairs).unwrap_or(i64::MAX)
        - i64::try_from(case.expected_corrected_pairs).unwrap_or(i64::MAX);
    let corrected_overlap_base_delta = i64::try_from(observed_corrected_overlap_bases)
        .unwrap_or(i64::MAX)
        - i64::try_from(case.expected_corrected_overlap_bases).unwrap_or(i64::MAX);

    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": LOCAL_OVERLAP_CORRECTION_SMOKE_METRICS_SCHEMA_VERSION,
            "stage_id": OVERLAP_CORRECTION_STAGE_ID,
            "sample_id": case.sample_id,
            "method": case.plan.tool_id.as_str(),
            "expected_pair_count": case.expected_pair_count,
            "pair_count": written_summary.pair_count,
            "pair_count_delta": pair_count_delta,
            "expected_corrected_pairs": case.expected_corrected_pairs,
            "corrected_pairs": written_summary.corrected_pairs,
            "corrected_pair_delta": corrected_pair_delta,
            "expected_corrected_overlap_bases": case.expected_corrected_overlap_bases,
            "corrected_overlap_bases": written_summary.corrected_overlap_bases,
            "corrected_overlap_base_delta": corrected_overlap_base_delta,
            "insufficiency_reason": written_summary.insufficiency_reason,
            "expectation_matched": expectation_matched,
        }),
    )?;

    let top_level_corrected_bam = output_root.join("overlap_corrected.bam");
    let top_level_corrected_bai = output_root.join("overlap_corrected.bam.bai");
    bijux_dna_infra::atomic_write_bytes(
        &top_level_corrected_bam,
        &std::fs::read(&corrected_bam_path)?,
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &top_level_corrected_bai,
        &std::fs::read(&corrected_bai_path)?,
    )?;

    summary.input_bam = relative_path(repo_root, &summary.input_bam);
    summary.output_bam = relative_path(repo_root, &summary.output_bam);

    Ok(LocalOverlapCorrectionSmokeReport {
        schema_version: LOCAL_OVERLAP_CORRECTION_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: OVERLAP_CORRECTION_STAGE_ID.to_string(),
        sample_id: case.sample_id.clone(),
        expectation_matched,
        input_bam: path_relative_to_repo(repo_root, &input_bam),
        overlap_corrected_bam: path_relative_to_repo(repo_root, &top_level_corrected_bam),
        method: case.plan.tool_id.as_str().to_string(),
        pair_count: written_summary.pair_count.unwrap_or(0),
        corrected_pairs: written_summary.corrected_pairs.unwrap_or(0),
        corrected_overlap_bases: written_summary.corrected_overlap_bases.unwrap_or(0),
        insufficiency_reason: written_summary.insufficiency_reason.clone(),
        overlap_correction_summary: path_relative_to_repo(repo_root, &summary_path),
        flagstat_before: path_relative_to_repo(repo_root, &flagstat_before_path),
        flagstat_after: path_relative_to_repo(repo_root, &flagstat_after_path),
        idxstats_before: path_relative_to_repo(repo_root, &idxstats_before_path),
        idxstats_after: path_relative_to_repo(repo_root, &idxstats_after_path),
        stage_metrics: path_relative_to_repo(repo_root, &stage_metrics_path),
    })
}

fn summarize_stage_overlap_correction_outputs(
    stage_dir: &Path,
    input_bam: &Path,
    output_bam: &Path,
    method: &str,
) -> Result<bijux_dna_domain_bam::BamOverlapCorrectionSummaryV1> {
    if let Ok(summary) = bijux_dna_domain_bam::summarize_tiny_bam_overlap_correction_outputs(
        input_bam, output_bam, method,
    ) {
        return Ok(summary);
    }

    let flagstat_before = parse_flagstat(stage_dir.join("flagstat.before.txt"))?;
    let flagstat_after = parse_flagstat(stage_dir.join("flagstat.after.txt"))?;
    Ok(bijux_dna_domain_bam::summarize_bam_overlap_correction(
        OVERLAP_CORRECTION_STAGE_ID,
        method,
        input_bam,
        output_bam,
        flagstat_before,
        flagstat_after,
        None,
        None,
        None,
        Some(OVERLAP_CORRECTION_PAIR_METRICS_UNAVAILABLE),
    ))
}

fn parse_flagstat(path: PathBuf) -> Result<bijux_dna_domain_bam::BamFlagstatCountsV1> {
    let parsed = bijux_dna_domain_bam::metrics::parse_samtools_flagstat(&path)
        .with_context(|| format!("parse {}", path.display()))?;
    let mapped_fraction =
        if parsed.total > 0 { Some(parsed.mapped as f64 / parsed.total as f64) } else { None };
    Ok(bijux_dna_domain_bam::BamFlagstatCountsV1 {
        total_reads: Some(parsed.total),
        mapped_reads: Some(parsed.mapped),
        duplicate_reads: Some(parsed.duplicates),
        mapped_fraction,
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
            anyhow!(
                "bam.overlap_correction local-smoke plan is missing governed output `{output_id}`"
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

fn resolve_stage_input_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    if let Ok(repo_root) = crate::support::workspace::resolve_repo_root() {
        let candidate = repo_root.join(path);
        if candidate.exists() {
            return candidate;
        }
    }
    path.to_path_buf()
}
