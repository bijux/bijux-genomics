//! Persisted local essential-pipeline execution simulation contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{
    BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUNS_SCHEMA_V1,
    BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUN_NODE_METRICS_SCHEMA_V1,
    BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUN_PIPELINE_SCHEMA_V1,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalEssentialPipelineFakeRunOutputEntry {
    pub artifact_id: String,
    pub declared_output: String,
    pub fake_run_path: String,
    pub role: String,
    pub exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalEssentialPipelineFakeRunInputBinding {
    pub input_id: String,
    pub source: String,
    pub producer_node_id: Option<String>,
    pub fake_run_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalEssentialPipelineFakeRunNodeMetrics {
    #[schemars(schema_with = "node_metrics_schema_version")]
    pub schema_version: String,
    pub pipeline_id: String,
    pub node_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub dependency_count: usize,
    pub external_input_count: usize,
    pub upstream_input_count: usize,
    pub command_step_count: usize,
    pub output_count: usize,
    pub materialized_byte_count: u64,
    pub simulated_elapsed_seconds: f64,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub upstream_bindings: Vec<BenchmarkLocalEssentialPipelineFakeRunInputBinding>,
    pub external_bindings: Vec<BenchmarkLocalEssentialPipelineFakeRunInputBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalEssentialPipelineFakeRunNodeReport {
    pub pipeline_id: String,
    pub node_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub domain: String,
    pub readiness_kind: String,
    pub command_source: String,
    pub dependency_count: usize,
    pub command_step_count: usize,
    pub declared_output_count: usize,
    pub created_output_count: usize,
    pub command_script_path: String,
    pub stdout_path: String,
    pub stderr_path: String,
    pub metrics_path: String,
    pub stage_result_path: String,
    pub outputs: Vec<BenchmarkLocalEssentialPipelineFakeRunOutputEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalEssentialPipelineFakeRunPipelineReport {
    #[schemars(schema_with = "pipeline_schema_version")]
    pub schema_version: String,
    pub pipeline_id: String,
    pub pipeline_manifest_path: String,
    pub node_count: usize,
    pub created_output_count: usize,
    pub topological_order: Vec<String>,
    pub nodes: Vec<BenchmarkLocalEssentialPipelineFakeRunNodeReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalEssentialPipelineFakeRunsReport {
    #[schemars(schema_with = "root_schema_version")]
    pub schema_version: String,
    pub fake_run_root: String,
    pub root_manifest_path: String,
    pub pipeline_count: usize,
    pub node_count: usize,
    pub created_output_count: usize,
    pub pipelines: Vec<BenchmarkLocalEssentialPipelineFakeRunPipelineReport>,
}

#[must_use]
pub fn benchmark_local_essential_pipeline_fake_runs_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalEssentialPipelineFakeRunsReport)
}

#[must_use]
pub fn benchmark_local_essential_pipeline_fake_run_pipeline_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalEssentialPipelineFakeRunPipelineReport)
}

#[must_use]
pub fn benchmark_local_essential_pipeline_fake_run_node_metrics_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalEssentialPipelineFakeRunNodeMetrics)
}

fn root_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUNS_SCHEMA_V1,
    })
}

fn pipeline_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUN_PIPELINE_SCHEMA_V1,
    })
}

fn node_metrics_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUN_NODE_METRICS_SCHEMA_V1,
    })
}
