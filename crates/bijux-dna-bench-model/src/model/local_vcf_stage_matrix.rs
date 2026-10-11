//! Owner: bijux-dna-bench-model
//! Persisted VCF stage-matrix declarations and validation reports.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::{
    LOCAL_VCF_STAGE_MATRIX_REPORT_SCHEMA_V1, LOCAL_VCF_STAGE_MATRIX_SCHEMA_V1,
    VCF_STAGE_MATRIX_VALIDATION_SCHEMA_V1,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LocalVcfStageMatrixConfig {
    #[schemars(schema_with = "config_schema_version")]
    pub schema_version: String,
    pub rows: Vec<VcfStageMatrixRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VcfStageMatrixRow {
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub adapter_id: String,
    pub parser_id: String,
    pub expected_outputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LocalVcfStageMatrixReport {
    #[schemars(schema_with = "report_schema_version")]
    pub schema_version: String,
    pub config_path: String,
    pub row_count: usize,
    pub stage_count: usize,
    pub supported_stage_count: usize,
    pub planned_stage_count: usize,
    pub rows: Vec<VcfStageMatrixRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValidateVcfStageMatrixReport {
    #[schemars(schema_with = "validation_schema_version")]
    pub schema_version: String,
    pub domain: String,
    pub matrix_path: String,
    pub strict: bool,
    pub row_count: usize,
    pub stage_count: usize,
    pub required_tool_count: usize,
    pub registry_tool_count: usize,
    pub rows: Vec<VcfStageMatrixRow>,
}

#[must_use]
pub fn local_vcf_stage_matrix_config_schema() -> Schema {
    schemars::schema_for!(LocalVcfStageMatrixConfig)
}

#[must_use]
pub fn local_vcf_stage_matrix_report_schema() -> Schema {
    schemars::schema_for!(LocalVcfStageMatrixReport)
}

#[must_use]
pub fn validate_vcf_stage_matrix_report_schema() -> Schema {
    schemars::schema_for!(ValidateVcfStageMatrixReport)
}

fn config_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": LOCAL_VCF_STAGE_MATRIX_SCHEMA_V1})
}

fn report_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": LOCAL_VCF_STAGE_MATRIX_REPORT_SCHEMA_V1})
}

fn validation_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": VCF_STAGE_MATRIX_VALIDATION_SCHEMA_V1})
}
