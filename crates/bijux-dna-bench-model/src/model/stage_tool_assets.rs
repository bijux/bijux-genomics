//! Persisted stage-tool benchmark asset configuration and readiness report.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{LOCAL_STAGE_TOOL_ASSETS_SCHEMA_V1, STAGE_TOOL_ASSETS_REPORT_SCHEMA_V1};

pub const STAGE_TOOL_ASSETS_SCOPE: &str = "governed_benchmark_command_assets";
pub const STAGE_TOOL_ASSETS_DECLARATION_ORIGIN: &str =
    "canonical_local_stage_plan_inputs_and_governed_stage_params";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageToolAssetRow {
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub asset_role: String,
    pub asset_id: String,
    pub asset_path: String,
    pub declaration_origin: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageToolAssetsConfig {
    #[schemars(schema_with = "config_schema_version")]
    pub schema_version: String,
    pub classification_scope: String,
    pub rows: Vec<StageToolAssetRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageToolAssetsReport {
    #[schemars(schema_with = "report_schema_version")]
    pub schema_version: String,
    pub config_path: String,
    pub classification_scope: String,
    pub row_count: usize,
    pub declared_stage_tool_row_count: usize,
    pub asset_id_row_count: usize,
    pub unique_asset_id_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub asset_role_counts: BTreeMap<String, usize>,
    pub rows: Vec<StageToolAssetRow>,
}

#[must_use]
pub fn stage_tool_assets_config_schema() -> Schema {
    schemars::schema_for!(StageToolAssetsConfig)
}

#[must_use]
pub fn stage_tool_assets_report_schema() -> Schema {
    schemars::schema_for!(StageToolAssetsReport)
}

fn config_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": LOCAL_STAGE_TOOL_ASSETS_SCHEMA_V1})
}

fn report_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": STAGE_TOOL_ASSETS_REPORT_SCHEMA_V1})
}
