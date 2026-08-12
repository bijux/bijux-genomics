//! Persisted local stage-command rendering contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{BenchmarkLocalStageReadinessKind, BENCHMARK_LOCAL_STAGE_COMMANDS_SCHEMA_V3};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageArtifactEntry {
    pub artifact_id: String,
    pub path: String,
    pub role: String,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageCommandEntry {
    pub stage_id: String,
    pub readiness_kind: BenchmarkLocalStageReadinessKind,
    pub tool_id: String,
    pub inputs: Vec<BenchmarkLocalStageArtifactEntry>,
    pub outputs: Vec<BenchmarkLocalStageArtifactEntry>,
    pub threads: u32,
    pub memory_mb: u32,
    pub argv: Vec<String>,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageCommandManifest {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub script_output_path: String,
    pub manifest_output_path: String,
    pub argv_output_path: String,
    pub command_count: usize,
    pub commands: Vec<BenchmarkLocalStageCommandEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageCommandArgvEntry {
    pub stage_id: String,
    pub tool_id: String,
    pub readiness_kind: BenchmarkLocalStageReadinessKind,
    pub argv: Vec<String>,
}

#[must_use]
pub fn benchmark_local_stage_commands_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalStageCommandManifest)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_STAGE_COMMANDS_SCHEMA_V3,
    })
}
