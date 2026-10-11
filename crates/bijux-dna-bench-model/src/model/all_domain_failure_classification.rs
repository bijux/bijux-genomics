//! Persisted all-domain benchmark failure-classification records.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_V1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainFailureClassificationRow {
    pub class_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub result_id: Option<String>,
    pub source_surface: String,
    pub evidence_path: String,
    pub observed_status: String,
    pub observed_error: String,
    pub detail: String,
    pub triggered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainFailureClassificationReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub fixture_root: String,
    pub row_count: usize,
    pub triggered_row_count: usize,
    pub required_class_count: usize,
    pub triggered_class_count: usize,
    pub missing_class_count: usize,
    pub class_counts: BTreeMap<String, usize>,
    pub missing_class_ids: Vec<String>,
    pub passes_behavior_test: bool,
    pub rows: Vec<AllDomainFailureClassificationRow>,
}

#[must_use]
pub fn all_domain_failure_classification_schema() -> Schema {
    schemars::schema_for!(AllDomainFailureClassificationReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_V1})
}
