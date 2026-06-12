use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_domain_bam::params::MarkDupEffectiveParams;
use serde::Serialize;

const LOCAL_MARKDUP_SMOKE_REPORT_SCHEMA_VERSION: &str = "bijux.bam.markdup.local_smoke.report.v1";
const LOCAL_MARKDUP_SMOKE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.markdup.local_smoke.metrics.v1";

#[derive(Debug, Clone, Serialize)]
struct LocalMarkdupSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    marked_bam: String,
    marked_bai: String,
    duplicate_metrics: String,
    duplicate_action: String,
    input_reads: u64,
    output_reads: u64,
    removed_reads: u64,
    duplicate_count: u64,
    duplicate_fraction: f64,
    duplicate_reads_before: Option<u64>,
    duplicate_reads_after: Option<u64>,
    newly_marked_reads: Option<u64>,
    markdup_summary: String,
    markdup_policy: String,
    markdup_comparison: String,
    flagstat_before: String,
    flagstat_after: String,
    idxstats_before: String,
    idxstats_after: String,
    stage_metrics: String,
}

/// Materialize the governed local-smoke `bam.markdup` artifacts and top-level report.
///
/// The written report lives at `runs/bench/local-smoke/bam.markdup/duplicates.json`
/// under the active repository root, alongside the curated top-level `marked.bam`.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_markdup_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_markdup_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "local-smoke bam.markdup expects exactly one governed case, found {}",
            cases.len()
        ));
    };

    let output_root = repo_root.join("runs/bench/local-smoke/bam.markdup");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let report = materialize_local_markdup_smoke_case(&repo_root, case, &output_root)?;
    let report_path = output_root.join("duplicates.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(report_path)
}

#[allow(clippy::too_many_lines)]
fn materialize_local_markdup_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalMarkdupSmokeCasePlan,
    output_root: &Path,
) -> Result<LocalMarkdupSmokeReport> {
    let case_out_dir = resolve_plan_path(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let marked_bam_path = resolve_output_path(repo_root, &case.plan, "markdup_bam")?;
    let marked_bai_path = resolve_output_path(repo_root, &case.plan, "markdup_bai")?;
    let flagstat_before_path = resolve_output_path(repo_root, &case.plan, "flagstat_before")?;
    let flagstat_after_path = resolve_output_path(repo_root, &case.plan, "flagstat_after")?;
    let idxstats_before_path = resolve_output_path(repo_root, &case.plan, "idxstats_before")?;
    let idxstats_after_path = resolve_output_path(repo_root, &case.plan, "idxstats_after")?;
    let summary_path = resolve_output_path(repo_root, &case.plan, "summary")?;
    let stage_metrics_path = resolve_output_path(repo_root, &case.plan, "stage_metrics")?;

    let input_bam = repo_root.join(&case.bam);
    let params =
        serde_json::from_value::<MarkDupEffectiveParams>(case.plan.effective_params.clone())?;
    let duplicate_action =
        serde_json::to_value(params.duplicate_action)?.as_str().unwrap_or("mark").to_string();
    let optical_duplicates =
        serde_json::to_value(params.optical_duplicates)?.as_str().map(ToOwned::to_owned);
    let umi_policy = serde_json::to_value(params.umi_policy)?.as_str().map(ToOwned::to_owned);

    let (policy, comparison) = bijux_dna_domain_bam::apply_duplicate_policy_tiny_bam(
        &input_bam,
        &marked_bam_path,
        &duplicate_action,
        umi_policy.as_deref(),
    )?;
    let input_qc_pre = bijux_dna_domain_bam::summarize_tiny_bam_qc_pre(&input_bam)?;
    let output_qc_pre = bijux_dna_domain_bam::summarize_tiny_bam_qc_pre(&marked_bam_path)?;
    let flagstat_before = flagstat_from_qc_pre(&input_qc_pre);
    let flagstat_after = flagstat_from_qc_pre(&output_qc_pre);
    let mut summary = bijux_dna_domain_bam::summarize_bam_markdup(
        "bam.markdup",
        &input_bam,
        &marked_bam_path,
        &duplicate_action,
        optical_duplicates.as_deref(),
        umi_policy.as_deref(),
        flagstat_before,
        flagstat_after,
    );
    summary.input_bam = relative_path(repo_root, &summary.input_bam);
    summary.output_bam = relative_path(repo_root, &summary.output_bam);

    let mut policy = policy;
    policy.optical_duplicates.clone_from(&optical_duplicates);
    policy.umi_policy.clone_from(&umi_policy);

    let comparison_path = case_out_dir.join("markdup.comparison.json");
    let policy_path = case_out_dir.join("markdup.policy.json");

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
    bijux_dna_infra::atomic_write_json(&summary_path, &summary)?;
    bijux_dna_infra::atomic_write_json(&policy_path, &policy)?;
    bijux_dna_infra::atomic_write_json(&comparison_path, &comparison)?;
    let duplicate_count = summary
        .duplicate_count
        .or(summary.duplicate_reads_after)
        .ok_or_else(|| anyhow!("bam.markdup local-smoke summary is missing duplicate_count"))?;
    let duplicate_fraction = summary
        .duplicate_fraction
        .or(summary.duplicate_fraction_after)
        .ok_or_else(|| anyhow!("bam.markdup local-smoke summary is missing duplicate_fraction"))?;
    let expectation_matched = summary.input_reads == case.expected_input_reads
        && summary.output_reads == case.expected_output_reads
        && summary.removed_reads == case.expected_removed_reads
        && summary.duplicate_reads_before == Some(case.expected_duplicate_reads_before)
        && summary.duplicate_reads_after == Some(case.expected_duplicate_reads_after)
        && (duplicate_fraction - case.expected_duplicate_fraction).abs() <= 1e-9
        && summary.newly_marked_reads == Some(case.expected_newly_marked_reads);
    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": LOCAL_MARKDUP_SMOKE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.markdup",
            "sample_id": case.sample_id,
            "duplicate_action": duplicate_action,
            "input_reads": summary.input_reads,
            "output_reads": summary.output_reads,
            "removed_reads": summary.removed_reads,
            "duplicate_count": duplicate_count,
            "duplicate_fraction": duplicate_fraction,
            "duplicate_reads_before": summary.duplicate_reads_before,
            "duplicate_reads_after": summary.duplicate_reads_after,
            "newly_marked_reads": summary.newly_marked_reads,
            "expectation_matched": expectation_matched,
        }),
    )?;
    bijux_dna_infra::atomic_write_bytes(&marked_bai_path, b"tiny-index\n")?;

    let top_level_marked_bam = output_root.join("marked.bam");
    let top_level_marked_index = output_root.join("marked.bam.bai");
    bijux_dna_infra::atomic_write_bytes(&top_level_marked_bam, &std::fs::read(&marked_bam_path)?)?;
    bijux_dna_infra::atomic_write_bytes(
        &top_level_marked_index,
        &std::fs::read(&marked_bai_path)?,
    )?;

    Ok(LocalMarkdupSmokeReport {
        schema_version: LOCAL_MARKDUP_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.markdup".to_string(),
        sample_id: case.sample_id.clone(),
        expectation_matched,
        input_bam: path_relative_to_repo(repo_root, &input_bam),
        marked_bam: path_relative_to_repo(repo_root, &top_level_marked_bam),
        marked_bai: path_relative_to_repo(repo_root, &top_level_marked_index),
        duplicate_metrics: path_relative_to_repo(repo_root, &summary_path),
        duplicate_action,
        input_reads: summary.input_reads,
        output_reads: summary.output_reads,
        removed_reads: summary.removed_reads,
        duplicate_count,
        duplicate_fraction,
        duplicate_reads_before: summary.duplicate_reads_before,
        duplicate_reads_after: summary.duplicate_reads_after,
        newly_marked_reads: summary.newly_marked_reads,
        markdup_summary: path_relative_to_repo(repo_root, &summary_path),
        markdup_policy: path_relative_to_repo(repo_root, &policy_path),
        markdup_comparison: path_relative_to_repo(repo_root, &comparison_path),
        flagstat_before: path_relative_to_repo(repo_root, &flagstat_before_path),
        flagstat_after: path_relative_to_repo(repo_root, &flagstat_after_path),
        idxstats_before: path_relative_to_repo(repo_root, &idxstats_before_path),
        idxstats_after: path_relative_to_repo(repo_root, &idxstats_after_path),
        stage_metrics: path_relative_to_repo(repo_root, &stage_metrics_path),
    })
}

#[allow(clippy::cast_precision_loss)]
fn flagstat_from_qc_pre(
    summary: &bijux_dna_domain_bam::BamQcPreSummaryV1,
) -> bijux_dna_domain_bam::BamFlagstatCountsV1 {
    let mapped_fraction = if summary.total_reads > 0 {
        Some(summary.mapped_reads as f64 / summary.total_reads as f64)
    } else {
        None
    };
    bijux_dna_domain_bam::BamFlagstatCountsV1 {
        total_reads: Some(summary.total_reads),
        mapped_reads: Some(summary.mapped_reads),
        duplicate_reads: Some(summary.duplicate_flagged_reads),
        mapped_fraction,
    }
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
            anyhow!("bam.markdup local-smoke plan is missing governed output `{output_id}`")
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
