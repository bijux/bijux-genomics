use super::super::paths::sanitize_tsv;
use super::model::FastqActiveStageToolMatrixRow;
use std::fmt::Write as _;

pub(super) fn render_fastq_active_stage_tool_matrix_tsv(
    rows: &[FastqActiveStageToolMatrixRow],
) -> String {
    let mut rendered = String::from(
        "stage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tbenchmark_status\tsupport_status\tadapter_status\tparser_status\tcorpus_status\tscope_proof_path\treason\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.corpus_id),
            sanitize_tsv(&row.asset_profile_id),
            sanitize_tsv(&row.adapter_id),
            sanitize_tsv(&row.parser_id),
            sanitize_tsv(&row.schema_id),
            sanitize_tsv(&row.benchmark_status),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.corpus_status),
            sanitize_tsv(&row.scope_proof_path),
            sanitize_tsv(&row.reason),
        );
    }
    rendered
}
