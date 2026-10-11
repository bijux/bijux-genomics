//! Persisted public genotype benchmark schemas.
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalVcfGlPropagationSmokeMetrics {
    pub schema_version: &'static str,
    pub stage_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub input_likelihood_fields: Vec<String>,
    pub output_likelihood_fields: Vec<String>,
    pub lost_fields: Vec<String>,
    pub site_count_before: u64,
    pub site_count_after: u64,
    pub sample_count: u64,
    pub tool_id: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfGlPropagationSmokeReport {
    pub schema_version: &'static str,
    pub command: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub sample_name: String,
    pub input_vcf_path: String,
    pub output_root: String,
    pub output_vcf_path: String,
    pub output_tbi_path: String,
    pub output_bcf_path: String,
    pub output_bcf_csi_path: String,
    pub report_path: String,
    pub metrics_path: String,
    pub stage_result_manifest_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub input_likelihood_fields: Vec<String>,
    pub output_likelihood_fields: Vec<String>,
    pub lost_fields: Vec<String>,
    pub site_count_before: u64,
    pub site_count_after: u64,
    pub sample_count: u64,
    pub parseable: bool,
    pub validation_checks: BTreeMap<String, bool>,
    pub gt_present: bool,
    pub gl_present: bool,
}
