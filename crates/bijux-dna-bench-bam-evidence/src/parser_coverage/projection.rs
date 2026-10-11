use super::declarations::FIXTURE_REFERENCE_KIND_CORPUS;
use super::models::{BamParserFixtureCoverageRow, BamParserFixtureCoverageStatus};
pub(super) fn render_parser_fixture_coverage_row(
    row: crate::command_coverage::BamCommandAdapterCoverageRow,
) -> BamParserFixtureCoverageRow {
    let coverage_status = if row.parser_status == "parser_fixture_validated" {
        BamParserFixtureCoverageStatus::Covered
    } else {
        BamParserFixtureCoverageStatus::Missing
    };
    let reason = match coverage_status {
        BamParserFixtureCoverageStatus::Covered => format!(
            "row `{}` / `{}` is benchmark_ready with governed support, adapter-backed command rendering, fixture-backed corpus coverage, and parser-fixture-validated output",
            row.stage_id, row.tool_id
        ),
        BamParserFixtureCoverageStatus::Missing => format!(
            "row `{}` / `{}` is benchmark_ready but lacks parser-fixture-validated BAM normalization (`{}`)",
            row.stage_id, row.tool_id, row.parser_status
        ),
    };

    BamParserFixtureCoverageRow {
        tool_id: row.tool_id,
        stage_id: row.stage_id,
        parser_fixture_reference_kind: FIXTURE_REFERENCE_KIND_CORPUS.to_string(),
        parser_fixture_reference: row.corpus_status,
        coverage_status,
        parser_status: row.parser_status,
        support_status: row.support_status,
        adapter_status: row.adapter_status,
        reason,
    }
}
