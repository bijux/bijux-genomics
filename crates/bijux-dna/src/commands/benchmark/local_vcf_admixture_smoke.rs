use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};

pub(crate) fn run_vcf_admixture_smoke(
    args: &parse::BenchLocalRunVcfAdmixtureSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_admixture_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.admixture_json_path);
    }
    Ok(())
}

pub(crate) use bijux_dna_bench_population::admixture::LocalVcfAdmixtureSmokeReport;

pub(crate) fn run_local_vcf_admixture_smoke(
    repo_root: &std::path::Path,
    tool_id: &str,
) -> Result<LocalVcfAdmixtureSmokeReport> {
    let inputs = super::population_inputs::supported(repo_root)?;
    bijux_dna_bench_population::admixture::run_local_vcf_admixture_smoke(
        &inputs, repo_root, tool_id,
    )
}

pub(crate) use bijux_dna_bench_population::admixture::LocalVcfAdmixtureSmokeRow;
