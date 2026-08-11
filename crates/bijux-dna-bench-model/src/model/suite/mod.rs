//! Owner: bijux-dna-bench-model
//! Benchmark suite specification (versioned).
//! Owns suite-level inputs for bench orchestration.
//! Must not perform IO or depend on compare/gate logic.
//! Invariants: `schema_version` is stable and versioned.

mod catalog;
mod construction;
mod manifest;
mod stage_graph;
mod support;

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::SUITE_SCHEMA_V1;
use crate::model::graph::BenchmarkStageEdge;
pub use manifest::{
    benchmark_suite_run_manifest_schema, ColdWarmContract, DecisionTraceRow, EnvironmentSnapshot,
    FairnessContract, MetricsArtifactRow, PostconditionRow, RunRecordRow, ScientificDefaultsStatus,
    SuiteRunManifest, ToolInvocationRow,
};
pub use support::{
    AnalysisRequirements, DatasetSpec, DiversityRequirements, ReplicatePolicy,
    StratificationRequirement,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkParamBinding {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_instance_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub values: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageSpec {
    pub stage: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_instance_id: Option<String>,
    pub tools: Vec<String>,
    #[serde(default)]
    pub params: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_bindings: Vec<BenchmarkParamBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upstream_stage_instance_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkSuiteSpec {
    #[schemars(schema_with = "benchmark_suite_schema_version")]
    pub schema_version: String,
    pub suite_id: String,
    pub datasets: Vec<DatasetSpec>,
    pub stages: Vec<BenchmarkStageSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<BenchmarkStageEdge>,
    pub replicate_policy: ReplicatePolicy,
    pub diversity: DiversityRequirements,
    pub stratifications: Vec<StratificationRequirement>,
    pub analysis_requirements: AnalysisRequirements,
}

#[must_use]
pub fn benchmark_suite_spec_schema() -> Schema {
    schemars::schema_for!(BenchmarkSuiteSpec)
}

fn benchmark_suite_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": SUITE_SCHEMA_V1})
}
