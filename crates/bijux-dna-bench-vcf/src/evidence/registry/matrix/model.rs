use serde::Serialize;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfMatrixRegistryConsistencyRow {
    pub stage_id: String,
    pub tool_id: String,
    pub drift_kind: String,
    pub stage_support_status: String,
    pub tool_registry_status: String,
    pub registry_stage_ids: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfMatrixRegistryConsistencyReport {
    pub schema_version: &'static str,
    pub domain: &'static str,
    pub output_path: String,
    pub passes_gate: bool,
    pub stage_count: usize,
    pub matrix_row_count: usize,
    pub registry_pair_count: usize,
    pub benchmark_ready_registry_pair_count: usize,
    pub unregistered_matrix_pair_count: usize,
    pub missing_benchmark_ready_registry_pair_count: usize,
    pub rows: Vec<VcfMatrixRegistryConsistencyRow>,
}
