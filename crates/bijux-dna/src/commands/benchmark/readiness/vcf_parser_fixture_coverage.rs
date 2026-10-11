use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_vcf::evidence::parsers::DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH;
use bijux_dna_bench_vcf::{
    adapters::observations::VcfAdapterObservations, inputs::VcfAdapterInputs,
};
pub(crate) use bijux_dna_bench_vcf_evidence::model::parsers::{
    VcfParserFixtureCoverageReport, VcfParserFixtureCoverageRow, VcfParserFixtureCoverageStatus,
};
use bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow;
use std::path::{Path, PathBuf};
pub(crate) fn run_render_vcf_parser_fixture_coverage(
    args: &parse::BenchReadinessRenderVcfParserFixtureCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_parser_fixture_coverage(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_parser_fixture_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfParserFixtureCoverageReport> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let retained = retained_rows(&inputs, repo_root)?;
    bijux_dna_bench_vcf::evidence::parsers::render_vcf_parser_fixture_coverage(
        &inputs,
        repo_root,
        &retained,
        &output_path,
    )
}
pub(crate) fn collect_vcf_parser_fixture_coverage_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<VcfParserFixtureCoverageRow>)> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let retained = retained_rows(&inputs, repo_root)?;
    bijux_dna_bench_vcf::evidence::parsers::collect_vcf_parser_fixture_coverage_rows_from_retained(
        &inputs, repo_root, &retained,
    )
}
fn retained_rows(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
) -> Result<Vec<VcfActiveStageToolMatrixRow>> {
    let observations = VcfAdapterObservations::new(inputs, repo_root);
    let expected =
        bijux_dna_bench_vcf::evidence::expectations::collect_vcf_expected_benchmark_result_rows(
            &observations,
        )?;
    super::vcf_active_stage_tool_matrix::collect_vcf_active_stage_tool_matrix_rows_from_expected(
        repo_root, inputs, expected,
    )
}
