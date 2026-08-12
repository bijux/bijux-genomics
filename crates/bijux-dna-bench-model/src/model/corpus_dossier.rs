//! Owner: bijux-dna-bench-model
//! Persisted corpus benchmark dossier summary and sample-result contracts.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

pub const BENCHMARK_CORPUS_DOSSIER_SUMMARY_SCHEMA_V1: &str = "bijux.corpus_benchmark.summary.v1";

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCorpusDossierSummary {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub corpus_id: String,
    pub stage_id: String,
    pub scenario_id: String,
    pub generated_at_utc: String,
    pub platform: String,
    pub corpus_root: String,
    pub run_root: String,
    pub sample_scope: String,
    pub samples_total: usize,
    pub samples_failed: usize,
    pub tools: Vec<String>,
    pub cohort_counts: BTreeMap<String, usize>,
    pub era_counts: BTreeMap<String, usize>,
    pub layout_counts: BTreeMap<String, usize>,
    pub tool_summary: Vec<BenchmarkCorpusDossierToolSummary>,
    pub headline: BenchmarkCorpusDossierHeadline,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCorpusDossierToolSummary {
    pub tool: String,
    pub records: usize,
    pub pass_rate: Option<f64>,
    pub median_runtime_s: Option<f64>,
    pub mean_runtime_s: Option<f64>,
    pub max_runtime_s: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCorpusDossierHeadline {
    pub fastest_tool: Option<String>,
    pub fastest_runtime_s: Option<f64>,
    pub highest_pass_rate_tool: Option<String>,
    pub highest_pass_rate: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCorpusDossierSampleResultRow {
    pub sample_id: String,
    pub accession: String,
    pub era: String,
    pub layout: String,
    pub study_accession: String,
    pub size_band: String,
    pub tool: String,
    pub runtime_s: Option<f64>,
    pub exit_code: Option<i64>,
    pub report_json: String,
}

#[must_use]
pub fn benchmark_corpus_dossier_summary_schema() -> Schema {
    schemars::schema_for!(BenchmarkCorpusDossierSummary)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_CORPUS_DOSSIER_SUMMARY_SCHEMA_V1,
    })
}
