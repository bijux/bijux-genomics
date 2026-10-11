use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};

pub(crate) fn run_vcf_demography_smoke(
    args: &parse::BenchLocalRunVcfDemographySmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_demography_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.demography_json_path);
    }
    Ok(())
}

pub(crate) use bijux_dna_bench_population::demography::LocalVcfDemographySmokeReport;

pub(crate) fn run_local_vcf_demography_smoke(
    repo_root: &std::path::Path,
    tool_id: &str,
) -> Result<LocalVcfDemographySmokeReport> {
    let inputs = super::population_inputs::ibd(repo_root)?;
    bijux_dna_bench_population::demography::run_local_vcf_demography_smoke(
        &inputs, repo_root, tool_id,
    )
}
