//! Owner: bijux-bench
//! Typed benchmark observations and invariants.
//! Owns bench core data model only.
//! Must not perform IO or depend on compare/gate logic.
//! Invariants: observations are fully typed and include confounders.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedMetricsRef {
    pub stage_id: String,
    pub schema_version: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchObservation {
    pub run_id: String,
    pub tool: String,
    pub params_hash: String,
    pub dataset_hash: String,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub metrics: TypedMetricsRef,
    pub replicate_id: String,
    pub replicate_index: u32,
    pub runner: String,
    pub platform: String,
    pub threads: u32,
    pub io_mode: String,
    pub image_digest: String,
}
