//! Owner: bijux-dna-bench-model
//! Persisted benchmark stage/tool scope and catalog-drift views.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::{
    ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_V1, ALL_DOMAIN_RETAINED_TOOLS_SCHEMA_V1,
    STAGE_REGISTRY_EXTRA_PAIRS_SCHEMA_V1,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainActiveStageToolMatrixRow {
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub adapter_id: String,
    pub parser_id: String,
    pub schema_id: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainActiveStageToolMatrixReport {
    #[schemars(schema_with = "active_stage_tool_matrix_schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub row_count: usize,
    pub stage_count: usize,
    pub tool_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub status_counts: BTreeMap<String, usize>,
    pub rows: Vec<AllDomainActiveStageToolMatrixRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainRetainedToolRow {
    pub tool_id: String,
    pub domains: Vec<String>,
    pub active_stage_count: usize,
    pub benchmark_ready_stage_count: usize,
    pub active_binding_count: usize,
    pub benchmark_ready_binding_count: usize,
    pub benchmark_statuses: Vec<String>,
    pub active_stage_ids: Vec<String>,
    pub benchmark_ready_stage_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainRetainedToolsReport {
    #[schemars(schema_with = "retained_tools_schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub row_count: usize,
    pub active_matrix_tool_count: usize,
    pub benchmark_ready_tool_count: usize,
    pub mixed_status_tool_count: usize,
    pub not_benchmark_ready_only_tool_count: usize,
    pub domain_tool_counts: BTreeMap<String, usize>,
    pub rows: Vec<AllDomainRetainedToolRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageRegistryExtraPairRow {
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub contract_status: String,
    pub registry_sources: Vec<String>,
    pub registered_stage_ids: Vec<String>,
    pub documented_exception_status: String,
    pub documented_exception_reason: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageRegistryExtraPairsReport {
    #[schemars(schema_with = "stage_registry_extra_pairs_schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub extra_pair_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub ok: bool,
    pub rows: Vec<StageRegistryExtraPairRow>,
}

#[must_use]
pub fn all_domain_active_stage_tool_matrix_schema() -> Schema {
    schemars::schema_for!(AllDomainActiveStageToolMatrixReport)
}

#[must_use]
pub fn all_domain_retained_tools_schema() -> Schema {
    schemars::schema_for!(AllDomainRetainedToolsReport)
}

#[must_use]
pub fn stage_registry_extra_pairs_schema() -> Schema {
    schemars::schema_for!(StageRegistryExtraPairsReport)
}

fn active_stage_tool_matrix_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_V1})
}

fn retained_tools_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": ALL_DOMAIN_RETAINED_TOOLS_SCHEMA_V1})
}

fn stage_registry_extra_pairs_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": STAGE_REGISTRY_EXTRA_PAIRS_SCHEMA_V1})
}
