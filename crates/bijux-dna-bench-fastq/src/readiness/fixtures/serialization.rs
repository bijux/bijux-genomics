use super::super::paths::sanitize_tsv;
use super::model::{FastqParserFixtureCoverageRow, FastqParserFixtureCoverageStatus};
use std::fmt::Write as _;

/// Evaluate admitted FASTQ readiness inputs.
///
/// # Errors
/// Rejects invalid identities, incompatible evidence or failed publication.
#[must_use]
pub fn coverage_status_label(status: FastqParserFixtureCoverageStatus) -> &'static str {
    match status {
        FastqParserFixtureCoverageStatus::Covered => "covered",
        FastqParserFixtureCoverageStatus::MissingFixtureBinding => "missing_fixture_binding",
        FastqParserFixtureCoverageStatus::MissingFixtureCase => "missing_fixture_case",
        FastqParserFixtureCoverageStatus::InvalidFixtureCase => "invalid_fixture_case",
    }
}

pub(super) fn render_fastq_parser_fixture_coverage_tsv(
    rows: &[FastqParserFixtureCoverageRow],
) -> String {
    let mut rendered = String::from(
        "stage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tparser_fixture_parser_id\tparser_fixture_schema_id\tparser_fixture_reference_kind\tparser_fixture_reference\tparser_fixture_surface\tparser_fixture_canonical_tool_id\tcoverage_status\treason\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.corpus_id),
            sanitize_tsv(&row.asset_profile_id),
            sanitize_tsv(&row.adapter_id),
            sanitize_tsv(&row.parser_id),
            sanitize_tsv(&row.schema_id),
            sanitize_tsv(&row.parser_fixture_parser_id),
            sanitize_tsv(&row.parser_fixture_schema_id),
            sanitize_tsv(&row.parser_fixture_reference_kind),
            sanitize_tsv(&row.parser_fixture_reference),
            sanitize_tsv(&row.parser_fixture_surface),
            sanitize_tsv(&row.parser_fixture_canonical_tool_id),
            sanitize_tsv(coverage_status_label(row.coverage_status)),
            sanitize_tsv(&row.reason),
        );
    }
    rendered
}
