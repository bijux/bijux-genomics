use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_vcf::evidence::scope::DEFAULT_VCF_ACTIVE_STAGE_TOOL_MATRIX_PATH;
use bijux_dna_bench_vcf::{
    adapters::observations::VcfAdapterObservations, inputs::VcfAdapterInputs,
};
pub(crate) use bijux_dna_bench_vcf_evidence::model::scope::{
    VcfActiveStageToolMatrixReport, VcfActiveStageToolMatrixRow,
};
use bijux_dna_bench_vcf_evidence::{
    model::expectations::VcfExpectedBenchmarkResultRow, snapshot::VcfStageReadinessBindingKey,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_vcf_active_stage_tool_matrix(
    args: &parse::BenchReadinessRenderVcfActiveStageToolMatrixArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_active_stage_tool_matrix(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_ACTIVE_STAGE_TOOL_MATRIX_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_active_stage_tool_matrix(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfActiveStageToolMatrixReport> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = VcfAdapterObservations::new(&inputs, repo_root);
    let expected =
        bijux_dna_bench_vcf::evidence::expectations::collect_vcf_expected_benchmark_result_rows(
            &observations,
        )?;
    let active = active_keys(repo_root)?;
    bijux_dna_bench_vcf::evidence::scope::render_vcf_active_stage_tool_matrix(
        &inputs,
        repo_root,
        &active,
        &expected,
        &output_path,
    )
}
pub(crate) fn collect_vcf_active_stage_tool_matrix_rows(
    repo_root: &Path,
) -> Result<Vec<VcfActiveStageToolMatrixRow>> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = VcfAdapterObservations::new(&inputs, repo_root);
    let expected =
        bijux_dna_bench_vcf::evidence::expectations::collect_vcf_expected_benchmark_result_rows(
            &observations,
        )?;
    collect_vcf_active_stage_tool_matrix_rows_from_expected(repo_root, &inputs, expected)
}
pub(crate) fn collect_vcf_active_stage_tool_matrix_rows_from_expected(
    repo_root: &Path,
    inputs: &VcfAdapterInputs,
    expected: Vec<VcfExpectedBenchmarkResultRow>,
) -> Result<Vec<VcfActiveStageToolMatrixRow>> {
    let active = active_keys(repo_root)?;
    bijux_dna_bench_vcf::evidence::scope::collect_vcf_active_stage_tool_matrix_rows_from_expected(
        inputs, repo_root, &active, &expected,
    )
}
pub(super) fn active_keys(repo_root: &Path) -> Result<Vec<VcfStageReadinessBindingKey>> {
    Ok(super::all_domain_active_stage_tool_matrix::collect_all_domain_active_stage_tool_matrix_rows(repo_root)?.into_iter().filter(|row|row.domain=="vcf").map(|row|VcfStageReadinessBindingKey {stage_id:row.stage_id,tool_id:row.tool_id,corpus_id:row.corpus_id,asset_profile_id:row.asset_profile_id}).collect())
}
