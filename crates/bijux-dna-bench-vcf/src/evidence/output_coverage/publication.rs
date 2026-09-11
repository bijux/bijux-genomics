use super::VCF_ADAPTER_OUTPUT_COVERAGE_SCHEMA_VERSION;
use crate::adapters::observations::VcfAdapterObservations;
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::model::outputs::{
    VcfAdapterOutputCoverageReport, VcfAdapterOutputCoverageRow, VcfAdapterOutputCoverageStatus,
};
use std::fmt::Write;
use std::path::{Path, PathBuf};
/// Publish compatible output coverage after all declaration and coverage contracts pass.
///
/// # Errors
/// Returns acquisition, contract or atomic publication errors.
pub fn render_vcf_adapter_output_coverage(
    observations: &VcfAdapterObservations<'_>,
    run_id: &str,
    output_path: &Path,
) -> Result<VcfAdapterOutputCoverageReport> {
    let repo_root = observations.repo_root();
    let output_path = repo_relative_path(repo_root, output_path);
    let rows = super::collect_vcf_adapter_output_coverage_rows(observations, run_id)?;

    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_vcf_adapter_output_coverage_tsv(&rows).as_bytes(),
    )?;

    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let benchmark_ready_complete_row_count = rows
        .iter()
        .filter(|row| {
            row.benchmark_status == "benchmark_ready"
                && row.status == VcfAdapterOutputCoverageStatus::Complete
        })
        .count();
    let benchmark_ready_incomplete_row_count =
        benchmark_ready_row_count.saturating_sub(benchmark_ready_complete_row_count);
    let complete_row_count =
        rows.iter().filter(|row| row.status == VcfAdapterOutputCoverageStatus::Complete).count();
    let incomplete_row_count = rows.len().saturating_sub(complete_row_count);

    Ok(VcfAdapterOutputCoverageReport {
        schema_version: VCF_ADAPTER_OUTPUT_COVERAGE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        benchmark_ready_row_count,
        benchmark_ready_complete_row_count,
        benchmark_ready_incomplete_row_count,
        complete_row_count,
        incomplete_row_count,
        rows,
    })
}

fn render_vcf_adapter_output_coverage_tsv(rows: &[VcfAdapterOutputCoverageRow]) -> String {
    let mut rendered = String::from(
        "stage_id\ttool_id\traw_outputs\tnormalized_metrics\tstdout\tstderr\tmanifest\tindex_outputs\tstatus\tbenchmark_status\tmissing_declarations\treason\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.raw_outputs.join(",")),
            sanitize_tsv(&row.normalized_metrics.join(",")),
            sanitize_tsv(&row.stdout),
            sanitize_tsv(&row.stderr),
            sanitize_tsv(&row.manifest),
            sanitize_tsv(&row.index_outputs.join(",")),
            sanitize_tsv(status_label(row.status)),
            sanitize_tsv(&row.benchmark_status),
            sanitize_tsv(&row.missing_declarations.join(",")),
            sanitize_tsv(&row.reason),
        );
    }
    rendered
}

fn status_label(status: VcfAdapterOutputCoverageStatus) -> &'static str {
    match status {
        VcfAdapterOutputCoverageStatus::Complete => "complete",
        VcfAdapterOutputCoverageStatus::Incomplete => "incomplete",
    }
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}
