use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FastqReportMapRow {
    pub result_row_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_family_id: String,
    pub fixture_id: String,
    pub sample_scope: String,
    pub canonical_stage_rank: usize,
    pub readiness_kind: String,
    pub stage_kind: String,
    pub criticality: String,
    pub support_status: String,
    pub report_section_id: String,
    pub report_section_title: String,
    pub summary_table_id: String,
    pub summary_table_title: String,
    pub metric_classes: Vec<String>,
    pub mutates_fastq: bool,
    pub produces_reports_only: bool,
    pub report_focus: String,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct FastqReportMapReport {
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
    pub stage_kind_counts: BTreeMap<String, usize>,
    pub rows: Vec<FastqReportMapRow>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FastqReportStageMetadata {
    pub canonical_stage_rank: usize,
    pub readiness_kind: String,
    pub stage_kind: String,
    pub criticality: String,
    pub anchor_tool_id: String,
    pub anchor_support_status: String,
    pub report_section_id: String,
    pub report_section_title: String,
    pub summary_table_id: String,
    pub summary_table_title: String,
    pub metric_classes: Vec<String>,
    pub mutates_fastq: bool,
    pub produces_reports_only: bool,
    pub report_focus: String,
}
#[derive(Debug, Clone, Copy)]
pub(super) struct FastqReportPlacement {
    pub(super) section_id: &'static str,
    pub(super) section_title: &'static str,
    pub(super) summary_table_id: &'static str,
    pub(super) summary_table_title: &'static str,
    pub(super) report_focus: &'static str,
}

#[derive(Clone, Copy)]
pub(super) struct RequiredReportMapping<'a> {
    pub(super) stage_id: &'a str,
    pub(super) tool_id: &'a str,
    pub(super) section_id: &'a str,
    pub(super) summary_table_id: &'a str,
    pub(super) stage_kind: &'a str,
    pub(super) criticality: &'a str,
    pub(super) support_status: &'a str,
}
