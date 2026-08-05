//! Owner: bijux-dna-bench-model
//! Typed benchmark observations and invariants.
//! Owns bench core data model only.
//! Must not perform IO or depend on compare/gate logic.
//! Invariants: observations are fully typed and include confounders.

mod construction;
mod metrics_envelope;
mod validation;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::OBSERVATION_SCHEMA_V1;

pub use metrics_envelope::MetricsEnvelope;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkObservation {
    #[schemars(schema_with = "benchmark_observation_schema_version")]
    pub schema_version: String,
    pub run_id: String,
    pub dataset_id: String,
    pub dataset_class: String,
    pub read_layout: String,
    pub stage_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_instance_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage_id: Option<String>,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: String,
    pub container_digest: String,
    pub params_hash: String,
    pub input_hash: String,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub exit_code: i32,
    pub failure_kind: Option<String>,
    pub metrics: MetricsEnvelope,
    pub replicate_id: String,
    pub replicate_index: u32,
    pub warmup_policy: String,
    pub seed_policy: String,
    pub runner: String,
    pub platform: String,
    pub cpu: String,
    pub threads: u32,
    pub io_mode: String,
}

#[must_use]
pub fn benchmark_observation_schema() -> Schema {
    schemars::schema_for!(BenchmarkObservation)
}

fn benchmark_observation_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": OBSERVATION_SCHEMA_V1})
}
