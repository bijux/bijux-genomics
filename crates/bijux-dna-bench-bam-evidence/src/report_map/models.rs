use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BamReportMapRow {
    pub result_row_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_family_id: String,
    pub fixture_id: String,
    pub sample_scope: String,
    pub canonical_stage_rank: usize,
    pub readiness_kind: String,
    pub criticality: String,
    pub support_status: String,
    pub anchor_tool_id: String,
    pub anchor_support_status: String,
    pub report_section_id: String,
    pub report_section_title: String,
    pub summary_table_id: String,
    pub summary_table_title: String,
    pub workflow_branch_id: Option<String>,
    pub scientific_context_required: Vec<String>,
    pub report_focus: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamReportMapReport {
    pub schema_version: &'static str,
    pub domain: &'static str,
    pub output_path: String,
    pub expected_result_row_count: usize,
    pub row_count: usize,
    pub stage_count: usize,
    pub tool_count: usize,
    pub section_count: usize,
    pub summary_table_count: usize,
    pub section_counts: BTreeMap<String, usize>,
    pub branch_counts: BTreeMap<String, usize>,
    pub rows: Vec<BamReportMapRow>,
}
