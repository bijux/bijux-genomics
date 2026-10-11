//! Owner: bijux-dna-bench-workflows
//! adna stages responsibilities.
mod fastq_validate;
pub(crate) use fastq_validate::run_fastq_validate_stage;
mod fastq_trim_terminal_damage;
pub(crate) use fastq_trim_terminal_damage::run_fastq_trim_terminal_damage_stage;
mod fastq_remove_duplicates;
pub(crate) use fastq_remove_duplicates::run_fastq_remove_duplicates_stage;
mod bam_align;
pub(crate) use bam_align::run_bam_align_stage;
mod bam_validate;
pub(crate) use bam_validate::run_bam_validate_stage;
mod bam_mapping_summary;
pub(crate) use bam_mapping_summary::run_bam_mapping_summary_stage;
mod bam_coverage;
pub(crate) use bam_coverage::run_bam_coverage_stage;
mod bam_damage;
pub(crate) use bam_damage::run_bam_damage_stage;
mod bam_authenticity;
pub(crate) use bam_authenticity::run_bam_authenticity_stage;
mod vcf_call_pseudohaploid_stage;
pub(crate) use vcf_call_pseudohaploid_stage::run_vcf_call_pseudohaploid_stage_row;
mod vcf_damage_filter_stage;
pub(crate) use vcf_damage_filter_stage::run_vcf_damage_filter_stage_row;
mod vcf_stats_stage;
pub(crate) use vcf_stats_stage::run_vcf_stats_stage_row;
