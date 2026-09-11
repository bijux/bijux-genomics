use super::models::BamReportMapRow;
use crate::publication::sanitize_tsv;
use std::fmt::Write;
pub(super) fn render_bam_report_map_tsv(rows: &[BamReportMapRow]) -> String {
    let mut rendered = String::from(
        "result_row_id\tstage_id\ttool_id\tcorpus_family_id\tfixture_id\tsample_scope\tcanonical_stage_rank\treadiness_kind\tcriticality\tsupport_status\tanchor_tool_id\tanchor_support_status\treport_section_id\treport_section_title\tsummary_table_id\tsummary_table_title\tworkflow_branch_id\tscientific_context_required\treport_focus\treason\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.result_row_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.corpus_family_id),
            sanitize_tsv(&row.fixture_id),
            sanitize_tsv(&row.sample_scope),
            row.canonical_stage_rank,
            sanitize_tsv(&row.readiness_kind),
            sanitize_tsv(&row.criticality),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.anchor_tool_id),
            sanitize_tsv(&row.anchor_support_status),
            sanitize_tsv(&row.report_section_id),
            sanitize_tsv(&row.report_section_title),
            sanitize_tsv(&row.summary_table_id),
            sanitize_tsv(&row.summary_table_title),
            sanitize_tsv(row.workflow_branch_id.as_deref().unwrap_or("")),
            sanitize_tsv(&row.scientific_context_required.join(",")),
            sanitize_tsv(&row.report_focus),
            sanitize_tsv(&row.reason),
        );
    }
    rendered
}
