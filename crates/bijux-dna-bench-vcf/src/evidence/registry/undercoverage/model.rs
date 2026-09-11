use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfUndercoveredStageRow {
    pub stage_id: String,
    pub valid_tool_classes: Vec<String>,
    pub registered_tools: Vec<String>,
    pub missing_tools: Vec<String>,
    pub decision: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfUndercoveredStagesReport {
    pub schema_version: &'static str,
    pub domain: &'static str,
    pub output_path: String,
    pub stage_count: usize,
    pub undercovered_stage_count: usize,
    pub decision_counts: BTreeMap<String, usize>,
    pub rows: Vec<VcfUndercoveredStageRow>,
}
