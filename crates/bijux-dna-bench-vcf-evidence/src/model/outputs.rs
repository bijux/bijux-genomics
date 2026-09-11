use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VcfAdapterOutputCoverageStatus {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfAdapterOutputCoverageRow {
    pub stage_id: String,
    pub tool_id: String,
    pub raw_outputs: Vec<String>,
    pub normalized_metrics: Vec<String>,
    pub stdout: String,
    pub stderr: String,
    pub manifest: String,
    pub index_outputs: Vec<String>,
    pub status: VcfAdapterOutputCoverageStatus,
    pub benchmark_status: String,
    pub missing_declarations: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfAdapterOutputCoverageReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub row_count: usize,
    pub benchmark_ready_row_count: usize,
    pub benchmark_ready_complete_row_count: usize,
    pub benchmark_ready_incomplete_row_count: usize,
    pub complete_row_count: usize,
    pub incomplete_row_count: usize,
    pub rows: Vec<VcfAdapterOutputCoverageRow>,
}

pub const DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-adapter-output-coverage.tsv";
