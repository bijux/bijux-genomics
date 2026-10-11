//! Persisted public benchmark metrics and reports.
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
// These independent observations preserve the persisted scientific report schema.
#[allow(clippy::struct_excessive_bools)]
pub struct LocalVcfPostprocessSmokeMetrics {
    pub schema_version: &'static str,
    pub stage_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub readable_vcf: bool,
    pub tabix_present: bool,
    pub contigs_consistent_with_species_context: bool,
    pub left_align_applied: bool,
    pub multiallelic_records_split: u64,
    pub indels_normalized: u64,
    pub variant_ids_normalized: u64,
    pub invalid_records_removed: u64,
    pub filter_standardized_to_pass: u64,
    pub record_count: u64,
    pub sample_count: u64,
    pub sample_ids: Vec<String>,
    pub mq_removed: bool,
    pub multiallelic_records_remaining: u64,
    pub tool_id: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
// These independent observations preserve the persisted scientific report schema.
#[allow(clippy::struct_excessive_bools)]
pub struct LocalVcfPostprocessSmokeReport {
    pub schema_version: &'static str,
    pub command: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub input_vcf_path: String,
    pub output_root: String,
    pub output_vcf_path: String,
    pub output_tbi_path: String,
    pub validate_outputs_path: String,
    pub final_manifest_path: String,
    pub normalization_contract_path: String,
    pub artifact_checksums_path: String,
    pub logs_path: String,
    pub metrics_path: String,
    pub stage_result_manifest_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub readable_vcf: bool,
    pub tabix_present: bool,
    pub contigs_consistent_with_species_context: bool,
    pub left_align_applied: bool,
    pub multiallelic_records_split: u64,
    pub indels_normalized: u64,
    pub variant_ids_normalized: u64,
    pub invalid_records_removed: u64,
    pub filter_standardized_to_pass: u64,
    pub record_count: u64,
    pub sample_count: u64,
    pub sample_ids: Vec<String>,
    pub mq_removed: bool,
    pub multiallelic_records_remaining: u64,
    pub parseable: bool,
    pub validation_checks: BTreeMap<String, bool>,
    pub gt_present: bool,
}
