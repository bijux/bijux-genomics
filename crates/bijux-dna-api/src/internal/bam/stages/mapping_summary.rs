use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

const LOCAL_MAPPING_SUMMARY_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.mapping_summary.local_smoke.metrics.v1";

/// Materialize the governed local-smoke `bam.mapping_summary` artifacts and TSV summary.
///
/// The written summary artifact lives at `runs/bench/local-smoke/bam.mapping_summary/mapping_summary.tsv`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_mapping_summary_smoke_summary() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_mapping_summary_smoke_plans(&repo_root)?;
    let output_root = repo_root.join("runs/bench/local-smoke/bam.mapping_summary");
    bijux_dna_infra::ensure_dir(&output_root)?;

    let mut body = String::from(
        "sample_id\ttotal_reads\tmapped_reads\tunmapped_reads\tmapping_fraction\tsecondary_reads\tsupplementary_reads\treference_name\texpectation_matched\tmapping_summary_json\tflagstat\tidxstats\tstats\tstage_metrics\n",
    );
    for case in &cases {
        let row = materialize_local_mapping_summary_smoke_case(&repo_root, case)?;
        writeln!(
            body,
            "{sample_id}\t{total_reads}\t{mapped_reads}\t{unmapped_reads}\t{mapping_fraction:.6}\t{secondary_reads}\t{supplementary_reads}\t{reference_name}\t{expectation_matched}\t{mapping_summary_json}\t{flagstat}\t{idxstats}\t{stats}\t{stage_metrics}",
            sample_id = row.sample_id,
            total_reads = row.total_reads,
            mapped_reads = row.mapped_reads,
            unmapped_reads = row.unmapped_reads,
            mapping_fraction = row.mapping_fraction,
            secondary_reads = row.secondary_reads,
            supplementary_reads = row.supplementary_reads,
            reference_name = row.reference_name,
            expectation_matched = row.expectation_matched,
            mapping_summary_json = row.mapping_summary_json,
            flagstat = row.flagstat,
            idxstats = row.idxstats,
            stats = row.stats,
            stage_metrics = row.stage_metrics,
        )
        .map_err(|error| anyhow!("write bam.mapping_summary local-smoke TSV row: {error}"))?;
    }

    let summary_path = output_root.join("mapping_summary.tsv");
    bijux_dna_infra::atomic_write_bytes(&summary_path, body.as_bytes())?;
    Ok(summary_path)
}

struct LocalMappingSummarySmokeRow {
    sample_id: String,
    total_reads: u64,
    mapped_reads: u64,
    unmapped_reads: u64,
    mapping_fraction: f64,
    secondary_reads: u64,
    supplementary_reads: u64,
    reference_name: String,
    expectation_matched: bool,
    mapping_summary_json: String,
    flagstat: String,
    idxstats: String,
    stats: String,
    stage_metrics: String,
}

fn materialize_local_mapping_summary_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalMappingSummarySmokeCasePlan,
) -> Result<LocalMappingSummarySmokeRow> {
    let case_out_dir = resolve_plan_dir(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let bam = repo_root.join(&case.bam);
    let mapping_summary = bijux_dna_domain_bam::summarize_tiny_bam_mapping(&bam)?;
    let qc_pre_summary = bijux_dna_domain_bam::summarize_tiny_bam_qc_pre(&bam)?;

    let total_reads = mapping_summary.flagstat.total_reads.unwrap_or(0);
    let mapped_reads = mapping_summary.flagstat.mapped_reads.unwrap_or(0);
    let unmapped_reads = qc_pre_summary.unmapped_reads;
    let mapping_fraction = derived_mapping_fraction(total_reads, mapped_reads);
    let secondary_reads = mapping_summary.secondary_reads.unwrap_or(0);
    let supplementary_reads = mapping_summary.supplementary_reads.unwrap_or(0);
    let reference_name = qc_pre_summary
        .contig_summary
        .iter()
        .find(|contig| contig.mapped > 0)
        .or_else(|| qc_pre_summary.contig_summary.first())
        .map(|contig| contig.contig.clone())
        .ok_or_else(|| {
            anyhow!(
                "bam.mapping_summary local-smoke case `{}` has no governed contig summary",
                case.sample_id
            )
        })?;
    let expectation_matched = total_reads == case.expected_total_reads
        && mapped_reads == case.expected_mapped_reads
        && (mapping_fraction - case.expected_mapping_fraction).abs() <= 1e-9
        && reference_name == case.expected_reference_name;

    let flagstat_path = resolve_output_path(repo_root, &case.plan, "flagstat")?;
    let idxstats_path = resolve_output_path(repo_root, &case.plan, "idxstats")?;
    let stats_path = resolve_output_path(repo_root, &case.plan, "stats")?;
    let summary_path = resolve_output_path(repo_root, &case.plan, "summary")?;
    let stage_metrics_path = resolve_output_path(repo_root, &case.plan, "stage_metrics")?;

    bijux_dna_infra::atomic_write_bytes(
        &flagstat_path,
        render_flagstat(total_reads, mapped_reads).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &idxstats_path,
        render_idxstats(&qc_pre_summary).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &stats_path,
        render_stats(total_reads, mapped_reads, &mapping_summary).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_json(&summary_path, &mapping_summary)?;
    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": LOCAL_MAPPING_SUMMARY_SMOKE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.mapping_summary",
            "sample_id": case.sample_id,
            "total_reads": total_reads,
            "mapped_reads": mapped_reads,
            "unmapped_reads": unmapped_reads,
            "mapping_fraction": mapping_fraction,
            "reference_name": reference_name,
            "expectation_matched": expectation_matched,
            "proper_pair_reads": mapping_summary.proper_pair_reads,
            "secondary_reads": secondary_reads,
            "supplementary_reads": supplementary_reads,
            "read_group_ids": mapping_summary
                .read_group_breakdown
                .iter()
                .map(|entry| entry.read_group_id.clone())
                .collect::<Vec<_>>(),
        }),
    )?;

    Ok(LocalMappingSummarySmokeRow {
        sample_id: case.sample_id.clone(),
        total_reads,
        mapped_reads,
        unmapped_reads,
        mapping_fraction,
        secondary_reads,
        supplementary_reads,
        reference_name,
        expectation_matched,
        mapping_summary_json: path_relative_to_repo(repo_root, &summary_path),
        flagstat: path_relative_to_repo(repo_root, &flagstat_path),
        idxstats: path_relative_to_repo(repo_root, &idxstats_path),
        stats: path_relative_to_repo(repo_root, &stats_path),
        stage_metrics: path_relative_to_repo(repo_root, &stage_metrics_path),
    })
}

#[allow(clippy::cast_precision_loss)]
fn derived_mapping_fraction(total_reads: u64, mapped_reads: u64) -> f64 {
    if total_reads == 0 {
        0.0
    } else {
        mapped_reads as f64 / total_reads as f64
    }
}

fn render_flagstat(total_reads: u64, mapped_reads: u64) -> String {
    let mapped_fraction = if total_reads > 0 {
        format!("{:.2}%", derived_mapping_fraction(total_reads, mapped_reads) * 100.0)
    } else {
        "N/A".to_string()
    };
    format!(
        "{total_reads} + 0 in total (QC-passed reads + QC-failed reads)\n\
{mapped_reads} + 0 mapped ({mapped_fraction} : N/A)\n",
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

fn render_stats(
    total_reads: u64,
    mapped_reads: u64,
    summary: &bijux_dna_domain_bam::BamMappingSummaryV1,
) -> String {
    let mut payload = String::new();
    let _ = writeln!(payload, "SN\traw total sequences:\t{total_reads}");
    let _ = writeln!(payload, "SN\treads mapped:\t{mapped_reads}");
    for (mapq, count) in &summary.mapq_histogram {
        let _ = writeln!(payload, "MQ\t{mapq}\t{count}");
    }
    payload
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
            anyhow!("bam.mapping_summary local-smoke plan is missing governed output `{output_id}`")
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
