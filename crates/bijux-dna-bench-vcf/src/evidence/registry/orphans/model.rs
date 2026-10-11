use serde::Serialize;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfOrphanToolRow {
    pub tool_id: String,
    pub registered_binary: String,
    pub served_stage_count: usize,
    pub decision: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfOrphanToolsReport {
    pub schema_version: &'static str,
    pub domain: &'static str,
    pub output_path: String,
    pub orphan_count: usize,
    pub required_tool_count: usize,
    pub registered_tool_count: usize,
    pub served_tool_count: usize,
    pub rows: Vec<VcfOrphanToolRow>,
}
