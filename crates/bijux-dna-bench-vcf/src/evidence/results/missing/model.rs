use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VcfMissingResultStatus {
    Present,
    MissingResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfMissingResultReportRow {
    pub result_row_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub expected_manifest_path: String,
    pub audit_manifest_path: String,
    pub result_status: VcfMissingResultStatus,
    pub expected_output_artifact_ids: Vec<String>,
    pub observed_output_artifact_ids: Vec<String>,
    pub expected_metrics: Vec<String>,
    pub report_section: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfMissingResultReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub fake_result_root: String,
    pub expected_row_count: usize,
    pub present_result_row_count: usize,
    pub missing_result_row_count: usize,
    pub passes_behavior_test: bool,
    pub removed_result_row_id: String,
    pub removed_manifest_path: String,
    pub report_section_counts: BTreeMap<String, usize>,
    pub rows: Vec<VcfMissingResultReportRow>,
}
