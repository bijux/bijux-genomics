use super::ToolServingMapRow;
use crate::publication::sanitize_tsv;
use std::fmt::Write;
pub(super) fn render_tool_serving_map_tsv(rows: &[ToolServingMapRow]) -> String {
    let mut rendered = String::from(
        "tool_id\tstage_id\tsupport_status\tadapter_status\tparser_status\tcorpus_status\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.corpus_status),
        );
    }
    rendered
}
