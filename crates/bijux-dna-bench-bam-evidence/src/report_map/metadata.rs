#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BamReportStageMetadata {
    pub(super) canonical_stage_rank: usize,
    pub(super) readiness_kind: String,
    pub(super) criticality: String,
    pub(super) anchor_tool_id: String,
    pub(super) anchor_support_status: String,
    pub(super) report_section_id: String,
    pub(super) report_section_title: String,
    pub(super) summary_table_id: String,
    pub(super) summary_table_title: String,
    pub(super) workflow_branch_id: Option<String>,
    pub(super) scientific_context_required: Vec<String>,
    pub(super) report_focus: String,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct BamReportPlacement {
    pub(super) section_id: &'static str,
    pub(super) section_title: &'static str,
    pub(super) summary_table_id: &'static str,
    pub(super) summary_table_title: &'static str,
    pub(super) report_focus: &'static str,
}
