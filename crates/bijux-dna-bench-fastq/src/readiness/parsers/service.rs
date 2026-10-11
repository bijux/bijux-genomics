use super::super::paths::{path_relative_to_repo, repo_relative_path};
use super::model::{FastqParserCoverageKind, FastqParserCoverageReport};
use super::serialization::render_fastq_parser_coverage_tsv;
use super::FASTQ_PARSER_COVERAGE_SCHEMA_VERSION;
use anyhow::{Context, Result};
use std::path::Path;

/// Build or publish admitted FASTQ readiness evidence.
///
/// # Errors
/// Returns an error for invalid inputs, incompatible evidence or publication failure.
pub fn render_fastq_parser_coverage(
    repo_root: &Path,
    output_path: &Path,
    input: super::super::inputs::FastqAdapterCoverage,
) -> Result<FastqParserCoverageReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let collection = super::projection::collect_fastq_parser_coverage_rows(input)?;
    let (stage_count, tool_count, rows) = collection;

    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_fastq_parser_coverage_tsv(&rows).as_bytes(),
    )
    .with_context(|| format!("write {}", output_path.display()))?;

    let parser_covered_row_count =
        rows.iter().filter(|row| row.parser_coverage == FastqParserCoverageKind::Covered).count();
    let parser_missing_row_count =
        rows.iter().filter(|row| row.parser_coverage == FastqParserCoverageKind::Missing).count();
    let parser_coverage_percent =
        super::super::policy::coverage_percent(parser_covered_row_count, rows.len())?;

    Ok(FastqParserCoverageReport {
        schema_version: FASTQ_PARSER_COVERAGE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        tool_count,
        row_count: rows.len(),
        parser_covered_row_count,
        parser_missing_row_count,
        parser_coverage_percent,
        rows,
    })
}
