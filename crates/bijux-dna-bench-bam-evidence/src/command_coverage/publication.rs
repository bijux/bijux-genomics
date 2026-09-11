use super::models::{
    BamAdapterCoverageKind, BamBenchmarkStatus, BamCommandAdapterCoverageReport,
    BamReadinessGapKind,
};
use super::{
    declarations::BAM_COMMAND_ADAPTER_COVERAGE_SCHEMA_VERSION,
    service::collect_bam_command_adapter_coverage_rows,
    tsv::render_bam_command_adapter_coverage_tsv,
};
use crate::publication::{path_relative_to_repo, repo_relative_path};
use crate::EvidenceContext;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
/// Build the declared BAM evidence view from explicit canonical observations.
///
/// # Errors
/// Returns acquisition, contract, or publication failures with source diagnostics.
pub fn render_bam_command_adapter_coverage(
    context: &EvidenceContext<'_>,
    output_path: &Path,
) -> Result<BamCommandAdapterCoverageReport> {
    let repo_root = context.repo_root;
    let output_path = repo_relative_path(repo_root, output_path);
    let (stage_count, tool_count, rows) = collect_bam_command_adapter_coverage_rows(context)?;

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_bam_command_adapter_coverage_tsv(&rows).as_bytes(),
    )?;

    let benchmark_ready_row_count = rows
        .iter()
        .filter(|row| row.benchmark_status == BamBenchmarkStatus::BenchmarkReady)
        .count();
    let benchmark_ready_adapter_covered_row_count = rows
        .iter()
        .filter(|row| {
            row.benchmark_status == BamBenchmarkStatus::BenchmarkReady
                && row.adapter_coverage == BamAdapterCoverageKind::Covered
        })
        .count();
    let benchmark_ready_adapter_missing_row_count = rows
        .iter()
        .filter(|row| {
            row.benchmark_status == BamBenchmarkStatus::BenchmarkReady
                && row.adapter_coverage == BamAdapterCoverageKind::Missing
        })
        .count();

    let mut readiness_gap_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        if row.readiness_gap != BamReadinessGapKind::None {
            *readiness_gap_counts.entry(row.readiness_gap.as_str().to_string()).or_default() += 1;
        }
    }

    Ok(BamCommandAdapterCoverageReport {
        schema_version: BAM_COMMAND_ADAPTER_COVERAGE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        tool_count,
        row_count: rows.len(),
        benchmark_ready_row_count,
        benchmark_ready_adapter_covered_row_count,
        benchmark_ready_adapter_missing_row_count,
        readiness_gap_counts,
        rows,
    })
}
