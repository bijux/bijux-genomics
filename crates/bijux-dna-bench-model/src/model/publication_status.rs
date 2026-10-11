//! Persisted benchmark publication audit status.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

pub const BENCHMARK_PUBLICATION_STATUS_SCHEMA_V1: &str = "bijux.bench.publication_status.v1";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkPublicationStatusReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub corpus_id: String,
    pub docs_root: String,
    pub benchmarkable_stage_count: usize,
    pub applicable_stage_count: usize,
    pub completed_stage_count: usize,
    pub incomplete_stage_count: usize,
    pub excluded_stage_count: usize,
    pub issue_count: usize,
    pub audit_warning_count: usize,
    pub audit_warnings: Vec<String>,
    pub supplemental_findings_generated_at_utc: Option<String>,
    pub excluded_stages: Vec<BenchmarkPublicationExcludedStage>,
    pub stages: Vec<BenchmarkPublicationStageStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkPublicationExcludedStage {
    pub stage_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkPublicationStageStatus {
    pub stage_id: String,
    pub scenario_id: String,
    pub sample_scope: String,
    pub contract_tool_roster: Vec<String>,
    pub expected_tool_roster: Vec<String>,
    pub method_path: String,
    pub corpus_path: String,
    pub status: String,
    pub issue_count: usize,
    pub results_status: String,
    pub results_issue_count: usize,
    pub results_selected_run_root: String,
    pub results_newest_available_run_root: String,
    pub results_selected_run_root_is_newest: bool,
    pub issues: Vec<BenchmarkPublicationAuditIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkPublicationAuditIssue {
    pub stage_id: String,
    pub issue_id: String,
    pub severity: String,
    pub detail: String,
}

#[must_use]
pub fn benchmark_publication_status_schema() -> Schema {
    schemars::schema_for!(BenchmarkPublicationStatusReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_PUBLICATION_STATUS_SCHEMA_V1,
    })
}
