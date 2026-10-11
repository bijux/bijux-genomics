use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations;
pub(crate) use bijux_dna_bench_vcf::evidence::expectations::collect_vcf_expected_benchmark_result_rows as collect_vcf_expected_benchmark_result_rows_with_observations;
pub(crate) use bijux_dna_bench_vcf::evidence::expectations::DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH;
pub(crate) use bijux_dna_bench_vcf_evidence::model::expectations::{
    VcfExpectedBenchmarkResultRow, VcfExpectedBenchmarkResultsReport,
};
use std::path::{Path, PathBuf};

pub(crate) fn run_render_vcf_expected_benchmark_results(
    args: &parse::BenchReadinessRenderVcfExpectedBenchmarkResultsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_expected_benchmark_results(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_expected_benchmark_results(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfExpectedBenchmarkResultsReport> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = VcfAdapterObservations::new(&inputs, repo_root);
    bijux_dna_bench_vcf::evidence::expectations::render_vcf_expected_benchmark_results(
        &observations,
        &output_path,
    )
}

pub(crate) fn collect_vcf_expected_benchmark_result_rows(
    repo_root: &Path,
) -> Result<Vec<VcfExpectedBenchmarkResultRow>> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = VcfAdapterObservations::new(&inputs, repo_root);
    collect_vcf_expected_benchmark_result_rows_with_observations(&observations)
}
