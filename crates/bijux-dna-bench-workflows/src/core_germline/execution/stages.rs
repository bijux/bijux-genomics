//! Owner: bijux-dna-bench-workflows
//! Ordered stage execution with explicitly admitted fixture operations.
use super::super::{
    run_bam_align_stage, run_bam_coverage_stage, run_bam_qc_pre_stage, run_bam_validate_stage,
    run_fastq_filter_stage, run_fastq_profile_stage, run_fastq_trim_stage,
    run_fastq_validate_stage, run_vcf_call_stage, run_vcf_filter_stage, run_vcf_qc_stage,
    run_vcf_stats_stage, CoreGermlineMicroPipelineRow, CoreGermlineWorkflowInputs, Path,
    PipelineInputFixtures, Result,
};

pub(crate) fn execute_stages(
    inputs: &CoreGermlineWorkflowInputs,
    repo_root: &Path,
    output_root: &Path,
    reference_fasta: &Path,
    input_fixtures: &PipelineInputFixtures,
) -> Result<Vec<CoreGermlineMicroPipelineRow>> {
    let validate_row = run_fastq_validate_stage(
        repo_root,
        output_root,
        &input_fixtures.raw_r1,
        &input_fixtures.raw_r2,
    )?;
    let profile_row = run_fastq_profile_stage(
        repo_root,
        output_root,
        &input_fixtures.raw_r1,
        &input_fixtures.raw_r2,
        &validate_row,
    )?;
    let trim_row = run_fastq_trim_stage(
        repo_root,
        output_root,
        &input_fixtures.raw_r1,
        &input_fixtures.raw_r2,
        &validate_row,
    )?;
    let filter_row = run_fastq_filter_stage(repo_root, output_root, &trim_row)?;
    let align_row = run_bam_align_stage(
        inputs.alignment(),
        repo_root,
        output_root,
        reference_fasta,
        input_fixtures,
        &filter_row,
    )?;
    let validate_bam_row =
        run_bam_validate_stage(repo_root, output_root, reference_fasta, &align_row)?;
    let qc_pre_row = run_bam_qc_pre_stage(repo_root, output_root, &align_row, &validate_bam_row)?;
    let coverage_row = run_bam_coverage_stage(
        inputs.alignment(),
        repo_root,
        output_root,
        &align_row,
        &qc_pre_row,
    )?;
    let vcf_tool_ids = inputs.vcf_tools();
    let call_row = run_vcf_call_stage(
        repo_root,
        output_root,
        &align_row,
        &coverage_row,
        reference_fasta,
        vcf_tool_ids,
    )?;
    let filter_vcf_row = run_vcf_filter_stage(repo_root, output_root, &call_row, vcf_tool_ids)?;
    let stats_row = run_vcf_stats_stage(repo_root, output_root, &filter_vcf_row, vcf_tool_ids)?;
    let qc_row =
        run_vcf_qc_stage(repo_root, output_root, &filter_vcf_row, &stats_row, vcf_tool_ids)?;

    Ok(vec![
        validate_row,
        profile_row,
        trim_row,
        filter_row,
        align_row,
        validate_bam_row,
        qc_pre_row,
        coverage_row,
        call_row,
        filter_vcf_row,
        stats_row,
        qc_row,
    ])
}
