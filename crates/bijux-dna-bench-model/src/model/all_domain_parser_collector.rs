//! Owner: bijux-dna-bench-model
//! Persisted all-domain parser-collector evidence.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::contract::ALL_DOMAIN_PARSER_COLLECTOR_SCHEMA_V1;

pub const ALL_DOMAIN_PARSER_COLLECTOR_OUTPUT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/parser-collector-all-domains.json";
pub const ALL_DOMAIN_PARSER_COLLECTOR_FIXTURE_ROOT: &str =
    "artifacts/benchmark-smoke/simulated-runs/readiness-probes/all-domains/parser-collector";
pub const ALL_DOMAIN_PARSER_COLLECTOR_FAKE_RUN_ROOT: &str =
    "artifacts/benchmark-smoke/simulated-runs/readiness-probes/all-domains/parser-collector/fake-runs";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AllDomainParserCollectorSourceKind {
    FakeRun,
    RealSmoke,
}

impl AllDomainParserCollectorSourceKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FakeRun => "fake_run",
            Self::RealSmoke => "real_smoke",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainParserCollectorRow {
    pub record_id: String,
    pub source_kind: AllDomainParserCollectorSourceKind,
    pub document_kind: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub result_id: Option<String>,
    pub parsed_path: String,
    pub parsed_schema_version: String,
    pub parsed_top_level_key_count: usize,
    pub parsed_top_level_keys: Vec<String>,
    pub manifest_path: Option<String>,
    pub manifest_status: Option<String>,
    pub manifest_exit_code: Option<i32>,
    pub declared_output_count: usize,
    pub normalized_snapshot: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainParserCollectorReport {
    #[schemars(schema_with = "all_domain_parser_collector_schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub fixture_root: String,
    pub fake_run_root: String,
    pub row_count: usize,
    pub fake_run_row_count: usize,
    pub real_smoke_row_count: usize,
    pub source_kind_counts: BTreeMap<String, usize>,
    pub domain_counts: BTreeMap<String, usize>,
    pub document_kind_counts: BTreeMap<String, usize>,
    pub rows: Vec<AllDomainParserCollectorRow>,
}

#[must_use]
pub fn all_domain_parser_collector_schema() -> Schema {
    schemars::schema_for!(AllDomainParserCollectorReport)
}

fn all_domain_parser_collector_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": ALL_DOMAIN_PARSER_COLLECTOR_SCHEMA_V1})
}
