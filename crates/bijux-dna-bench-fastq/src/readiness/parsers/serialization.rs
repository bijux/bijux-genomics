use super::super::paths::sanitize_tsv;
use super::model::{FastqParserCoverageKind, FastqParserCoverageRow};
use std::fmt::Write as _;

pub(super) fn benchmark_scope_label(value: &str) -> &'static str {
    if value.starts_with("fixture:") {
        "fixture-backed"
    } else if value.starts_with("asset:") {
        "asset-backed"
    } else {
        "ungoverned"
    }
}

pub(super) fn render_fastq_parser_coverage_tsv(rows: &[FastqParserCoverageRow]) -> String {
    let mut rendered = String::from(
        "tool_id\tstage_id\tparser_coverage\tparser_status\tsupport_status\tadapter_status\tcorpus_status\treason\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(parser_coverage_label(row.parser_coverage)),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.corpus_status),
            sanitize_tsv(&row.reason),
        );
    }
    rendered
}

pub(super) fn parser_coverage_label(status: FastqParserCoverageKind) -> &'static str {
    status.as_str()
}
