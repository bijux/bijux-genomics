//! Persisted closure of the supported container catalog against domain and benchmark protocols.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

pub const BENCHMARK_STAGE_TOOL_CLOSURE_SCHEMA_V1: &str = "bijux.benchmark.stage_tool_closure.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageToolClosureSource {
    pub path: String,
    #[schemars(schema_with = "sha256_schema")]
    pub sha256: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkStageToolProtocolDisposition {
    BenchmarkRequired,
    QualificationOnly,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkStageToolClosureDisposition {
    Aligned,
    MissingDomainBinding,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageToolClosureRow {
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub product_support: String,
    pub artifact_delivery: String,
    pub redistribution_permitted: bool,
    pub domain_declared: bool,
    pub protocol_disposition: BenchmarkStageToolProtocolDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub benchmark_run_id: Option<String>,
    pub closure: BenchmarkStageToolClosureDisposition,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageToolClosureSummary {
    pub supported_tool_count: usize,
    pub stage_count: usize,
    pub catalog_pair_count: usize,
    pub domain_pair_count: usize,
    pub aligned_pair_count: usize,
    pub drift_pair_count: usize,
    pub benchmark_pair_count: usize,
    pub qualification_only_pair_count: usize,
    pub external_artifact_pair_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageToolClosureReport {
    #[schemars(schema_with = "stage_tool_closure_schema_version")]
    pub schema_version: String,
    pub sources: Vec<BenchmarkStageToolClosureSource>,
    pub summary: BenchmarkStageToolClosureSummary,
    pub rows: Vec<BenchmarkStageToolClosureRow>,
}

#[must_use]
pub fn benchmark_stage_tool_closure_schema() -> Schema {
    schemars::schema_for!(BenchmarkStageToolClosureReport)
}

fn stage_tool_closure_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_STAGE_TOOL_CLOSURE_SCHEMA_V1,
    })
}

fn sha256_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "pattern": "^[0-9a-f]{64}$",
    })
}
