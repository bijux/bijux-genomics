//! Persisted all-domain expected benchmark result inventory.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_SCHEMA_V1;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainExpectedBenchmarkResultRow {
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub expected_outputs: Vec<String>,
    pub expected_metrics: Vec<String>,
    pub report_section: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainExpectedBenchmarkResultsReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub row_count: usize,
    pub result_id_count: usize,
    pub stage_count: usize,
    pub tool_count: usize,
    pub corpus_count: usize,
    pub asset_profile_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub report_section_counts: BTreeMap<String, usize>,
    pub rows: Vec<AllDomainExpectedBenchmarkResultRow>,
}

#[must_use]
pub fn all_domain_expected_benchmark_results_schema() -> Schema {
    schemars::schema_for!(AllDomainExpectedBenchmarkResultsReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_SCHEMA_V1})
}
