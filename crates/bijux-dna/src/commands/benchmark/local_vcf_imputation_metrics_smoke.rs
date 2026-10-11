use crate::commands::cli::parse;
use crate::commands::cli::render;
use anyhow::Context;
use anyhow::Result;

pub(crate) fn run_vcf_imputation_metrics_smoke(
    args: &parse::BenchLocalRunVcfImputationMetricsSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_imputation_metrics_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.imputation_metrics_path);
    }
    Ok(())
}

pub(crate) use bijux_dna_bench_haplotypes::imputation_metrics::LocalVcfImputationMetricsSmokeReport;

pub(crate) fn run_local_vcf_imputation_metrics_smoke(
    repo_root: &std::path::Path,
    tool_id: &str,
) -> Result<LocalVcfImputationMetricsSmokeReport> {
    let inputs = super::haplotype_inputs::governed()?;
    bijux_dna_bench_haplotypes::imputation_metrics::run_local_vcf_imputation_metrics_smoke(
        &inputs, repo_root, tool_id,
    )
}
