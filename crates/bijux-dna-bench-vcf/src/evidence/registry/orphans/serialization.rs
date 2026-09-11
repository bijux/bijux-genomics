use super::model::VcfOrphanToolRow;
use std::fmt::Write;
pub(super) fn render_vcf_orphan_tools_tsv(rows: &[VcfOrphanToolRow]) -> String {
    let mut rendered = String::from("tool_id\tregistered_binary\tserved_stage_count\tdecision\n");
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.registered_binary),
            row.served_stage_count,
            sanitize_tsv(&row.decision),
        );
    }
    rendered
}
fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}
