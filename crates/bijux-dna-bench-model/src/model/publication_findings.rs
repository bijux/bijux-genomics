//! Persisted supplemental findings for benchmark publication audits.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::BenchmarkPublicationAuditIssue;

pub const BENCHMARK_PUBLICATION_FINDINGS_SCHEMA_V1: &str = "bijux.bench.publication_findings.v1";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkPublicationFindings {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub corpus_id: String,
    pub generated_at_utc: String,
    pub findings: Vec<BenchmarkPublicationAuditIssue>,
}

#[must_use]
pub fn benchmark_publication_findings_schema() -> Schema {
    schemars::schema_for!(BenchmarkPublicationFindings)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_PUBLICATION_FINDINGS_SCHEMA_V1,
    })
}
