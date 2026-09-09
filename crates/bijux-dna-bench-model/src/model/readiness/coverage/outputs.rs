//! Versioned all domain output contract coverage wire contracts.
use schemars::{json_schema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

pub const ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_output_contract_coverage.v1";

// Independent persisted observations retain their existing boolean wire fields.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainOutputContractCoverageRow {
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub adapter_id: String,
    pub proof_source: String,
    pub source_contract_status: String,
    pub output_declaration_status: String,
    pub raw_output_ids: Vec<String>,
    pub normalized_metric_ids: Vec<String>,
    pub log_declarations: Vec<String>,
    pub manifest: String,
    pub index_output_ids: Vec<String>,
    pub raw_outputs_declared: bool,
    pub normalized_metrics_declared: bool,
    pub logs_declared: bool,
    pub manifest_declared: bool,
    pub index_coverage_status: String,
    pub coverage_status: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainOutputContractCoverageReport {
    #[serde(deserialize_with = "deserialize_version")]
    #[schemars(schema_with = "version_schema")]
    pub schema_version: String,
    pub output_path: String,
    pub row_count: usize,
    pub result_id_count: usize,
    pub stage_count: usize,
    pub tool_count: usize,
    pub output_declaration_binding_count: usize,
    pub source_proof_binding_count: usize,
    pub covered_row_count: usize,
    pub missing_row_count: usize,
    pub coverage_percent: f64,
    pub raw_output_declared_row_count: usize,
    pub normalized_metrics_declared_row_count: usize,
    pub logs_declared_row_count: usize,
    pub manifest_declared_row_count: usize,
    pub index_required_row_count: usize,
    pub index_declared_row_count: usize,
    pub index_not_applicable_row_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub proof_source_counts: BTreeMap<String, usize>,
    pub index_coverage_counts: BTreeMap<String, usize>,
    pub coverage_status_counts: BTreeMap<String, usize>,
    pub violation_count: usize,
    pub ok: bool,
    pub rows: Vec<AllDomainOutputContractCoverageRow>,
    pub violations: Vec<AllDomainOutputContractCoverageRow>,
}

fn deserialize_version<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(
            "unsupported all domain output contract coverage schema",
        ));
    }
    Ok(version)
}
fn version_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_SCHEMA_VERSION})
}
