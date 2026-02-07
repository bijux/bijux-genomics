use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryEventV1 {
    pub schema_version: String,
    pub run_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub event_name: String,
    pub timestamp: String,
    pub duration_ms: Option<u64>,
    pub status: String,
    pub trace_id: String,
    pub span_id: String,
    pub attrs: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactsRowV1 {
    pub schema_version: String,
    pub run_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: Option<String>,
    pub trace_id: String,
    pub span_id: String,
    pub params_hash: String,
    pub input_hash: String,
    pub output_hashes: Vec<String>,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub exit_code: i32,
    pub bank_hashes: serde_json::Value,
    pub reads_in: Option<u64>,
    pub reads_out: Option<u64>,
    pub bases_in: Option<u64>,
    pub bases_out: Option<u64>,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
    pub metrics: serde_json::Value,
    pub reports: serde_json::Value,
    pub artifacts: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunProvenanceV1 {
    pub schema_version: String,
    pub tool_image_digest: Option<String>,
    pub tool_version: String,
    pub params_hash: String,
    pub input_hashes: Vec<String>,
    pub reference_genome: Option<String>,
    pub pipeline_id: String,
    pub git_commit: String,
    pub build_profile: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_hash: Option<String>,
}
