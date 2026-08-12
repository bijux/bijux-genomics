//! Persisted local corpus-to-stage compatibility validation contracts.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::BENCHMARK_LOCAL_CORPUS_STAGE_COMPATIBILITY_SCHEMA_V1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalCorpusStageValidatedFixture {
    pub corpus_family_id: String,
    pub fixture_id: String,
    pub fixture_manifest: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalCorpusStageCompatibilityEntry {
    pub stage_id: String,
    pub readiness_kind: String,
    pub compatibility_kind: String,
    pub corpus_family_id: Option<String>,
    pub fixture_id: Option<String>,
    pub benchmark_scope_id: Option<String>,
    pub compatibility_note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalCorpusStageCompatibilityReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub matrix_path: String,
    pub fixture_count: usize,
    pub stage_count: usize,
    pub fixture_backed_stage_count: usize,
    pub asset_backed_stage_count: usize,
    pub planner_only_stage_count: usize,
    pub corpus_family_counts: BTreeMap<String, usize>,
    pub valid: bool,
    pub fixtures: Vec<BenchmarkLocalCorpusStageValidatedFixture>,
    pub stages: Vec<BenchmarkLocalCorpusStageCompatibilityEntry>,
}

#[must_use]
pub fn benchmark_local_corpus_stage_compatibility_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalCorpusStageCompatibilityReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_CORPUS_STAGE_COMPATIBILITY_SCHEMA_V1,
    })
}
