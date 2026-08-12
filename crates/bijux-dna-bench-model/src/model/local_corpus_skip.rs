//! Persisted local benchmark corpus-skip report contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::BENCHMARK_LOCAL_CORPUS_SKIP_REPORT_SCHEMA_V1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalCorpusFixture {
    pub corpus_id: String,
    pub corpus_family_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalCorpusSkip {
    pub stage_id: String,
    pub readiness_kind: String,
    pub corpus_id: String,
    pub corpus_family_id: String,
    pub reason: String,
    pub replacement_corpus_id: String,
    pub replacement_corpus_family_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalPlannerOnlyStage {
    pub stage_id: String,
    pub readiness_kind: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalAssetBackedStage {
    pub stage_id: String,
    pub readiness_kind: String,
    pub benchmark_scope_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalCorpusSkipReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub matrix_path: String,
    pub output_path: String,
    pub fixture_count: usize,
    pub stage_count: usize,
    pub skip_count: usize,
    pub asset_backed_stage_count: usize,
    pub planner_only_stage_count: usize,
    pub fixtures: Vec<BenchmarkLocalCorpusFixture>,
    pub skips: Vec<BenchmarkLocalCorpusSkip>,
    pub asset_backed_stages: Vec<BenchmarkLocalAssetBackedStage>,
    pub planner_only_stages: Vec<BenchmarkLocalPlannerOnlyStage>,
}

#[must_use]
pub fn benchmark_local_corpus_skip_report_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalCorpusSkipReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_CORPUS_SKIP_REPORT_SCHEMA_V1,
    })
}
