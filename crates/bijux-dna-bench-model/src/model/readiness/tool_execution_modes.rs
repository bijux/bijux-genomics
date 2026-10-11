//! Persisted runtime-mode readiness report; a declaration check, not executable availability.
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

pub const TOOL_EXECUTION_MODES_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.tool_execution_modes_validation.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolExecutionModeRow {
    pub tool_id: String,
    pub execution_mode: String,
    pub expected_install_kind: String,
    pub domains: Vec<String>,
    pub stage_ids: Vec<String>,
    pub required_runtime_fields: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolExecutionModesValidationReport {
    #[serde(deserialize_with = "deserialize_schema_version")]
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub config_path: String,
    pub classification_scope: String,
    pub mode_count: usize,
    pub tool_count: usize,
    pub multidomain_tool_count: usize,
    pub mode_counts: BTreeMap<String, usize>,
    pub valid: bool,
    pub rows: Vec<ToolExecutionModeRow>,
}

fn deserialize_schema_version<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != TOOL_EXECUTION_MODES_VALIDATION_SCHEMA_VERSION {
        return Err(serde::de::Error::custom("unsupported tool execution mode validation schema"));
    }
    Ok(version)
}
fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const": TOOL_EXECUTION_MODES_VALIDATION_SCHEMA_VERSION})
}
