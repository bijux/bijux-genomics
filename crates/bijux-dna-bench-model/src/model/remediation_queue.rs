//! Persisted benchmark remediation queue.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

pub const BENCHMARK_REMEDIATION_QUEUE_SCHEMA_V1: &str = "bijux.bench.remediation_queue.v1";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkRemediationQueue {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub corpus_id: String,
    pub stage_count: usize,
    pub open_stage_count: usize,
    pub clear_stage_count: usize,
    pub stages: Vec<BenchmarkRemediationStageEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkRemediationStageEntry {
    pub stage_id: String,
    pub owner: String,
    pub status: String,
    pub issue_count: usize,
    pub issue_group_count: usize,
    pub recommended_action: String,
    pub publication_status: String,
    pub results_status: String,
    pub sample_scope: String,
    pub published_generated_at_utc: Option<String>,
    pub run_root_source: Option<String>,
    pub issue_groups: Vec<BenchmarkRemediationIssueGroup>,
    pub issues: Vec<BenchmarkRemediationIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkRemediationIssue {
    pub issue_id: String,
    pub detail: String,
    pub severity: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkRemediationIssueGroup {
    pub issue_id: String,
    pub count: usize,
    pub sources: Vec<String>,
    pub severity: String,
    pub example_details: Vec<String>,
    pub additional_detail_count: usize,
}

#[must_use]
pub fn benchmark_remediation_queue_schema() -> Schema {
    schemars::schema_for!(BenchmarkRemediationQueue)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_REMEDIATION_QUEUE_SCHEMA_V1,
    })
}
