//! Owner: bijux-dna-bench-model
//! Typed benchmark stage-interface and parameter-configuration contracts.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::{
    BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1, BENCHMARK_STAGE_INTERFACES_SCHEMA_V1,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageDownstreamProbe {
    pub consumer_stage_id: String,
    pub input_contract: String,
    pub source: String,
    pub assertions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageInterfaceSpec {
    pub stage_id: String,
    pub accepted_input_contracts: Vec<String>,
    pub output_contract: String,
    pub normalized_result_contract: String,
    pub parameter_schema_version: String,
    pub default_provider: String,
    pub parameter_config: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planner_contract: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operator_override_keys: Vec<String>,
    pub input_derived_parameters: Vec<String>,
    pub immutable_parameters: Vec<String>,
    pub effective_parameters_artifact: String,
    pub tool_ids: Vec<String>,
    pub provenance_sources: Vec<String>,
    pub adapters: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub parameter_projections: BTreeMap<String, BTreeMap<String, String>>,
    pub downstream_probe: BenchmarkStageDownstreamProbe,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageInterfaceCatalog {
    #[schemars(schema_with = "benchmark_stage_interfaces_schema_version")]
    pub schema_version: String,
    pub interfaces: Vec<BenchmarkStageInterfaceSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
pub enum BenchmarkParameterValue {
    Integer(i64),
    Boolean(bool),
    Text(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageParameterConfig {
    #[schemars(length(min = 1))]
    pub parameter_schema_version: String,
    #[schemars(length(min = 1))]
    pub defaults_provider: String,
    pub overrides: BTreeMap<String, BenchmarkParameterValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkPipelineParameterConfig {
    #[schemars(schema_with = "benchmark_pipeline_parameters_schema_version")]
    pub schema_version: String,
    #[schemars(length(min = 1))]
    pub profile: String,
    #[schemars(schema_with = "nonempty_stage_parameter_map_schema")]
    pub stages: BTreeMap<String, BenchmarkStageParameterConfig>,
}

#[must_use]
pub fn benchmark_stage_interface_catalog_schema() -> Schema {
    schemars::schema_for!(BenchmarkStageInterfaceCatalog)
}

#[must_use]
pub fn benchmark_pipeline_parameter_config_schema() -> Schema {
    schemars::schema_for!(BenchmarkPipelineParameterConfig)
}

fn benchmark_stage_interfaces_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_STAGE_INTERFACES_SCHEMA_V1,
    })
}

fn benchmark_pipeline_parameters_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1,
    })
}

fn nonempty_stage_parameter_map_schema(generator: &mut SchemaGenerator) -> Schema {
    let mut schema = generator.subschema_for::<BTreeMap<String, BenchmarkStageParameterConfig>>();
    schema.insert("minProperties".to_string(), serde_json::Value::from(1));
    schema
}
