use super::super::paths::{path_relative_to_repo, repo_relative_path};
use super::model::{FastqParserFixtureCoverageReport, FastqParserFixtureCoverageStatus};
use super::serialization::coverage_status_label;
use super::serialization::render_fastq_parser_fixture_coverage_tsv;
use super::FASTQ_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION;
use anyhow::{anyhow, Context, Result};
use std::collections::BTreeMap;
use std::path::Path;

/// Build or publish admitted FASTQ readiness evidence.
///
/// # Errors
/// Returns an error for invalid inputs, incompatible evidence or publication failure.
pub fn render_fastq_parser_fixture_coverage(
    repo_root: &Path,
    output_path: &Path,
    input: Vec<super::super::active::FastqActiveStageToolMatrixRow>,
) -> Result<FastqParserFixtureCoverageReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let collection = super::projection::collect_fastq_parser_fixture_coverage_rows(input)?;
    let (stage_count, tool_count, rows) = collection;

    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_fastq_parser_fixture_coverage_tsv(&rows).as_bytes(),
    )
    .with_context(|| format!("write {}", output_path.display()))?;

    let covered_row_count = rows
        .iter()
        .filter(|row| row.coverage_status == FastqParserFixtureCoverageStatus::Covered)
        .count();
    let missing_row_count = rows.len().saturating_sub(covered_row_count);
    let parser_fixture_coverage_percent =
        super::super::policy::coverage_percent(covered_row_count, rows.len())?;
    let mut coverage_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *coverage_status_counts
            .entry(coverage_status_label(row.coverage_status).to_string())
            .or_default() += 1;
    }

    if missing_row_count != 0 {
        let missing_rows = rows
            .iter()
            .filter(|row| row.coverage_status != FastqParserFixtureCoverageStatus::Covered)
            .map(|row| {
                format!(
                    "{}:{}:{}",
                    row.stage_id,
                    row.tool_id,
                    coverage_status_label(row.coverage_status)
                )
            })
            .collect::<Vec<_>>();
        return Err(anyhow!(
            "FASTQ parser fixture coverage must be complete for every active FASTQ row, missing coverage for: {}",
            missing_rows.join(", ")
        ));
    }

    Ok(FastqParserFixtureCoverageReport {
        schema_version: FASTQ_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        tool_count,
        row_count: rows.len(),
        covered_row_count,
        missing_row_count,
        parser_fixture_coverage_percent,
        coverage_status_counts,
        rows,
    })
}
