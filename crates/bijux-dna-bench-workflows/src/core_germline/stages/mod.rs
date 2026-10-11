//! Owner: bijux-dna-bench-workflows
//! Core germline stages responsibilities.
mod fastq_validate;
pub(crate) use fastq_validate::run_fastq_validate_stage;
mod fastq_profile;
pub(crate) use fastq_profile::run_fastq_profile_stage;
mod fastq_trim;
pub(crate) use fastq_trim::run_fastq_trim_stage;
mod fastq_filter;
pub(crate) use fastq_filter::run_fastq_filter_stage;
mod bam_align;
pub(crate) use bam_align::run_bam_align_stage;
mod bam_validate;
pub(crate) use bam_validate::run_bam_validate_stage;
mod bam_qc_pre;
pub(crate) use bam_qc_pre::run_bam_qc_pre_stage;
mod bam_coverage;
pub(crate) use bam_coverage::run_bam_coverage_stage;
mod vcf_call;
pub(crate) use vcf_call::run_vcf_call_stage;
mod vcf_filter;
pub(crate) use vcf_filter::run_vcf_filter_stage;
mod vcf_stats;
pub(crate) use vcf_stats::run_vcf_stats_stage;
mod vcf_qc;
pub(crate) use vcf_qc::run_vcf_qc_stage;
