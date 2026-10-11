use serde::Serialize;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfAdapterMissingInputTestRow {
    pub contract_surface: String,
    pub stage_id: String,
    pub tool_id: String,
    pub benchmark_status: String,
    pub missing_input_role: String,
    pub artifact_id: String,
    pub artifact_path: String,
    pub expected_error_fragment: String,
    pub observed_error: String,
    pub passed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfAdapterMissingInputTestsReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub row_count: usize,
    pub passed_row_count: usize,
    pub failed_row_count: usize,
    pub adapter_row_count: usize,
    pub support_row_count: usize,
    pub rows: Vec<VcfAdapterMissingInputTestRow>,
}
