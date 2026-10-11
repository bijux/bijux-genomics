//! Persisted public benchmark metrics and reports.
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfFilterSmokeMetrics {
    pub schema_version: &'static str,
    pub stage_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub input_variants: u64,
    pub pass_variants: u64,
    pub failed_variants: u64,
    pub filter_ids: Vec<String>,
    pub depth_threshold: f64,
    pub quality_threshold: f64,
    pub missingness_threshold: f64,
    pub sample_count: u64,
    pub tool_id: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfFilterSmokeReport {
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
    pub metrics_path: String,
    pub filter_breakdown_path: String,
    pub filter_breakdown_tsv_path: String,
    pub filter_explain_path: String,
    pub stage_result_manifest_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub input_variants: u64,
    pub pass_variants: u64,
    pub failed_variants: u64,
    pub filter_ids: Vec<String>,
    pub depth_threshold: f64,
    pub quality_threshold: f64,
    pub missingness_threshold: f64,
    pub sample_count: u64,
    pub parseable: bool,
    pub validation_checks: BTreeMap<String, bool>,
    pub gt_present: bool,
    pub gl_present: bool,
}
