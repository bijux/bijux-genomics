use serde::Serialize;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ToolServingMapRow {
    pub tool_id: String,
    pub stage_id: String,
    pub support_status: String,
    pub adapter_status: String,
    pub parser_status: String,
    pub corpus_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolServingMapReport {
    pub schema_version: &'static str,
    pub domain: &'static str,
    pub output_path: String,
    pub stage_count: usize,
    pub tool_count: usize,
    pub row_count: usize,
    pub rows: Vec<ToolServingMapRow>,
}
