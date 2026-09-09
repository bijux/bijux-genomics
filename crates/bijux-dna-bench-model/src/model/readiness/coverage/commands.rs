//! Versioned all domain no placeholder command check wire contracts.
use schemars::{json_schema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

pub const ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_no_placeholder_command_check.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainNoPlaceholderCommandStepAudit {
    pub step_id: String,
    pub step_kind: String,
    pub executable: String,
    pub is_shell_wrapper: bool,
    pub command_heads: Vec<String>,
    pub has_real_invocation: bool,
    pub finding_types: Vec<String>,
    pub ok: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainNoPlaceholderCommandRow {
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub command_source: String,
    pub command_step_count: usize,
    pub script_command_count: usize,
    pub has_real_invocation: bool,
    pub finding_count: usize,
    pub finding_types: Vec<String>,
    pub ok: bool,
    pub step_audits: Vec<AllDomainNoPlaceholderCommandStepAudit>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainNoPlaceholderCommandCheckReport {
    #[serde(deserialize_with = "deserialize_version")]
    #[schemars(schema_with = "version_schema")]
    pub schema_version: String,
    pub output_path: String,
    pub script_output_path: String,
    pub argv_output_path: String,
    pub row_count: usize,
    pub result_id_count: usize,
    pub stage_count: usize,
    pub tool_count: usize,
    pub command_step_count: usize,
    pub shell_wrapped_step_count: usize,
    pub direct_step_count: usize,
    pub valid_row_count: usize,
    pub invalid_row_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub command_source_counts: BTreeMap<String, usize>,
    pub finding_type_counts: BTreeMap<String, usize>,
    pub violation_count: usize,
    pub ok: bool,
    pub rows: Vec<AllDomainNoPlaceholderCommandRow>,
    pub violations: Vec<AllDomainNoPlaceholderCommandRow>,
}

fn deserialize_version<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(
            "unsupported all domain no placeholder command check schema",
        ));
    }
    Ok(version)
}
fn version_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_SCHEMA_VERSION})
}
