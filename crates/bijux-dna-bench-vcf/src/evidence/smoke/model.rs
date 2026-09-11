use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfLocalContainerSmokeRow {
    pub stage_id: String,
    pub tool_id: String,
    pub registered_binary: String,
    pub tool_status: String,
    pub stage_support_status: String,
    pub scope_state: String,
    pub scope_detail: String,
    pub smoke_path_kind: String,
    pub smoke_runtime: String,
    pub smoke_tool_id: String,
    pub smoke_command: String,
    pub smoke_support_path: String,
    pub smoke_minimal_cmd: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfLocalContainerSmokeReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub row_count: usize,
    pub stage_count: usize,
    pub tool_count: usize,
    pub host_stage_smoke_row_count: usize,
    pub container_smoke_row_count: usize,
    pub external_artifact_row_count: usize,
    pub runtime_counts: BTreeMap<String, usize>,
    pub rows: Vec<VcfLocalContainerSmokeRow>,
}
