//! Owner: bijux-dna-bench-model
//! Gate decision outputs.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};

use crate::contract::DECISION_SCHEMA_V1;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GateViolation {
    pub metric_id: String,
    pub observed: f64,
    pub threshold: f64,
    pub direction: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GateDecision {
    #[schemars(schema_with = "gate_decision_schema_version")]
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

#[must_use]
pub fn gate_decision_schema() -> Schema {
    schemars::schema_for!(GateDecision)
}

fn gate_decision_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": DECISION_SCHEMA_V1})
}
