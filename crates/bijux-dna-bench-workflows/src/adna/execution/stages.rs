//! Owner: bijux-dna-bench-workflows
//! Ordered stage execution with explicitly admitted fixture operations.
use super::super::{
    run_bam_align_stage, run_bam_authenticity_stage, run_bam_coverage_stage, run_bam_damage_stage,
    run_bam_mapping_summary_stage, run_bam_validate_stage, run_fastq_remove_duplicates_stage,
    run_fastq_trim_terminal_damage_stage, run_fastq_validate_stage,
    run_vcf_call_pseudohaploid_stage_row, run_vcf_damage_filter_stage_row, run_vcf_stats_stage_row,
    skipped_row, AdnaMicroPipelineRow, AlignmentFixtureOperations, Path, PipelineInputFixtures,
    Result, VCF_CALL_GL_TOOL_ID, VCF_GL_PROPAGATION_TOOL_ID,
};
use bijux_dna_core::id_catalog as stage;

pub(crate) fn execute_stages(
    inputs: &AlignmentFixtureOperations,
    repo_root: &Path,
    output_root: &Path,
    reference_fasta: &Path,
    input_fixtures: &PipelineInputFixtures,
) -> Result<Vec<AdnaMicroPipelineRow>> {
    let validate_row = run_fastq_validate_stage(
        repo_root,
        output_root,
        &input_fixtures.raw_r1,
        &input_fixtures.raw_r2,
    )?;
    let trim_row = run_fastq_trim_terminal_damage_stage(
        repo_root,
        output_root,
        &input_fixtures.raw_r1,
        &input_fixtures.raw_r2,
        &validate_row,
    )?;
    let dedup_row = run_fastq_remove_duplicates_stage(repo_root, output_root, &trim_row)?;
    let align_row = run_bam_align_stage(
        inputs,
        repo_root,
        output_root,
        reference_fasta,
        input_fixtures,
        &dedup_row,
    )?;
    let validate_bam_row =
        run_bam_validate_stage(repo_root, output_root, reference_fasta, &align_row)?;
    let mapping_row =
        run_bam_mapping_summary_stage(repo_root, output_root, &align_row, &validate_bam_row)?;
    let coverage_row =
        run_bam_coverage_stage(inputs, repo_root, output_root, &align_row, &mapping_row)?;
    let damage_row = run_bam_damage_stage(
        repo_root,
        output_root,
        reference_fasta,
        input_fixtures,
        &trim_row,
        &align_row,
    )?;
    let authenticity_row =
        run_bam_authenticity_stage(repo_root, output_root, &align_row, &damage_row)?;
    let contamination_row = skipped_row(
        stage::BAM_CONTAMINATION,
        "bam",
        "verifybamid2",
        "synthetic aDNA micro execution does not claim panel-backed contamination evidence; authenticity stays damage-driven in this run",
    );
    let pseudohaploid_row = run_vcf_call_pseudohaploid_stage_row(
        repo_root,
        output_root,
        &align_row,
        &coverage_row,
        &damage_row,
        &authenticity_row,
        reference_fasta,
    )?;
    let gl_call_row = skipped_row(
        stage::VCF_CALL_GL,
        "vcf",
        VCF_CALL_GL_TOOL_ID,
        "aDNA micro execution chooses the governed pseudohaploid branch; likelihood-bearing calling remains covered by dedicated GL smoke proof",
    );
    let gl_propagation_row = skipped_row(
        stage::VCF_GL_PROPAGATION,
        "vcf",
        VCF_GL_PROPAGATION_TOOL_ID,
        "aDNA micro execution does not produce a GL-bearing VCF because the pseudohaploid branch is the active governed path for this summary",
    );
    let damage_filter_row =
        run_vcf_damage_filter_stage_row(repo_root, output_root, &pseudohaploid_row)?;
    let stats_row = run_vcf_stats_stage_row(repo_root, output_root, &damage_filter_row)?;

    Ok(vec![
        validate_row,
        trim_row,
        dedup_row,
        align_row,
        validate_bam_row,
        mapping_row,
        coverage_row,
        damage_row,
        authenticity_row,
        contamination_row,
        pseudohaploid_row,
        gl_call_row,
        gl_propagation_row,
        damage_filter_row,
        stats_row,
    ])
}
