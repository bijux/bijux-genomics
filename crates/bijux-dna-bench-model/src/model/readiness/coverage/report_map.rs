//! Versioned all domain report map coverage wire contracts.
use schemars::{json_schema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

pub const ALL_DOMAIN_REPORT_MAP_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_report_map_coverage.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainReportMapCoverageRow {
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub adapter_id: String,
    pub parser_id: String,
    pub schema_id: String,
    pub expected_report_section: String,
    pub report_section_id: String,
    pub summary_table_id: String,
    pub proof_source: String,
    pub coverage_status: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainReportMapCoverageReport {
    #[serde(deserialize_with = "deserialize_version")]
    #[schemars(schema_with = "version_schema")]
    pub schema_version: String,
    pub output_path: String,
    pub row_count: usize,
    pub result_id_count: usize,
    pub stage_count: usize,
    pub tool_count: usize,
    pub report_map_binding_count: usize,
    pub covered_row_count: usize,
    pub missing_row_count: usize,
    pub coverage_percent: f64,
    pub domain_counts: BTreeMap<String, usize>,
    pub proof_source_counts: BTreeMap<String, usize>,
    pub report_section_counts: BTreeMap<String, usize>,
    pub summary_table_counts: BTreeMap<String, usize>,
    pub coverage_status_counts: BTreeMap<String, usize>,
    pub violation_count: usize,
    pub ok: bool,
    pub rows: Vec<AllDomainReportMapCoverageRow>,
    pub violations: Vec<AllDomainReportMapCoverageRow>,
}

fn deserialize_version<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != ALL_DOMAIN_REPORT_MAP_COVERAGE_SCHEMA_VERSION {
        return Err(serde::de::Error::custom("unsupported all domain report map coverage schema"));
    }
    Ok(version)
}
fn version_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":ALL_DOMAIN_REPORT_MAP_COVERAGE_SCHEMA_VERSION})
}
