use super::metadata::BamReportStageMetadata;
use crate::observations::ExpectedResultBinding;
pub(super) fn report_reason(
    row: &ExpectedResultBinding,
    metadata: &BamReportStageMetadata,
    support_status: &str,
) -> String {
    let report_section_id = metadata.report_section_id.as_str();
    let summary_table_id = metadata.summary_table_id.as_str();
    let report_focus = metadata.report_focus.as_str();
    let anchor_tool_id = metadata.anchor_tool_id.as_str();
    let anchor_support_status = metadata.anchor_support_status.as_str();
    let workflow_branch_id = metadata.workflow_branch_id.as_deref();
    let scientific_context_required = &metadata.scientific_context_required;

    let branch_note = workflow_branch_id
        .map(|branch_id| format!("; workflow branch `{branch_id}`"))
        .unwrap_or_default();
    let scientific_note = if scientific_context_required.is_empty() {
        String::new()
    } else {
        format!("; scientific context requires {}", scientific_context_required.join(", "))
    };
    format!(
        "benchmark result binding `{}` / `{}` on fixture `{}` keeps BAM tool support status `{support_status}` while stage anchor tool `{anchor_tool_id}` remains `{anchor_support_status}`; benchmark reporting places it in section `{report_section_id}` and summary table `{summary_table_id}` to track {report_focus}{branch_note}{scientific_note}",
        row.stage_id, row.tool_id, row.fixture_id
    )
}
