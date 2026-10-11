//! Versioned removed from scope wire contracts.
use schemars::{json_schema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

pub const REMOVED_FROM_SCOPE_SCHEMA_VERSION: &str = "bijux.bench.readiness.removed_from_scope.v1";

// Independent persisted observations retain their existing boolean wire fields.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RemovedFromScopeRow {
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub adapter_id: String,
    pub parser_id: String,
    pub schema_id: String,
    pub status: String,
    pub adapter_status: String,
    pub scope_exit_kind: String,
    pub stage_removed_from_active_scope: bool,
    pub tool_removed_from_active_scope: bool,
    pub absent_from_active_matrix: bool,
    pub absent_from_rendered_commands: bool,
    pub absent_from_expected_results: bool,
    pub absent_from_full_benchmark_report: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RemovedFromScopeReport {
    #[serde(deserialize_with = "deserialize_version")]
    #[schemars(schema_with = "version_schema")]
    pub schema_version: String,
    pub output_path: String,
    pub full_benchmark_report_check_source: String,
    pub candidate_row_count: usize,
    pub candidate_stage_count: usize,
    pub candidate_tool_count: usize,
    pub active_row_count: usize,
    pub active_stage_count: usize,
    pub active_tool_count: usize,
    pub removed_row_count: usize,
    pub removed_stage_count: usize,
    pub removed_tool_count: usize,
    pub fully_removed_stage_count: usize,
    pub fully_removed_tool_count: usize,
    pub scope_exit_kind_counts: BTreeMap<String, usize>,
    pub violation_count: usize,
    pub ok: bool,
    pub rows: Vec<RemovedFromScopeRow>,
    pub violations: Vec<RemovedFromScopeRow>,
}

fn deserialize_version<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != REMOVED_FROM_SCOPE_SCHEMA_VERSION {
        return Err(serde::de::Error::custom("unsupported removed from scope schema"));
    }
    Ok(version)
}
fn version_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":REMOVED_FROM_SCOPE_SCHEMA_VERSION})
}
