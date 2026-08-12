//! Persisted status of published benchmark result mirrors.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

pub const BENCHMARK_PUBLISHED_RESULTS_STATUS_SCHEMA_V1: &str =
    "bijux.bench.published_results_status.v1";

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PublishedResultsStatusReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub corpus_id: String,
    pub applicable_stage_count: usize,
    pub published_stage_count: usize,
    pub complete_stage_count: usize,
    pub incomplete_stage_count: usize,
    pub issue_count: usize,
    pub stages: Vec<PublishedResultsStageReport>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PublishedResultsStageReport {
    pub stage_id: String,
    pub status: String,
    pub issue_count: usize,
    #[serde(default)]
    pub reported_run_root: String,
    #[serde(default)]
    pub selected_run_root: String,
    #[serde(default)]
    pub newest_available_run_root: String,
    #[serde(default)]
    pub selected_run_root_is_newest: bool,
    #[serde(default)]
    pub available_run_roots: Vec<String>,
    pub issues: Vec<StageResultIssue>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageResultIssue {
    pub stage_id: String,
    pub issue_id: String,
    pub detail: String,
}

#[must_use]
pub fn benchmark_published_results_status_schema() -> Schema {
    schemars::schema_for!(PublishedResultsStatusReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_PUBLISHED_RESULTS_STATUS_SCHEMA_V1,
    })
}
