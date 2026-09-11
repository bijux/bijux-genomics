use serde::Serialize;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BamAdapterOutputContractStatus {
    Complete,
    Incomplete,
    MissingAdapter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BamAdapterOutputContractRow {
    pub tool_id: String,
    pub stage_id: String,
    pub adapter_status: String,
    pub output_contract_status: BamAdapterOutputContractStatus,
    pub stage_output_ids: Vec<String>,
    pub stage_expected_artifact_ids: Vec<String>,
    pub declared_output_ids: Vec<String>,
    pub execution_expected_output_ids: Vec<String>,
    pub missing_declarations: Vec<String>,
    pub raw_output_artifact_ids: Vec<String>,
    pub normalized_metrics_output_id: Option<String>,
    pub stdout_path_template: Option<String>,
    pub stderr_path_template: Option<String>,
    pub stage_result_manifest_path_template: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamAdapterOutputContractReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub row_count: usize,
    pub adapter_row_count: usize,
    pub complete_adapter_row_count: usize,
    pub incomplete_adapter_row_count: usize,
    pub missing_adapter_row_count: usize,
    pub rows: Vec<BamAdapterOutputContractRow>,
}

impl BamAdapterOutputContractStatus {
    /// Stable label used by the persisted evidence schema.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            BamAdapterOutputContractStatus::Complete => "complete",
            BamAdapterOutputContractStatus::Incomplete => "incomplete",
            BamAdapterOutputContractStatus::MissingAdapter => "missing_adapter",
        }
    }
}
