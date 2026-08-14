//! Owner: bijux-dna-bench-model
//! Persisted VCF stage-catalog declarations and render reports.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::{
    LOCAL_VCF_STAGE_CATALOG_REPORT_SCHEMA_V1, LOCAL_VCF_STAGE_CATALOG_SCHEMA_V1,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LocalVcfStageCatalogConfig {
    #[schemars(schema_with = "config_schema_version")]
    pub schema_version: String,
    pub rows: Vec<VcfStageCatalogRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VcfStageCatalogRow {
    pub stage_id: String,
    pub stage_name: String,
    pub support_status: String,
    pub default_tool_id: String,
    pub metrics_schema_id: String,
    pub input_types: Vec<String>,
    pub output_types: Vec<String>,
    pub required_assets: Vec<String>,
    pub benchmark_category: String,
    pub local_smoke_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LocalVcfStageCatalogReport {
    #[schemars(schema_with = "report_schema_version")]
    pub schema_version: String,
    pub config_path: String,
    pub stage_count: usize,
    pub supported_stage_count: usize,
    pub planned_stage_count: usize,
    pub rows: Vec<VcfStageCatalogRow>,
}

#[must_use]
pub fn local_vcf_stage_catalog_config_schema() -> Schema {
    schemars::schema_for!(LocalVcfStageCatalogConfig)
}

#[must_use]
pub fn local_vcf_stage_catalog_report_schema() -> Schema {
    schemars::schema_for!(LocalVcfStageCatalogReport)
}

fn config_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": LOCAL_VCF_STAGE_CATALOG_SCHEMA_V1})
}

fn report_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": LOCAL_VCF_STAGE_CATALOG_REPORT_SCHEMA_V1})
}
