use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfParserFailureTestRow {
    pub case_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub parser_id: String,
    pub failure_reason: String,
    pub fixture_path: String,
    pub probe_artifact_path: String,
    pub expected_error_fragment: String,
    pub observed_error: String,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfParserFailureTestsReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub row_count: usize,
    pub passed_row_count: usize,
    pub failed_row_count: usize,
    pub failure_reason_counts: BTreeMap<String, usize>,
    pub rows: Vec<VcfParserFailureTestRow>,
}
