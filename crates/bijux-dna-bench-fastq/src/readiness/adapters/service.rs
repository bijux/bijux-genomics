use super::super::paths::{path_relative_to_repo, repo_relative_path};
use super::model::{
    FastqAdapterCoverageKind, FastqBenchmarkStatus, FastqCommandAdapterCoverageReport,
    FastqReadinessGapKind,
};
use super::serialization::readiness_gap_label;
use super::serialization::render_fastq_command_adapter_coverage_tsv;
use super::FASTQ_COMMAND_ADAPTER_COVERAGE_SCHEMA_VERSION;
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::Path;

/// Build or publish admitted FASTQ readiness evidence.
///
/// # Errors
/// Returns an error for invalid inputs, incompatible evidence or publication failure.
pub fn render_fastq_command_adapter_coverage(
    repo_root: &Path,
    output_path: &Path,
    input: super::super::inputs::FastqServingSnapshot,
) -> Result<FastqCommandAdapterCoverageReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let collection = super::projection::collect_fastq_command_adapter_coverage_rows(input)?;
    let (stage_count, tool_count, rows) = collection;

    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_fastq_command_adapter_coverage_tsv(&rows).as_bytes(),
    )
    .with_context(|| format!("write {}", output_path.display()))?;

    let benchmark_ready_row_count = rows
        .iter()
        .filter(|row| row.benchmark_status == FastqBenchmarkStatus::BenchmarkReady)
        .count();
    let benchmark_ready_adapter_covered_row_count = rows
        .iter()
        .filter(|row| {
            row.benchmark_status == FastqBenchmarkStatus::BenchmarkReady
                && row.adapter_coverage == FastqAdapterCoverageKind::Covered
        })
        .count();
    let benchmark_ready_adapter_missing_row_count = rows
        .iter()
        .filter(|row| {
            row.benchmark_status == FastqBenchmarkStatus::BenchmarkReady
                && row.adapter_coverage == FastqAdapterCoverageKind::Missing
        })
        .count();

    let mut readiness_gap_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        if row.readiness_gap != FastqReadinessGapKind::None {
            *readiness_gap_counts
                .entry(readiness_gap_label(row.readiness_gap).to_string())
                .or_default() += 1;
        }
    }

    Ok(FastqCommandAdapterCoverageReport {
        schema_version: FASTQ_COMMAND_ADAPTER_COVERAGE_SCHEMA_VERSION,
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
