use super::models::BamParserFixtureCoverageRow;
use super::projection::render_parser_fixture_coverage_row;
use crate::command_coverage::BamBenchmarkStatus;
use crate::EvidenceContext;
use anyhow::Result;
use std::collections::BTreeMap;
/// Build the declared BAM evidence view from explicit canonical observations.
///
/// # Errors
/// Returns acquisition, contract, or publication failures with source diagnostics.
pub fn collect_bam_parser_fixture_coverage_rows(
    context: &EvidenceContext<'_>,
) -> Result<ParserCoverageRows> {
    let (stage_count, tool_count, rows) =
        crate::command_coverage::collect_bam_command_adapter_coverage_rows(context)?;
    let mut excluded_readiness_gap_counts = BTreeMap::<String, usize>::new();
    let mut parser_rows = Vec::new();

    for row in rows {
        if row.benchmark_status == BamBenchmarkStatus::BenchmarkReady {
            parser_rows.push(render_parser_fixture_coverage_row(row));
        } else {
            *excluded_readiness_gap_counts
                .entry(row.readiness_gap.as_str().to_string())
                .or_default() += 1;
        }
    }

    Ok(ParserCoverageRows {
        stage_count,
        tool_count,
        rows: parser_rows,
        excluded_readiness_gap_counts,
    })
}

/// Collected parser declaration observations and exclusions, before publication.
pub struct ParserCoverageRows {
    pub stage_count: usize,
    pub tool_count: usize,
    pub rows: Vec<BamParserFixtureCoverageRow>,
    pub excluded_readiness_gap_counts: BTreeMap<String, usize>,
}
