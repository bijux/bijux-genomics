use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
pub struct RealOutputParserSmokeRow {
    pub family_id: String,
    pub family_summary: String,
    pub representative_stage_id: String,
    pub representative_tool_id: String,
    pub proof_path: String,
    pub parser_surface: String,
    pub parsed_schema_version: String,
    pub parsed_top_level_key_count: usize,
    pub parsed_top_level_keys: Vec<String>,
    pub normalized_snapshot: BTreeMap<String, Value>,
    pub passed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RealOutputParserSmokeReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub probe_root: String,
    pub family_count: usize,
    pub passed_family_count: usize,
    pub failed_family_count: usize,
    pub parser_surface_counts: BTreeMap<String, usize>,
    pub rows: Vec<RealOutputParserSmokeRow>,
}
