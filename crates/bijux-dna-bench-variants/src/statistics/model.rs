use serde::Serialize;
pub(super) const DEFAULT_VCF_STATS_SMOKE_ROOT: &str = "artifacts/runs/bench/local-smoke/vcf.stats";
pub(super) const LOCAL_VCF_STATS_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_stats_smoke.v1";
pub(super) const LOCAL_VCF_STATS_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_stats_smoke.metrics.v1";
pub(super) const LOCAL_VCF_STATS_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-stats-smoke";
pub(super) const GOVERNED_VCF_STATS_STAGE_ID: &str = "vcf.stats";
pub(super) const GOVERNED_VCF_STATS_TOOL_ID: &str = "bcftools";
pub(super) const GOVERNED_VCF_STATS_CORPUS_ID: &str = "vcf_production_regression";
pub(super) const GOVERNED_VCF_STATS_ASSET_PROFILE_ID: &str = "vcf_cohort";
pub(super) const GOVERNED_VCF_STATS_INPUT_FIXTURE_ID: &str = "stats_cohort_minimal";
pub(super) const GOVERNED_VCF_STATS_SAMPLE_NAME: &str = "cohort_stats";
pub(super) const DEFAULT_INPUT_VCF_NAME: &str = "stats_input.vcf";
pub(super) const DEFAULT_OUTPUT_STATS_NAME: &str = "stats.json";
pub(super) const DEFAULT_OUTPUT_BCFTOOLS_STATS_NAME: &str = "bcftools_stats.txt";
pub(super) const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";
pub(super) const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GovernedVcfStatsSmokeContract {
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) corpus_id: String,
    pub(super) input_fixture_id: String,
    pub(super) sample_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct VcfStatsTruthCounts {
    pub(super) variants: u64,
    pub(super) snps: u64,
    pub(super) indels: u64,
    pub(super) transitions: u64,
    pub(super) transversions: u64,
    pub(super) samples: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfStatsSmokeMetrics {
    pub schema_version: &'static str,
    pub stage_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub variant_count: u64,
    pub snp_count: u64,
    pub indel_count: u64,
    pub transition_count: u64,
    pub transversion_count: u64,
    pub ti_tv: f64,
    pub sample_count: u64,
    pub tool_id: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfStatsSmokeReport {
    pub schema_version: &'static str,
    pub command: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub sample_name: String,
    pub input_vcf_path: String,
    pub output_root: String,
    pub stats_json_path: String,
    pub bcftools_stats_path: String,
    pub metrics_path: String,
    pub stage_result_manifest_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub variant_count: u64,
    pub snp_count: u64,
    pub indel_count: u64,
    pub transition_count: u64,
    pub transversion_count: u64,
    pub ti_tv: f64,
    pub sample_count: u64,
}
