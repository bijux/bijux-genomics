//! Persisted index of benchmark publication dossiers.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

pub const BENCHMARK_DOSSIER_INDEX_SCHEMA_V1: &str = "bijux.bench.dossier_index.v1";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDossierIndex {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub corpus_id: String,
    pub stage_count: usize,
    pub published_stage_count: usize,
    pub missing_stage_count: usize,
    pub stages: Vec<BenchmarkDossierStageEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDossierStageEntry {
    pub stage_id: String,
    pub sample_scope: String,
    pub status: String,
    pub summary_path: String,
    pub dossier_path: String,
    pub expected_remote_run_root: String,
    pub expected_local_cache_mirror_run_root: String,
    pub expected_local_results_run_root: String,
    pub generated_at_utc: Option<String>,
    pub platform: Option<String>,
    pub corpus_root: Option<String>,
    pub run_root: Option<String>,
    pub run_root_source: Option<String>,
}

#[must_use]
pub fn benchmark_dossier_index_schema() -> Schema {
    schemars::schema_for!(BenchmarkDossierIndex)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_DOSSIER_INDEX_SCHEMA_V1,
    })
}
