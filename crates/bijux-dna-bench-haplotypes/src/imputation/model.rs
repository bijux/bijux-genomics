//! Persisted imputation contracts and declared identities.
use serde::Serialize;
use std::collections::BTreeMap;

pub(crate) const DEFAULT_VCF_IMPUTE_SMOKE_ROOT: &str =
    "artifacts/runs/bench/local-smoke/vcf.impute";
pub(crate) const LOCAL_VCF_IMPUTE_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_impute_smoke.v1";
pub(crate) const LOCAL_VCF_IMPUTE_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_impute_smoke.metrics.v1";
pub(crate) const LOCAL_VCF_IMPUTE_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-impute-smoke";
pub(crate) const GOVERNED_VCF_IMPUTE_STAGE_ID: &str = "vcf.impute";
pub(crate) const GOVERNED_VCF_IMPUTE_INPUT_FIXTURE_ID: &str = "masked_truth_two_sample";
pub(crate) const DEFAULT_INPUT_VCF_NAME: &str = "impute_input.vcf";
pub(crate) const DEFAULT_TRUTH_VCF_NAME: &str = "impute_truth.vcf";
pub(crate) const DEFAULT_OUTPUT_VCF_NAME: &str = "imputed.vcf.gz";
pub(crate) const DEFAULT_OUTPUT_PANEL_ASSETS_NAME: &str = "panel_assets.json";
pub(crate) const DEFAULT_OUTPUT_QC_NAME: &str = "imputation_qc.json";
pub(crate) const DEFAULT_OUTPUT_QC_TSV_NAME: &str = "imputation_qc.tsv";
pub(crate) const DEFAULT_OUTPUT_MANIFEST_NAME: &str = "imputation_manifest.json";
pub(crate) const DEFAULT_OUTPUT_OVERLAP_NAME: &str = "overlap_stats.json";
pub(crate) const DEFAULT_OUTPUT_WARNINGS_NAME: &str = "warnings.json";
pub(crate) const DEFAULT_OUTPUT_ACCEPT_NAME: &str = "imputation_accept.json";
pub(crate) const DEFAULT_OUTPUT_PANEL_MISMATCH_NAME: &str = "panel_mismatch_diagnostics.json";
pub(crate) const DEFAULT_OUTPUT_MAF_BINS_NAME: &str = "maf_bins.tsv";
pub(crate) const DEFAULT_OUTPUT_INFO_HIST_NAME: &str = "info_hist.json";
pub(crate) const DEFAULT_OUTPUT_LOGS_NAME: &str = "logs.txt";
pub(crate) const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";
pub(crate) const EXPECTED_SAMPLE_IDS: [&str; 2] = ["masked_sample", "donor_sample"];
pub(crate) const EXPECTED_VARIANT_COUNT: u64 = 2;
pub(crate) const EXPECTED_MASKED_SAMPLE_GT: &str = "0/1";
pub(crate) const EXPECTED_DONOR_SAMPLE_GT: &str = "0/1";
pub(crate) const EXPECTED_MISSING_BEFORE: u64 = 1;
pub(crate) const EXPECTED_MISSING_AFTER: u64 = 0;
pub(crate) const EXPECTED_IMPUTED_GENOTYPES: u64 = 1;
pub(crate) const EXPECTED_LOW_CONFIDENCE_COUNT: u64 = 1;
pub(crate) const EXPECTED_MASKED_TRUTH_SITE_COUNT: u64 = 1;
pub(crate) const EXPECTED_MASKED_TRUTH_MATCH_COUNT: u64 = 1;
pub(crate) const EXPECTED_UNRESOLVED_COUNT: u64 = 0;
pub(crate) const GOVERNED_IMPUTE_SEED: u64 = 42;
pub(crate) const GOVERNED_IMPUTE_THREADS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImputeOutputSummary {
    pub(crate) variant_count: u64,
    pub(crate) sample_ids: Vec<String>,
    pub(crate) masked_sample_gt: String,
    pub(crate) donor_sample_gt: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfImputeSmokeMetrics {
    pub schema_version: &'static str,
    pub stage_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub panel_id: String,
    pub map_id: String,
    pub variant_count: u64,
    pub missing_before: u64,
    pub missing_after: u64,
    pub imputed_genotypes: u64,
    pub low_confidence_count: u64,
    pub masked_truth_site_count: u64,
    pub masked_truth_match_count: u64,
    pub unresolved_count: u64,
    pub not_imputable_reasons: BTreeMap<String, u64>,
    pub sample_count: u64,
    pub sample_ids: Vec<String>,
    pub tool_id: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfImputeSmokeReport {
    pub schema_version: &'static str,
    pub command: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub panel_id: String,
    pub map_id: String,
    pub input_vcf_path: String,
    pub truth_vcf_path: String,
    pub output_root: String,
    pub output_vcf_path: String,
    pub output_tbi_path: String,
    pub panel_assets_path: String,
    pub imputation_qc_path: String,
    pub imputation_qc_tsv_path: String,
    pub imputation_manifest_path: String,
    pub overlap_stats_path: String,
    pub warnings_path: String,
    pub imputation_accept_path: String,
    pub panel_mismatch_path: String,
    pub maf_bins_path: String,
    pub info_hist_path: String,
    pub logs_path: String,
    pub metrics_path: String,
    pub stage_result_manifest_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub variant_count: u64,
    pub missing_before: u64,
    pub missing_after: u64,
    pub imputed_genotypes: u64,
    pub low_confidence_count: u64,
    pub masked_truth_site_count: u64,
    pub masked_truth_match_count: u64,
    pub unresolved_count: u64,
    pub not_imputable_reasons: BTreeMap<String, u64>,
    pub sample_count: u64,
    pub sample_ids: Vec<String>,
    pub masked_sample_gt: String,
    pub donor_sample_gt: String,
    pub parseable: bool,
    pub validation_checks: BTreeMap<String, bool>,
    pub gt_present: bool,
    pub gl_present: bool,
}
