//! Versioned all domain parser fixture coverage wire contracts.
use schemars::{json_schema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

pub const ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_parser_fixture_coverage.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainParserFixtureCoverageRow {
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub adapter_id: String,
    pub parser_id: String,
    pub schema_id: String,
    pub parser_fixture_parser_id: String,
    pub parser_fixture_schema_id: String,
    pub parser_fixture_reference_kind: String,
    pub parser_fixture_reference: String,
    pub proof_source: String,
    pub coverage_status: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainParserFixtureCoverageReport {
    #[serde(deserialize_with = "deserialize_version")]
    #[schemars(schema_with = "version_schema")]
    pub schema_version: String,
    pub output_path: String,
    pub row_count: usize,
    pub stage_count: usize,
    pub tool_count: usize,
    pub parser_proof_binding_count: usize,
    pub covered_row_count: usize,
    pub missing_row_count: usize,
    pub coverage_percent: f64,
    pub domain_counts: BTreeMap<String, usize>,
    pub proof_source_counts: BTreeMap<String, usize>,
    pub coverage_status_counts: BTreeMap<String, usize>,
    pub violation_count: usize,
    pub ok: bool,
    pub rows: Vec<AllDomainParserFixtureCoverageRow>,
    pub violations: Vec<AllDomainParserFixtureCoverageRow>,
}

fn deserialize_version<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(
            "unsupported all domain parser fixture coverage schema",
        ));
    }
    Ok(version)
}
fn version_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION})
}
