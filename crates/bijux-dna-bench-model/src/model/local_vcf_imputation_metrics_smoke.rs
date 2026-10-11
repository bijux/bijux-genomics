//! Owner: bijux-dna-bench-model
//! Persisted local VCF imputation-metrics smoke records.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

pub const BENCHMARK_LOCAL_VCF_IMPUTATION_METRICS_SMOKE_SCHEMA_V1: &str =
    "bijux.bench.local_vcf_imputation_metrics_smoke.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct BenchmarkImputationQualityFieldAvailability {
    pub concordance: bool,
    pub dosage_r2: bool,
    pub maf_strata: bool,
    pub masked_truth_sites: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalVcfImputationMetricsSmokeReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub command: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub panel_id: String,
    pub map_id: String,
    pub output_root: String,
    pub imputation_metrics_path: String,
    pub source_imputation_qc_path: String,
    pub source_impute_smoke_metrics_path: String,
    pub source_imputation_manifest_path: String,
    pub stage_result_manifest_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub concordance: Option<f64>,
    pub mean_info_score: f64,
    pub r2_available: bool,
    pub dosage_r2: Option<f64>,
    pub low_confidence_sites: u64,
    pub masked_truth_sites: u64,
    pub quality_field_availability: BenchmarkImputationQualityFieldAvailability,
    pub missing_quality_fields: Vec<String>,
    pub status: String,
}

#[must_use]
pub fn benchmark_local_vcf_imputation_metrics_smoke_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalVcfImputationMetricsSmokeReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_VCF_IMPUTATION_METRICS_SMOKE_SCHEMA_V1
    })
}
