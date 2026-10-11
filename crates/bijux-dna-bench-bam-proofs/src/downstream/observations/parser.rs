use std::collections::BTreeMap;
/// Consumed canonical parser evidence; not another serialized parser report.
#[derive(Debug, Clone)]
pub struct ParserObservation {
    pub representative_stage_id: String,
    pub representative_tool_id: String,
    pub proof_path: String,
    pub parsed_schema_version: String,
    pub normalized_snapshot: BTreeMap<String, serde_json::Value>,
    pub passed: bool,
}

/// Preserves canonical parser rows until stage/tool admission.
pub struct ParserObservations {
    pub rows: Vec<ParserObservation>,
}
