//! Persisted stage-tool benchmark resource configuration and readiness report.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_V1, STAGE_TOOL_RESOURCES_REPORT_SCHEMA_V1};

pub const STAGE_TOOL_RESOURCES_SCOPE: &str = "benchmark_ready_command_resources";
pub const FASTQ_STAGE_TOOL_RESOURCE_ORIGIN: &str = "tool_constraints_with_stage_walltime_profile";
pub const BAM_STAGE_TOOL_RESOURCE_ORIGIN: &str = "tool_constraints_with_tiny_bam_stage_estimate";
pub const VCF_STAGE_TOOL_RESOURCE_ORIGIN: &str =
    "planner_stage_constraints_with_stage_walltime_profile";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageToolResourceRow {
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub threads: u32,
    pub memory_gb: u32,
    pub walltime_minutes: u32,
    pub scratch_gb: u32,
    pub resource_origin: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageToolResourcesConfig {
    #[schemars(schema_with = "config_schema_version")]
    pub schema_version: String,
    pub classification_scope: String,
    pub rows: Vec<StageToolResourceRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageToolResourcesReport {
    #[schemars(schema_with = "report_schema_version")]
    pub schema_version: String,
    pub config_path: String,
    pub classification_scope: String,
    pub row_count: usize,
    pub benchmark_ready_row_count: usize,
    pub nonzero_resource_row_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub resource_origin_counts: BTreeMap<String, usize>,
    pub rows: Vec<StageToolResourceRow>,
}

#[must_use]
pub fn stage_tool_resources_config_schema() -> Schema {
    schemars::schema_for!(StageToolResourcesConfig)
}

#[must_use]
pub fn stage_tool_resources_report_schema() -> Schema {
    schemars::schema_for!(StageToolResourcesReport)
}

fn config_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_V1})
}

fn report_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": STAGE_TOOL_RESOURCES_REPORT_SCHEMA_V1})
}
