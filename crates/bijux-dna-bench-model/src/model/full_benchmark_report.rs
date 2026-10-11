//! Persisted aggregate FASTQ, BAM, and VCF benchmark reports.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{AllDomainFailureClassificationRow, FULL_BENCHMARK_REPORT_SCHEMA_V1};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FullBenchmarkReportRowStatus {
    Present,
    MissingResult,
    UnsupportedPair,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkReportRow {
    pub report_row_id: String,
    pub row_status: FullBenchmarkReportRowStatus,
    pub result_id: Option<String>,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub report_section: String,
    pub pipeline_ids: Vec<String>,
    pub expected_output_count: usize,
    pub expected_metric_count: usize,
    pub comparable_metric_count: usize,
    pub simulated_elapsed_seconds: Option<f64>,
    pub real_smoke_execution_id: Option<String>,
    pub real_smoke_elapsed_seconds: Option<f64>,
    pub declared_memory_mb: Option<f64>,
    pub declared_cpu_threads: Option<u32>,
    pub real_smoke_memory_mb: Option<f64>,
    pub real_smoke_cpu_threads: Option<u32>,
    pub evidence_path: String,
    pub manifest_path: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkStageSectionRow {
    pub domain: String,
    pub stage_id: String,
    pub row_count: usize,
    pub present_row_count: usize,
    pub missing_result_row_count: usize,
    pub unsupported_pair_row_count: usize,
    pub tool_ids: Vec<String>,
    pub corpus_ids: Vec<String>,
    pub pipeline_ids: Vec<String>,
    pub report_sections: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkToolSectionRow {
    pub tool_id: String,
    pub row_count: usize,
    pub present_row_count: usize,
    pub missing_result_row_count: usize,
    pub unsupported_pair_row_count: usize,
    pub domains: Vec<String>,
    pub stage_ids: Vec<String>,
    pub corpus_ids: Vec<String>,
    pub pipeline_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkCorpusSectionRow {
    pub corpus_id: String,
    pub row_count: usize,
    pub present_row_count: usize,
    pub missing_result_row_count: usize,
    pub unsupported_pair_row_count: usize,
    pub domains: Vec<String>,
    pub stage_ids: Vec<String>,
    pub tool_ids: Vec<String>,
    pub pipeline_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkPipelineSectionRow {
    pub pipeline_id: String,
    pub row_count: usize,
    pub present_row_count: usize,
    pub missing_result_row_count: usize,
    pub unsupported_pair_row_count: usize,
    pub domains: Vec<String>,
    pub stage_ids: Vec<String>,
    pub tool_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkPipelineSection {
    pub row_count: usize,
    pub unmapped_row_count: usize,
    pub rows: Vec<FullBenchmarkPipelineSectionRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkRuntimeRow {
    pub report_row_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub row_status: FullBenchmarkReportRowStatus,
    pub simulated_elapsed_seconds: Option<f64>,
    pub real_smoke_execution_id: Option<String>,
    pub real_smoke_elapsed_seconds: Option<f64>,
    pub runtime_source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkMemoryRow {
    pub report_row_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub row_status: FullBenchmarkReportRowStatus,
    pub declared_memory_mb: Option<f64>,
    pub declared_cpu_threads: Option<u32>,
    pub real_smoke_memory_mb: Option<f64>,
    pub real_smoke_cpu_threads: Option<u32>,
    pub memory_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkFailureRow {
    pub record_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub result_id: Option<String>,
    pub evidence_path: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkFailuresSection {
    pub simulated_failure_row_count: usize,
    pub failure_class_row_count: usize,
    pub simulated_failure_domain_counts: BTreeMap<String, usize>,
    pub rows: Vec<FullBenchmarkFailureRow>,
    pub classification_rows: Vec<AllDomainFailureClassificationRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkMissingResultRow {
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub report_section: String,
    pub audit_manifest_path: Option<String>,
    pub evidence_path: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkComparableMetricRow {
    pub domain: String,
    pub stage_id: String,
    pub metric_id: String,
    pub metric_name: String,
    pub unit: Option<String>,
    pub direction: String,
    pub tolerance_kind: String,
    pub tolerance_value: f64,
    pub pass_rule: String,
    pub insufficiency_behavior: String,
    pub required: bool,
    pub tool_ids: Vec<String>,
    pub contract_status: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkUnsupportedPairRow {
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub evidence_path: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub markdown_output_path: String,
    pub json_output_path: String,
    pub row_count: usize,
    pub expected_result_row_count: usize,
    pub explicit_unsupported_row_count: usize,
    pub present_row_count: usize,
    pub missing_result_row_count: usize,
    pub unsupported_pair_row_count: usize,
    pub stage_centric_row_count: usize,
    pub tool_centric_row_count: usize,
    pub corpus_centric_row_count: usize,
    pub pipeline_centric_row_count: usize,
    pub runtime_row_count: usize,
    pub memory_row_count: usize,
    pub comparable_metric_row_count: usize,
    pub failure_row_count: usize,
    pub failure_class_row_count: usize,
    pub passes_behavior_test: bool,
    pub domain_counts: BTreeMap<String, usize>,
    pub row_status_counts: BTreeMap<String, usize>,
    pub rows: Vec<FullBenchmarkReportRow>,
    pub stage_centric: Vec<FullBenchmarkStageSectionRow>,
    pub tool_centric: Vec<FullBenchmarkToolSectionRow>,
    pub corpus_centric: Vec<FullBenchmarkCorpusSectionRow>,
    pub pipeline_centric: FullBenchmarkPipelineSection,
    pub runtime: Vec<FullBenchmarkRuntimeRow>,
    pub memory: Vec<FullBenchmarkMemoryRow>,
    pub failures: FullBenchmarkFailuresSection,
    pub missing_results: Vec<FullBenchmarkMissingResultRow>,
    pub comparable_metrics: Vec<FullBenchmarkComparableMetricRow>,
    pub unsupported_pairs: Vec<FullBenchmarkUnsupportedPairRow>,
}

#[must_use]
pub fn full_benchmark_report_schema() -> Schema {
    schemars::schema_for!(FullBenchmarkReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": FULL_BENCHMARK_REPORT_SCHEMA_V1})
}
