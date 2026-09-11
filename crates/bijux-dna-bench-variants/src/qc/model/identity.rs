//! Admitted benchmark identity and governed artifact declarations.

pub(crate) const DEFAULT_VCF_QC_SMOKE_ROOT: &str = "artifacts/runs/bench/local-smoke/vcf.qc";

pub(crate) const LOCAL_VCF_QC_SMOKE_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_qc_smoke.v1";

pub(crate) const LOCAL_VCF_QC_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_qc_smoke.metrics.v1";

pub(crate) const LOCAL_VCF_QC_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-qc-smoke";

pub(crate) const GOVERNED_VCF_QC_CORPUS_ID: &str = "vcf_production_regression";

pub(crate) const GOVERNED_VCF_QC_ASSET_PROFILE_ID: &str = "vcf_cohort";

pub(crate) const GOVERNED_VCF_QC_INPUT_FIXTURE_ID: &str = "qc_cohort_missingness";

pub(crate) const GOVERNED_VCF_QC_SAMPLE_NAME: &str = "qc_cohort";

pub(crate) const DEFAULT_INPUT_VCF_NAME: &str = "qc_input.vcf";

pub(crate) const DEFAULT_OUTPUT_QC_NAME: &str = "qc.json";

pub(crate) const DEFAULT_OUTPUT_SUMMARY_NAME: &str = "qc_summary.json";

pub(crate) const DEFAULT_OUTPUT_QC_TABLES_NAME: &str = "qc_tables.tsv";

pub(crate) const DEFAULT_OUTPUT_IMPUTATION_QC_NAME: &str = "imputation_qc.tsv";

pub(crate) const DEFAULT_OUTPUT_WARNINGS_NAME: &str = "warnings.json";

pub(crate) const DEFAULT_OUTPUT_HISTOGRAMS_NAME: &str = "qc_histograms.json";

pub(crate) const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";

pub(crate) const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

pub(crate) const EXPECTED_EXCLUDED_SAMPLE_ID: &str = "qc_sparse";

pub(crate) const EXPECTED_EXCLUDED_VARIANT_ID: &str = "chr1:30:G:A";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfQcSmokeContract {
    pub(crate) report_format: crate::qc::admission::report_format::QcReportFormat,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) sample_name: String,
}
