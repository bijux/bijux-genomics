//! Owner: bijux-dna-bench-model
//! Persisted stage/tool benchmark-readiness report contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::STAGE_TOOL_BENCHMARK_READY_SCHEMA_V1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StageToolBenchmarkReadySurfaceStatus {
    ReadySliceComplete,
    FailingPairsPresent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageToolBenchmarkReadySurfaceSummary {
    pub surface_id: String,
    pub surface_status: StageToolBenchmarkReadySurfaceStatus,
    pub measured_scope: String,
    pub expected_count: usize,
    pub covered_count: usize,
    pub excluded_count: usize,
    pub failing_count: usize,
    pub evidence_paths: Vec<String>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageToolBenchmarkReadyFailingPair {
    pub row_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub failure_surfaces: Vec<String>,
    pub benchmark_status: String,
    pub readiness_gap: String,
    pub support_status: String,
    pub adapter_status: String,
    pub parser_status: String,
    pub corpus_status: String,
    pub asset_status: String,
    pub registry_status: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageToolBenchmarkReadyExcludedPair {
    pub row_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub readiness_gap: String,
    pub support_status: String,
    pub adapter_status: String,
    pub parser_status: String,
    pub corpus_status: String,
    pub asset_status: String,
    pub registry_status: String,
    pub excluded_from_generated_jobs: bool,
    pub excluded_from_expected_results: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageToolBenchmarkReadyReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub passes_gate: bool,
    pub expected_pair_count: usize,
    pub benchmark_ready_pair_count: usize,
    pub excluded_pair_count: usize,
    pub failing_pair_count: usize,
    pub generated_job_pair_count: usize,
    pub expected_result_pair_count: usize,
    pub benchmark_ready_stage_count: usize,
    pub excluded_registry_gap_count: usize,
    pub surface_summaries: Vec<StageToolBenchmarkReadySurfaceSummary>,
    pub failing_pairs: Vec<StageToolBenchmarkReadyFailingPair>,
    pub excluded_pairs: Vec<StageToolBenchmarkReadyExcludedPair>,
}

#[must_use]
pub fn stage_tool_benchmark_ready_schema() -> Schema {
    schemars::schema_for!(StageToolBenchmarkReadyReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": STAGE_TOOL_BENCHMARK_READY_SCHEMA_V1})
}
