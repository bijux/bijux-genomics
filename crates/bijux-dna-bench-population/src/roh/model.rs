//! Owner: bijux-dna-bench-population
//! Persisted population report contracts and admitted stage identities.
use serde::Serialize;
pub(crate) const DEFAULT_VCF_ROH_SMOKE_ROOT: &str = "artifacts/runs/bench/local-smoke/vcf.roh";
pub(crate) const LOCAL_VCF_ROH_SMOKE_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_roh_smoke.v1";
pub(crate) const LOCAL_VCF_ROH_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-roh-smoke";
pub(crate) const GOVERNED_VCF_ROH_STAGE_ID: &str = bijux_dna_core::id_catalog::VCF_ROH;
pub(crate) const GOVERNED_VCF_ROH_CORPUS_ID: &str = "vcf_production_regression";
pub(crate) const GOVERNED_VCF_ROH_ASSET_PROFILE_ID: &str = "vcf_cohort";
pub(crate) const GOVERNED_VCF_ROH_INPUT_FIXTURE_ID: &str = "vcf_mini_multisample_cohort";
pub(crate) const DEFAULT_INPUT_VCF_NAME: &str = "roh_input.vcf";
pub(crate) const DEFAULT_INPUT_SAMPLE_METADATA_NAME: &str = "sample_metadata.tsv";
pub(crate) const DEFAULT_OUTPUT_TSV_NAME: &str = "roh.tsv";
pub(crate) const DEFAULT_OUTPUT_JSON_NAME: &str = "roh.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_SEGMENTS_NAME: &str = "source_roh_segments.tsv";
pub(crate) const DEFAULT_OUTPUT_SOURCE_PER_SAMPLE_NAME: &str = "source_roh_per_sample.tsv";
pub(crate) const DEFAULT_OUTPUT_SOURCE_ROH_REPORT_NAME: &str = "source_roh.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_METRICS_NAME: &str = "source_metrics.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_SUMMARY_NAME: &str = "source_roh_summary.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_ROH_METRICS_NAME: &str = "source_roh_metrics.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_LOGS_NAME: &str = "source_logs.txt";
pub(crate) const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfRohSmokeContract {
    pub(crate) stage: String,
    pub(crate) tool: String,
    pub(crate) corpus: String,
    pub(crate) input_fixture: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfRohSmokeSegment {
    pub sample_id: String,
    pub contig: String,
    pub start: u64,
    pub end: u64,
    pub length: u64,
    pub variant_count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfRohSmokePerSampleSummary {
    pub sample_id: String,
    pub segment_count: u64,
    pub total_length: u64,
    pub mean_length: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfRohSmokeReport {
    pub schema_version: &'static str,
    pub command: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub fixture_manifest_path: String,
    pub input_vcf_path: String,
    pub sample_metadata_path: String,
    pub output_root: String,
    pub roh_tsv_path: String,
    pub roh_json_path: String,
    pub source_roh_segments_path: String,
    pub source_roh_per_sample_path: String,
    pub source_roh_report_path: String,
    pub source_metrics_path: String,
    pub source_roh_summary_path: String,
    pub source_roh_metrics_path: String,
    pub source_logs_path: String,
    pub stage_result_manifest_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub execution_mode: String,
    pub tool_ok: bool,
    pub sample_count: u64,
    pub segment_count: u64,
    pub total_length: u64,
    pub segments: Vec<LocalVcfRohSmokeSegment>,
    pub per_sample_summary: Vec<LocalVcfRohSmokePerSampleSummary>,
    pub status: String,
}
