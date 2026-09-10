//! Owner: governed execution-mode declarations and admitted tool assignments.
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::Deserialize;

pub const LOCAL_TOOL_EXECUTION_MODES_SCHEMA_VERSION: &str =
    "bijux.bench.local_tool_execution_modes.v1";
pub(crate) const TOOL_EXECUTION_CLASSIFICATION_SCOPE: &str = "primary_operator_runtime";

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolExecutionModesConfig {
    #[schemars(schema_with = "schema_version")]
    pub(crate) schema_version: String,
    #[schemars(schema_with = "classification_scope")]
    pub(crate) classification_scope: String,
    pub(crate) modes: Vec<ToolExecutionModeDefinition>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct ToolExecutionModeDefinition {
    pub(crate) execution_mode: String,
    pub(crate) expected_install_kind: String,
    pub(crate) required_runtime_fields: Vec<String>,
    pub(crate) summary: String,
    pub(crate) tool_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolExecutionModeAssignment {
    pub execution_mode: String,
    pub expected_install_kind: String,
    pub required_runtime_fields: Vec<String>,
    pub summary: String,
}

impl ToolExecutionModesConfig {
    #[must_use]
    pub fn classification_scope(&self) -> &str {
        &self.classification_scope
    }
    #[must_use]
    pub fn mode_count(&self) -> usize {
        self.modes.len()
    }
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":LOCAL_TOOL_EXECUTION_MODES_SCHEMA_VERSION})
}
fn classification_scope(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":TOOL_EXECUTION_CLASSIFICATION_SCOPE})
}
