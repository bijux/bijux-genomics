use super::super::paths::sanitize_tsv;
use super::model::{
    FastqAdapterCoverageKind, FastqBenchmarkStatus, FastqCommandAdapterCoverageRow,
    FastqReadinessGapKind,
};
use std::fmt::Write as _;

pub(super) fn render_fastq_command_adapter_coverage_tsv(
    rows: &[FastqCommandAdapterCoverageRow],
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
            sanitize_tsv(benchmark_status_label(row.benchmark_status)),
            sanitize_tsv(adapter_coverage_label(row.adapter_coverage)),
            sanitize_tsv(readiness_gap_label(row.readiness_gap)),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.corpus_status),
            sanitize_tsv(&row.reason),
        );
    }
    rendered
}

pub(super) fn benchmark_status_label(status: FastqBenchmarkStatus) -> &'static str {
    status.as_str()
}

pub(super) fn adapter_coverage_label(coverage: FastqAdapterCoverageKind) -> &'static str {
    coverage.as_str()
}

pub(super) fn readiness_gap_label(gap: FastqReadinessGapKind) -> &'static str {
    gap.as_str()
}
