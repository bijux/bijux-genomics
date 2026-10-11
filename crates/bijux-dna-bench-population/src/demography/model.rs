//! Owner: bijux-dna-bench-population
//! Persisted population report contracts and admitted stage identities.
use serde::Serialize;
pub(crate) const DEFAULT_VCF_DEMOGRAPHY_SMOKE_ROOT: &str =
    "artifacts/runs/bench/local-smoke/vcf.demography";
pub(crate) const LOCAL_VCF_DEMOGRAPHY_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_demography_smoke.v1";
pub(crate) const LOCAL_VCF_DEMOGRAPHY_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-demography-smoke";
pub(crate) const GOVERNED_VCF_DEMOGRAPHY_STAGE_ID: &str =
    bijux_dna_core::id_catalog::VCF_DEMOGRAPHY;
pub(crate) const GOVERNED_VCF_DEMOGRAPHY_CORPUS_ID: &str = "vcf_production_regression";
pub(crate) const GOVERNED_VCF_DEMOGRAPHY_ASSET_PROFILE_ID: &str = "json_ibd_segments";
pub(crate) const GOVERNED_VCF_DEMOGRAPHY_INPUT_FIXTURE_ID: &str = "vcf_mini_multisample_cohort";
pub(crate) const GOVERNED_VCF_DEMOGRAPHY_EXPECTED_OUTPUT_ID: &str = "demography_report";
pub(crate) const GOVERNED_VCF_DEMOGRAPHY_UPSTREAM_TOOL_ID: &str = "germline";
pub(crate) const GOVERNED_VCF_DEMOGRAPHY_MIN_SEGMENTS: usize = 1;
pub(crate) const DEFAULT_INPUT_IBD_NAME: &str = "input_ibd_segments.tsv";
pub(crate) const DEFAULT_OUTPUT_JSON_NAME: &str = "demography.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_UPSTREAM_IBD_REPORT_NAME: &str = "source_ibd_smoke.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_UPSTREAM_FILTERED_SEGMENTS_NAME: &str =
    "source_ibd_filtered_segments.tsv";
pub(crate) const DEFAULT_OUTPUT_SOURCE_NE_TRAJECTORY_NAME: &str = "source_ne_trajectory.tsv";
pub(crate) const DEFAULT_OUTPUT_SOURCE_DEMOGRAPHY_CONTRACT_NAME: &str =
    "source_demography_contract.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_DEMOGRAPHY_METRICS_NAME: &str =
    "source_demography_metrics.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_LOGS_NAME: &str = "source_logs.txt";
pub(crate) const DEFAULT_PROBE_INPUT_IBD_NAME: &str = "probe_input_ibd_segments.tsv";
pub(crate) const DEFAULT_PROBE_SOURCE_NE_TRAJECTORY_NAME: &str = "probe_source_ne_trajectory.tsv";
pub(crate) const DEFAULT_PROBE_SOURCE_DEMOGRAPHY_CONTRACT_NAME: &str =
    "probe_source_demography_contract.json";
pub(crate) const DEFAULT_PROBE_SOURCE_DEMOGRAPHY_METRICS_NAME: &str =
    "probe_source_demography_metrics.json";
pub(crate) const DEFAULT_PROBE_SOURCE_LOGS_NAME: &str = "probe_source_logs.txt";
pub(crate) const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfDemographySmokeContract {
    pub(crate) stage: String,
    pub(crate) tool: String,
    pub(crate) corpus: String,
    pub(crate) input_fixture: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfDemographyEstimate {
    pub time_bin: u64,
    pub ne: f64,
    pub ci_low: f64,
    pub ci_high: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfDemographyInsufficientProbe {
    pub input_ibd: String,
    pub method: String,
    pub status: String,
    pub insufficient_reason: String,
    pub time_bins: Vec<u64>,
    pub ne_estimates: Vec<LocalVcfDemographyEstimate>,
    pub source_ne_trajectory_path: String,
    pub source_demography_contract_path: String,
    pub source_demography_metrics_path: String,
    pub source_logs_path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfDemographySmokeReport {
    pub schema_version: &'static str,
    pub command: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub output_root: String,
    pub demography_json_path: String,
    pub input_ibd: String,
    pub source_upstream_ibd_report_path: String,
    pub source_upstream_filtered_segments_path: String,
    pub source_ne_trajectory_path: String,
    pub source_demography_contract_path: String,
    pub source_demography_metrics_path: String,
    pub source_logs_path: String,
    pub stage_result_manifest_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub method: String,
    pub inference_status: String,
    pub tool_ok: bool,
    pub time_bins: Vec<u64>,
    pub ne_estimates: Vec<LocalVcfDemographyEstimate>,
    pub status: String,
    pub insufficient_reason: Option<String>,
    pub insufficient_data_probe: LocalVcfDemographyInsufficientProbe,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NormalizedDemographyEvidence {
    pub(crate) method: String,
    pub(crate) inference_status: String,
    pub(crate) status: String,
    pub(crate) insufficient_reason: Option<String>,
    pub(crate) tool_ok: bool,
    pub(crate) time_bins: Vec<u64>,
    pub(crate) ne_estimates: Vec<LocalVcfDemographyEstimate>,
}
