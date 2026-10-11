use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfExpectedBenchmarkResultRow {
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub expected_outputs: Vec<String>,
    pub expected_metrics: Vec<String>,
    pub report_section: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfExpectedBenchmarkResultsReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub row_count: usize,
    pub stage_count: usize,
    pub tool_count: usize,
    pub corpus_count: usize,
    pub asset_profile_count: usize,
    pub report_section_counts: BTreeMap<String, usize>,
    pub rows: Vec<VcfExpectedBenchmarkResultRow>,
}

pub const DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-expected-benchmark-results.tsv";
