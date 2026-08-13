//! Persisted all-domain benchmark output declarations.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::ALL_DOMAIN_OUTPUT_DECLARATIONS_SCHEMA_V1;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AllDomainOutputDeclarationStatus {
    Complete,
    Incomplete,
    MissingAdapter,
}

impl AllDomainOutputDeclarationStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Incomplete => "incomplete",
            Self::MissingAdapter => "missing_adapter",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainOutputDeclarationRow {
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub raw_outputs: Vec<String>,
    pub normalized_metrics: Vec<String>,
    pub logs: Vec<String>,
    pub manifest: String,
    pub index_outputs: Vec<String>,
    pub status: AllDomainOutputDeclarationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainOutputDeclarationsReport {
    #[schemars(schema_with = "report_schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub row_count: usize,
    pub result_id_count: usize,
    pub complete_row_count: usize,
    pub incomplete_row_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub status_counts: BTreeMap<String, usize>,
    pub rows: Vec<AllDomainOutputDeclarationRow>,
}

#[must_use]
pub fn all_domain_output_declarations_schema() -> Schema {
    schemars::schema_for!(AllDomainOutputDeclarationsReport)
}

fn report_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": ALL_DOMAIN_OUTPUT_DECLARATIONS_SCHEMA_V1})
}
