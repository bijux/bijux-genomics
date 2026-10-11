//! Persisted FASTQ corpus benchmark run records.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use super::BENCHMARK_LOCAL_CORPUS_RUN_SCHEMA_V1;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct BenchmarkCorpusSampleRun {
    pub sample_id: String,
    pub r1: String,
    pub r2: Option<String>,
    pub layout: String,
    pub status: String,
    pub exit_code: i32,
    pub command: Vec<String>,
    pub report_json: String,
    #[serde(flatten)]
    pub extra_fields: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct BenchmarkCorpusRunManifest {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub generated_at_utc: String,
    pub corpus_id: String,
    pub stage_id: String,
    pub scenario_id: String,
    pub sample_scope: String,
    pub tool_kind: String,
    pub platform: String,
    pub tools: Vec<String>,
    pub threads: u32,
    pub jobs: u32,
    pub sample_jobs: usize,
    pub sample_limit: Option<usize>,
    pub dry_run: bool,
    pub config_path: String,
    pub publication_config_path: String,
    pub repo_root: String,
    pub corpus_root: String,
    pub out_root: String,
    pub stage_args: Vec<String>,
    pub samples_total: usize,
    pub samples_failed: usize,
    pub runs: Vec<BenchmarkCorpusSampleRun>,
    #[serde(flatten)]
    pub extra_fields: BTreeMap<String, serde_json::Value>,
}

#[must_use]
pub fn benchmark_local_corpus_run_schema() -> Schema {
    schemars::schema_for!(BenchmarkCorpusRunManifest)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_CORPUS_RUN_SCHEMA_V1,
    })
}
