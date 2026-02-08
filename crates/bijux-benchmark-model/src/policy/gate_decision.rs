//! Owner: bijux-benchmark
//! Gate decision outputs.

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GateViolation {
    pub metric_id: String,
    pub observed: f64,
    pub threshold: f64,
    pub direction: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GateDecision {
    pub schema_version: String,
    pub dataset_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub params_hash: String,
    pub passes: bool,
    pub violations: Vec<GateViolation>,
    pub missing_metrics: Vec<String>,
    pub completeness_score: f64,
    pub rationale_trace: Vec<String>,
}
