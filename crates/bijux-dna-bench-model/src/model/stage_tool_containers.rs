//! Persisted stage-tool benchmark runtime declarations and readiness report.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{LOCAL_STAGE_TOOL_CONTAINERS_SCHEMA_V1, STAGE_TOOL_CONTAINERS_REPORT_SCHEMA_V1};

pub const STAGE_TOOL_CONTAINERS_SCOPE: &str = "benchmark_ready_runtime_declarations";
pub const STAGE_TOOL_CONTAINERS_DECLARATION_ORIGIN: &str =
    "runtime_probe_with_primary_operator_runtime";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageToolContainerRow {
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub execution_mode: String,
    pub install_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_entrypoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_binary_mode: Option<String>,
    pub declaration_origin: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageToolContainersConfig {
    #[schemars(schema_with = "config_schema_version")]
    pub schema_version: String,
    pub classification_scope: String,
    pub rows: Vec<StageToolContainerRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageToolContainersReport {
    #[schemars(schema_with = "report_schema_version")]
    pub schema_version: String,
    pub config_path: String,
    pub classification_scope: String,
    pub row_count: usize,
    pub benchmark_ready_row_count: usize,
    pub external_row_count: usize,
    pub container_declared_row_count: usize,
    pub command_entrypoint_row_count: usize,
    pub host_binary_row_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub execution_mode_counts: BTreeMap<String, usize>,
    pub rows: Vec<StageToolContainerRow>,
}

#[must_use]
pub fn stage_tool_containers_config_schema() -> Schema {
    schemars::schema_for!(StageToolContainersConfig)
}

#[must_use]
pub fn stage_tool_containers_report_schema() -> Schema {
    schemars::schema_for!(StageToolContainersReport)
}

fn config_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": LOCAL_STAGE_TOOL_CONTAINERS_SCHEMA_V1})
}

fn report_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": STAGE_TOOL_CONTAINERS_REPORT_SCHEMA_V1})
}
