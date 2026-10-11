use super::model::VcfUndercoveredStageRow;
use std::fmt::Write;
pub(super) fn render_vcf_undercovered_stages_tsv(rows: &[VcfUndercoveredStageRow]) -> String {
    let mut rendered =
        String::from("stage_id\tvalid_tool_classes\tregistered_tools\tmissing_tools\tdecision\n");
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.valid_tool_classes.join(",")),
            sanitize_tsv(&row.registered_tools.join(",")),
            sanitize_tsv(&row.missing_tools.join(",")),
            sanitize_tsv(&row.decision),
        );
    }
    rendered
}
fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}
