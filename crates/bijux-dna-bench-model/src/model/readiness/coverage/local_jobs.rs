//! Versioned all domain local job coverage wire contracts.
use schemars::{json_schema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

pub const ALL_DOMAIN_LOCAL_JOB_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_local_job_coverage.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainLocalJobCoverageRow {
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub adapter_id: String,
    pub parser_id: String,
    pub schema_id: String,
    pub readiness_kind: String,
    pub benchmark_status: String,
    pub command_source: String,
    pub command_step_count: usize,
    pub script_command_count: usize,
    pub command_step_ids: Vec<String>,
    pub primary_executables: Vec<String>,
    pub script_output_path: String,
    pub argv_output_path: String,
    pub coverage_status: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainLocalJobCoverageReport {
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
    pub local_job_binding_count: usize,
    pub covered_row_count: usize,
    pub missing_row_count: usize,
    pub coverage_percent: f64,
    pub domain_counts: BTreeMap<String, usize>,
    pub command_source_counts: BTreeMap<String, usize>,
    pub coverage_status_counts: BTreeMap<String, usize>,
    pub violation_count: usize,
    pub ok: bool,
    pub rows: Vec<AllDomainLocalJobCoverageRow>,
    pub violations: Vec<AllDomainLocalJobCoverageRow>,
}

fn deserialize_version<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != ALL_DOMAIN_LOCAL_JOB_COVERAGE_SCHEMA_VERSION {
        return Err(serde::de::Error::custom("unsupported all domain local job coverage schema"));
    }
    Ok(version)
}
fn version_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":ALL_DOMAIN_LOCAL_JOB_COVERAGE_SCHEMA_VERSION})
}
