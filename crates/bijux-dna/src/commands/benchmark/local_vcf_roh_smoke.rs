use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};

pub(crate) fn run_vcf_roh_smoke(args: &parse::BenchLocalRunVcfRohSmokeArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_roh_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.roh_json_path);
    }
    Ok(())
}

pub(crate) use bijux_dna_bench_population::roh::LocalVcfRohSmokeReport;

pub(crate) fn run_local_vcf_roh_smoke(
    repo_root: &std::path::Path,
    tool_id: &str,
) -> Result<LocalVcfRohSmokeReport> {
    let inputs = super::population_inputs::governed(repo_root)?;
    bijux_dna_bench_population::roh::run_local_vcf_roh_smoke(&inputs, repo_root, tool_id)
}
