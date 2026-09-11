//! Persisted public genotype benchmark schemas.
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalVcfCallDiploidSmokeMetrics {
    pub schema_version: &'static str,
    pub stage_id: String,
    pub corpus_id: String,
    pub sample_id: String,
    pub input_bam: String,
    pub reference: String,
    pub ploidy: &'static str,
    pub called_genotypes: u64,
    pub heterozygous_count: u64,
    pub homozygous_ref_count: u64,
    pub homozygous_alt_count: u64,
    pub missing_count: u64,
    pub sample_count: u64,
    pub tool_id: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
// Independent persisted scientific observations retain their existing wire fields.
#[allow(clippy::struct_excessive_bools)]
pub struct LocalVcfCallDiploidSmokeReport {
    pub schema_version: &'static str,
    pub command: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub sample_id: String,
    pub sample_name: String,
    pub input_bam: String,
    pub input_bam_index: String,
    pub reference: String,
    pub materialized_reference_path: String,
    pub output_root: String,
    pub output_vcf_path: String,
    pub output_tbi_path: String,
    pub metrics_path: String,
    pub stage_result_manifest_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub ploidy: &'static str,
    pub called_genotypes: u64,
    pub heterozygous_count: u64,
    pub homozygous_ref_count: u64,
    pub homozygous_alt_count: u64,
    pub missing_count: u64,
    pub sample_count: u64,
    pub parseable: bool,
    pub validation_checks: BTreeMap<String, bool>,
    pub diploid_compatible: bool,
    pub gt_present: bool,
    pub gl_present: bool,
}
