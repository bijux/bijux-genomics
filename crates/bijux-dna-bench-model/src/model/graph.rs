//! Owner: bijux-dna-bench-model
//! Graph nodes and edges for benchmark suite execution topology.
//! Must not perform IO or depend on compare/gate logic.

use schemars::{json_schema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::BENCHMARK_LOCAL_HPC_JOB_GRAPH_SCHEMA_V1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkGraphNodeKind {
    Stage,
    StageTool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkGraphNode {
    pub node_id: String,
    pub kind: BenchmarkGraphNodeKind,
    pub stage_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_instance_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageEdge {
    pub from: String,
    pub to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_output_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_input_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcJobGraphNode {
    pub job_id_local: String,
    pub job_kind: String,
    pub result_id: Option<String>,
    pub pipeline_id: Option<String>,
    pub node_id: Option<String>,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub dependency_count: usize,
    pub dependent_count: usize,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcJobGraph {
    #[schemars(schema_with = "local_hpc_job_graph_schema_version")]
    pub schema_version: String,
    pub manifest_path: String,
    pub job_count: usize,
    pub benchmark_job_count: usize,
    pub essential_pipeline_job_count: usize,
    pub dependency_count: usize,
    pub root_job_ids: Vec<String>,
    pub topological_order: Vec<String>,
    pub nodes: Vec<BenchmarkLocalHpcJobGraphNode>,
}

#[must_use]
pub fn benchmark_local_hpc_job_graph_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalHpcJobGraph)
}

fn local_hpc_job_graph_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_HPC_JOB_GRAPH_SCHEMA_V1,
    })
}
