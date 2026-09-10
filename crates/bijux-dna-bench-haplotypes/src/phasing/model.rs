//! Persisted phasing contracts and declared identities.
use serde::Serialize;
use std::collections::BTreeMap;

pub(crate) const DEFAULT_VCF_PHASING_SMOKE_ROOT: &str =
    "artifacts/runs/bench/local-smoke/vcf.phasing";
pub(crate) const LOCAL_VCF_PHASING_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_phasing_smoke.v1";
pub(crate) const LOCAL_VCF_PHASING_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_phasing_smoke.metrics.v1";
pub(crate) const LOCAL_VCF_PHASING_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-phasing-smoke";
pub(crate) const GOVERNED_VCF_PHASING_STAGE_ID: &str = "vcf.phasing";
pub(crate) const GOVERNED_VCF_PHASING_INPUT_FIXTURE_ID: &str = "cohort_unphased_two_sample";
pub(crate) const DEFAULT_INPUT_VCF_NAME: &str = "phasing_input.vcf";
pub(crate) const DEFAULT_OUTPUT_VCF_NAME: &str = "phased.vcf.gz";
pub(crate) const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";
pub(crate) const DEFAULT_OUTPUT_PANEL_ASSETS_NAME: &str = "panel_assets.json";
pub(crate) const DEFAULT_OUTPUT_QC_NAME: &str = "phasing_qc.json";
pub(crate) const DEFAULT_OUTPUT_MANIFEST_NAME: &str = "phasing_manifest.json";
pub(crate) const DEFAULT_OUTPUT_PHASE_BLOCK_STATS_NAME: &str = "phase_block_stats.tsv";
pub(crate) const DEFAULT_OUTPUT_SWITCH_PROXY_NAME: &str = "switch_error_proxy.tsv";
pub(crate) const DEFAULT_OUTPUT_LOGS_NAME: &str = "logs.txt";
pub(crate) const EXPECTED_SAMPLE_IDS: [&str; 2] = ["cohort_alpha", "cohort_beta"];
pub(crate) const EXPECTED_INPUT_GENOTYPES: u64 = 8;
pub(crate) const EXPECTED_PHASED_GENOTYPES: u64 = 8;
pub(crate) const EXPECTED_UNPHASED_GENOTYPES: u64 = 0;
pub(crate) const EXPECTED_PHASE_SET_COUNT: u64 = 2;
pub(crate) const GOVERNED_PHASING_REGION: &str = "1:1-1000000";
pub(crate) const GOVERNED_PHASING_SEED: u64 = 37;
pub(crate) const GOVERNED_PHASING_THREADS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalVcfPhasingSmokeMetrics {
    pub schema_version: &'static str,
    pub stage_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub panel_id: String,
    pub map_id: String,
    pub input_genotypes: u64,
    pub phased_genotypes: u64,
    pub unphased_genotypes: u64,
    pub phase_set_count: u64,
    pub sample_count: u64,
    pub sample_ids: Vec<String>,
    pub tool_id: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfPhasingSmokeReport {
    pub schema_version: &'static str,
    pub command: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub panel_id: String,
    pub map_id: String,
    pub input_vcf_path: String,
    pub output_root: String,
    pub output_vcf_path: String,
    pub output_tbi_path: String,
    pub panel_assets_path: String,
    pub phasing_qc_path: String,
    pub phasing_manifest_path: String,
    pub phase_block_stats_path: String,
    pub switch_error_proxy_path: String,
    pub logs_path: String,
    pub metrics_path: String,
    pub stage_result_manifest_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub input_genotypes: u64,
    pub phased_genotypes: u64,
    pub unphased_genotypes: u64,
    pub phase_set_count: u64,
    pub sample_count: u64,
    pub sample_ids: Vec<String>,
    pub parseable: bool,
    pub validation_checks: BTreeMap<String, bool>,
    pub gt_present: bool,
    pub gl_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PhaseSummary {
    pub(crate) genotype_count: u64,
    pub(crate) phased_genotypes: u64,
    pub(crate) unphased_genotypes: u64,
    pub(crate) phase_set_count: u64,
    pub(crate) sample_ids: Vec<String>,
}
