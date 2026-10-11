//! Persisted public benchmark metrics and reports.
use super::observations::LocalVcfQcSmokeHeterozygosity;
use super::observations::LocalVcfQcSmokeHweSummary;
use super::observations::LocalVcfQcSmokeMafSummary;
use super::observations::LocalVcfQcSmokeSampleMissingnessRow;
use super::observations::LocalVcfQcSmokeVariantMissingnessRow;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfQcSmokeMetrics {
    pub schema_version: &'static str,
    pub stage_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub sample_missingness: Vec<LocalVcfQcSmokeSampleMissingnessRow>,
    pub variant_missingness: Vec<LocalVcfQcSmokeVariantMissingnessRow>,
    pub maf_summary: LocalVcfQcSmokeMafSummary,
    pub heterozygosity: LocalVcfQcSmokeHeterozygosity,
    pub hwe_summary: LocalVcfQcSmokeHweSummary,
    pub excluded_samples: Vec<LocalVcfQcSmokeSampleMissingnessRow>,
    pub excluded_variants: Vec<LocalVcfQcSmokeVariantMissingnessRow>,
    pub sample_missingness_exclusion_threshold: f64,
    pub variant_missingness_exclusion_threshold: f64,
    pub tool_id: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfQcSmokeReport {
    pub schema_version: &'static str,
    pub command: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub sample_name: String,
    pub input_vcf_path: String,
    pub output_root: String,
    pub qc_json_path: String,
    pub qc_summary_path: String,
    pub qc_tables_path: String,
    pub imputation_qc_path: String,
    pub warnings_path: String,
    pub qc_histograms_path: String,
    pub metrics_path: String,
    pub stage_result_manifest_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub sample_missingness: Vec<LocalVcfQcSmokeSampleMissingnessRow>,
    pub variant_missingness: Vec<LocalVcfQcSmokeVariantMissingnessRow>,
    pub maf_summary: LocalVcfQcSmokeMafSummary,
    pub heterozygosity: LocalVcfQcSmokeHeterozygosity,
    pub hwe_summary: LocalVcfQcSmokeHweSummary,
    pub excluded_samples: Vec<LocalVcfQcSmokeSampleMissingnessRow>,
    pub excluded_variants: Vec<LocalVcfQcSmokeVariantMissingnessRow>,
    pub sample_missingness_exclusion_threshold: f64,
    pub variant_missingness_exclusion_threshold: f64,
}
