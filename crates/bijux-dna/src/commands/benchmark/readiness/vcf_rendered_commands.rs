use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations;
pub(crate) use bijux_dna_bench_vcf::evidence::commands::DEFAULT_VCF_RENDERED_COMMANDS_PATH;
pub(crate) use bijux_dna_bench_vcf_evidence::model::command_reports::VcfRenderedCommandsReport;
use std::path::{Path, PathBuf};
pub(crate) fn run_render_vcf_commands(
    args: &parse::BenchReadinessRenderVcfCommandsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_commands(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_RENDERED_COMMANDS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_commands(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfRenderedCommandsReport> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = VcfAdapterObservations::new(&inputs, repo_root);
    let expected=super::vcf_expected_benchmark_results::collect_vcf_expected_benchmark_result_rows_with_observations(&observations)?;
    let retained=super::vcf_active_stage_tool_matrix::collect_vcf_active_stage_tool_matrix_rows_from_expected(repo_root, &inputs,expected)?;
    render_vcf_commands_with_observations(repo_root, output_path, &observations, &retained)
}
pub(crate) fn render_vcf_commands_with_observations(
    repo_root: &Path,
    output_path: PathBuf,
    observations: &VcfAdapterObservations<'_>,
    retained: &[bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow],
) -> Result<VcfRenderedCommandsReport> {
    bijux_dna_bench_vcf::evidence::commands::render_vcf_commands_with_observations(
        repo_root,
        &output_path,
        observations,
        retained,
    )
}
