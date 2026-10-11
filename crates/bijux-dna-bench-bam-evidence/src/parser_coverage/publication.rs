use super::models::{BamParserFixtureCoverageReport, BamParserFixtureCoverageStatus};
use super::{
    declarations::BAM_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION,
    service::collect_bam_parser_fixture_coverage_rows, tsv::render_bam_parser_fixture_coverage_tsv,
};
use crate::publication::{path_relative_to_repo, repo_relative_path};
use crate::EvidenceContext;
use anyhow::{anyhow, Result};
use std::path::Path;
/// Build the declared BAM evidence view from explicit canonical observations.
///
/// # Errors
/// Returns acquisition, contract, or publication failures with source diagnostics.
pub fn render_bam_parser_fixture_coverage(
    context: &EvidenceContext<'_>,
    output_path: &Path,
) -> Result<BamParserFixtureCoverageReport> {
    let repo_root = context.repo_root;
    let output_path = repo_relative_path(repo_root, output_path);
    let super::service::ParserCoverageRows {
        stage_count,
        tool_count,
        rows,
        excluded_readiness_gap_counts,
    } = collect_bam_parser_fixture_coverage_rows(context)?;

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_bam_parser_fixture_coverage_tsv(&rows).as_bytes(),
    )?;

    let covered_row_count = rows
        .iter()
        .filter(|row| row.coverage_status == BamParserFixtureCoverageStatus::Covered)
        .count();
    let missing_row_count = rows.len().saturating_sub(covered_row_count);
    let parser_fixture_coverage_percent = coverage_percent(covered_row_count, rows.len());
    let excluded_non_benchmark_ready_row_count = excluded_readiness_gap_counts.values().sum();
    if missing_row_count != 0 {
        let missing_rows = rows
            .iter()
            .filter(|row| row.coverage_status != BamParserFixtureCoverageStatus::Covered)
            .map(|row| format!("{}:{}:{}", row.stage_id, row.tool_id, row.coverage_status.as_str()))
            .collect::<Vec<_>>();
        return Err(anyhow!(
            "BAM parser fixture coverage must be complete for every active BAM row, missing coverage for: {}",
            missing_rows.join(", ")
        ));
    }

    Ok(BamParserFixtureCoverageReport {
        schema_version: BAM_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        tool_count,
        row_count: rows.len(),
        covered_row_count,
        missing_row_count,
        parser_fixture_coverage_percent,
        excluded_non_benchmark_ready_row_count,
        excluded_readiness_gap_counts,
        rows,
    })
}

// Percentages are approximate display values; the exact integer counts remain
// the report authority even when a count exceeds the floating-point mantissa.
#[allow(clippy::cast_precision_loss)]
fn coverage_percent(covered: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        covered as f64 * 100.0 / total as f64
    }
}
