//! Closed illustrative completion-report wire contracts.
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

pub const DEFAULT_ALL_DOMAIN_COMPLETION_CHECK_PATH: &str =
    "artifacts/benchmark-smoke/readiness/completion-check-all-domains.json";
pub const DEFAULT_ALL_DOMAIN_COMPLETION_CHECK_FIXTURE_ROOT: &str =
    "artifacts/benchmark-smoke/simulated-runs/readiness-probes/all-domains/completion-check";
pub const ALL_DOMAIN_COMPLETION_CHECK_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_completion_check.v1";

pub const MUTATION_MISSING_DECLARED_OUTPUT: &str = "missing_declared_output";
pub const MUTATION_MISSING_NORMALIZED_METRICS: &str = "missing_normalized_metrics";
pub const MUTATION_MISSING_MANIFEST: &str = "missing_manifest";
pub const MUTATION_REQUIRED_FILE_EMPTY: &str = "required_file_empty";
pub const MUTATION_EXECUTION_NOT_SUCCESSFUL: &str = "execution_not_successful";

pub const REASON_MISSING_MANIFEST: &str = "missing_manifest";
pub const REASON_INVALID_MANIFEST: &str = "invalid_manifest";
pub const REASON_REQUIRED_FILES_INCOMPLETE: &str = "required_files_incomplete";
pub const REASON_MISSING_DECLARED_OUTPUTS: &str = "missing_declared_outputs";
pub const REASON_MISSING_NORMALIZED_METRICS: &str = "missing_normalized_metrics";
pub const REASON_EXECUTION_NOT_SUCCESSFUL: &str = "execution_not_successful";
pub const REASON_MANIFEST_OUTPUT_MISMATCH: &str = "manifest_output_mismatch";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AllDomainCompletionStatus {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainCompletionRequiredFile {
    pub file_id: String,
    pub path: String,
    pub exists: bool,
    pub non_empty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainCompletionSeededMutation {
    pub mutation_id: String,
    pub result_id: String,
    pub evidence_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
// Persisted protocol flags retain their wire shape; admission enforces their relationships.
#[allow(clippy::struct_excessive_bools)]
pub struct AllDomainCompletionCheckRow {
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub command_script_path: String,
    pub stdout_path: String,
    pub stderr_path: String,
    pub metrics_path: String,
    pub stage_result_path: String,
    pub manifest_exists: bool,
    pub manifest_valid: bool,
    pub manifest_output_match: bool,
    pub exit_code_zero: bool,
    pub runtime_status: Option<String>,
    pub exit_code: Option<i32>,
    pub declared_output_count: usize,
    pub present_declared_output_count: usize,
    pub missing_declared_output_ids: Vec<String>,
    pub normalized_metrics_count: usize,
    pub present_normalized_metrics_count: usize,
    pub missing_normalized_metric_ids: Vec<String>,
    pub required_file_count: usize,
    pub ready_required_file_count: usize,
    pub required_files: Vec<AllDomainCompletionRequiredFile>,
    pub completion_status: AllDomainCompletionStatus,
    pub failure_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainCompletionCheckReport {
    #[serde(deserialize_with = "deserialize_version")]
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub fixture_root: String,
    pub row_count: usize,
    pub complete_row_count: usize,
    pub incomplete_row_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub failure_reason_counts: BTreeMap<String, usize>,
    pub passes_behavior_test: bool,
    pub seeded_mutations: Vec<AllDomainCompletionSeededMutation>,
    pub rows: Vec<AllDomainCompletionCheckRow>,
}

#[must_use]
pub fn all_domain_completion_check_schema() -> Schema {
    schemars::schema_for!(AllDomainCompletionCheckReport)
}
fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":ALL_DOMAIN_COMPLETION_CHECK_SCHEMA_VERSION})
}
fn deserialize_version<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != ALL_DOMAIN_COMPLETION_CHECK_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(format!("unsupported completion schema {version}")));
    }
    Ok(version)
}
#[cfg(test)]
#[path = "../../../tests/contracts/simulation/completion.rs"]
mod tests;
