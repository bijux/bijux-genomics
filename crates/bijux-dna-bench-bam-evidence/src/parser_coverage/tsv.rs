use super::models::BamParserFixtureCoverageRow;
use crate::publication::sanitize_tsv;
use std::fmt::Write;
pub(super) fn render_bam_parser_fixture_coverage_tsv(
    rows: &[BamParserFixtureCoverageRow],
) -> String {
    let mut rendered = String::from(
        "tool_id\tstage_id\tparser_fixture_reference_kind\tparser_fixture_reference\tcoverage_status\tparser_status\tsupport_status\tadapter_status\treason\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.parser_fixture_reference_kind),
            sanitize_tsv(&row.parser_fixture_reference),
            sanitize_tsv(row.coverage_status.as_str()),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.reason),
        );
    }
    rendered
}
