use crate::bam::assignment::models::BamCorpusAssignmentRow;
use std::fmt::Write;
pub(in crate::bam::assignment) fn render_bam_corpus_assignment_tsv(
    rows: &[BamCorpusAssignmentRow],
) -> String {
    let mut rendered = String::from(
        "tool_id\tstage_id\tbenchmark_status\tsupport_status\tadapter_status\tparser_status\tcorpus_family_id\tfixture_id\tsample_id\tinput_contract\tbenchmark_limits\trequired_assets\texpected_outputs\tskip_behavior\treason\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.benchmark_status),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.corpus_family_id),
            sanitize_tsv(&row.fixture_id),
            sanitize_tsv(&row.sample_id),
            sanitize_tsv(&row.input_contract),
            sanitize_tsv(&row.benchmark_limits),
            sanitize_tsv(&row.required_assets),
            sanitize_tsv(&row.expected_outputs),
            sanitize_tsv(&row.skip_behavior),
            sanitize_tsv(&row.reason),
        );
    }
    rendered
}
pub(in crate::bam::assignment) fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}
