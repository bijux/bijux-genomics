//! Owner: closed illustrative missing-result audit observations.
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

pub const DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH: &str =
    "artifacts/benchmark-smoke/readiness/missing-result-test-all-domains.json";
pub const DEFAULT_ALL_DOMAIN_MISSING_RESULT_FIXTURE_ROOT: &str =
    "artifacts/benchmark-smoke/simulated-runs/readiness-probes/all-domains/missing-result-test";
pub const ALL_DOMAIN_MISSING_RESULT_TEST_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_missing_result_audit.v1";

pub const FASTQ_REMOVED_RESULT_ID: &str =
    "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2";
pub const BAM_REMOVED_RESULT_ID: &str = "bam:corpus-01-bam-mini:bam.coverage:sample-set:samtools";
pub const VCF_REMOVED_RESULT_ID: &str =
    "vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AllDomainMissingResultStatus {
    Present,
    MissingResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainMissingResultRow {
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub expected_manifest_path: String,
    pub audit_manifest_path: String,
    pub result_status: AllDomainMissingResultStatus,
    pub expected_output_artifact_ids: Vec<String>,
    pub observed_output_artifact_ids: Vec<String>,
    pub expected_metrics: Vec<String>,
    pub report_section: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainMissingResultTestReport {
    #[serde(deserialize_with = "deserialize_version")]
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub fake_result_root: String,
    pub expected_row_count: usize,
    pub present_result_row_count: usize,
    pub missing_result_row_count: usize,
    pub passes_behavior_test: bool,
    pub removed_result_ids: Vec<String>,
    pub removed_manifest_paths: Vec<String>,
    pub domain_counts: BTreeMap<String, usize>,
    pub rows: Vec<AllDomainMissingResultRow>,
}

#[must_use]
pub fn all_domain_missing_result_audit_schema() -> Schema {
    schemars::schema_for!(AllDomainMissingResultTestReport)
}
fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":ALL_DOMAIN_MISSING_RESULT_TEST_SCHEMA_VERSION})
}
fn deserialize_version<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != ALL_DOMAIN_MISSING_RESULT_TEST_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(format!(
            "unsupported missing-result audit schema {version}"
        )));
    }
    Ok(version)
}

#[cfg(test)]
#[path = "../../../tests/contracts/simulation/missing.rs"]
mod tests;
