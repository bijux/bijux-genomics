use super::models::BamCommandAdapterCoverageRow;
use crate::publication::sanitize_tsv;
use std::fmt::Write;
pub(super) fn render_bam_command_adapter_coverage_tsv(
    rows: &[BamCommandAdapterCoverageRow],
) -> String {
    let mut rendered = String::from(
        "tool_id\tstage_id\tbenchmark_status\tadapter_coverage\treadiness_gap\tsupport_status\tadapter_status\tparser_status\tcorpus_status\treason\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(row.benchmark_status.as_str()),
            sanitize_tsv(row.adapter_coverage.as_str()),
            sanitize_tsv(row.readiness_gap.as_str()),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.corpus_status),
            sanitize_tsv(&row.reason),
        );
    }
    rendered
}
