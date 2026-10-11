//! Persisted imputation metrics contracts and declared identities.
use bijux_dna_bench_model::BenchmarkImputationQualityFieldAvailability as QualityFieldAvailability;

pub(crate) const DEFAULT_VCF_IMPUTATION_METRICS_SMOKE_ROOT: &str =
    "artifacts/runs/bench/local-smoke/vcf.imputation_metrics";
pub(crate) const LOCAL_VCF_IMPUTATION_METRICS_COMMAND: &str =
    "bijux-dna bench local run-vcf-imputation-metrics-smoke";
pub(crate) const LOCAL_VCF_IMPUTATION_METRICS_STAGE_ID: &str = "vcf.imputation_metrics";
pub(crate) const DEFAULT_OUTPUT_REPORT_NAME: &str = "imputation_metrics.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_VCF_NAME: &str = "source_imputed.vcf.gz";
pub(crate) const DEFAULT_OUTPUT_SOURCE_TBI_NAME: &str = "source_imputed.vcf.gz.tbi";
pub(crate) const DEFAULT_OUTPUT_SOURCE_ACCEPT_NAME: &str = "source_imputation_accept.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_QC_NAME: &str = "source_imputation_qc.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_SMOKE_NAME: &str = "source_impute_smoke_metrics.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME: &str = "source_imputation_manifest.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_LOGS_NAME: &str = "source_logs.txt";
pub(crate) const DEFAULT_OUTPUT_ORCHESTRATION_MANIFEST_NAME: &str = "orchestration_manifest.json";
pub(crate) const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ImputationMetricsSummary {
    pub(crate) concordance: Option<f64>,
    pub(crate) mean_info_score: f64,
    pub(crate) r2_available: bool,
    pub(crate) dosage_r2: Option<f64>,
    pub(crate) low_confidence_sites: u64,
    pub(crate) masked_truth_sites: u64,
    pub(crate) quality_field_availability: QualityFieldAvailability,
    pub(crate) missing_quality_fields: Vec<String>,
    pub(crate) status: String,
}
