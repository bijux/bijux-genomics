//! Owner: bijux-dna-bench-model
//! Persisted local VCF genotype-likelihood calling smoke records.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

pub const BENCHMARK_LOCAL_VCF_CALL_GL_SMOKE_SCHEMA_V1: &str =
    "bijux.bench.local_vcf_call_gl_smoke.v1";
pub const BENCHMARK_LOCAL_VCF_CALL_GL_SMOKE_METRICS_SCHEMA_V1: &str =
    "bijux.bench.local_vcf_call_gl_smoke.metrics.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalVcfCallGlSmokeMetrics {
    #[schemars(schema_with = "metrics_schema_version")]
    pub schema_version: String,
    pub stage_id: String,
    pub corpus_id: String,
    pub sample_id: String,
    pub input_bam: String,
    pub reference: String,
    pub likelihood_field: String,
    pub sites_with_likelihoods: u64,
    pub samples_with_likelihoods: u64,
    pub missing_likelihoods: u64,
    pub sample_count: u64,
    pub tool_id: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalVcfCallGlSmokeReport {
    #[schemars(schema_with = "report_schema_version")]
    pub schema_version: String,
    pub command: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub sample_id: String,
    pub sample_name: String,
    pub input_bam: String,
    pub input_bam_index: String,
    pub reference: String,
    pub materialized_reference_path: String,
    pub output_root: String,
    pub output_vcf_path: String,
    pub output_tbi_path: String,
    pub metrics_path: String,
    pub stage_result_manifest_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub likelihood_field: String,
    pub sites_with_likelihoods: u64,
    pub samples_with_likelihoods: u64,
    pub missing_likelihoods: u64,
    pub sample_count: u64,
    pub parseable: bool,
    pub validation_checks: BTreeMap<String, bool>,
    pub gt_present: bool,
    pub gl_present: bool,
}

#[must_use]
pub fn benchmark_local_vcf_call_gl_smoke_metrics_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalVcfCallGlSmokeMetrics)
}

#[must_use]
pub fn benchmark_local_vcf_call_gl_smoke_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalVcfCallGlSmokeReport)
}

fn metrics_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_VCF_CALL_GL_SMOKE_METRICS_SCHEMA_V1
    })
}

fn report_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_VCF_CALL_GL_SMOKE_SCHEMA_V1
    })
}
