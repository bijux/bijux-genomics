use crate::commands::benchmark::path_resolution::BenchmarkPathResolver;
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_api::v1::api::bench::validate_benchmark_local_slurm_script_bodies as validate_slurm_script_bodies;

pub(crate) fn run_validate_slurm_script_bodies(
    args: &parse::BenchLocalValidateSlurmScriptBodiesArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let root_path =
        args.root.clone().unwrap_or_else(|| benchmark_paths.benchmark_slurm_dry_run_root());
    let report_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_slurm_dry_run_root().join("no-placeholder-report.json")
    });
    let report = validate_slurm_script_bodies(&repo_root, root_path, report_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.report_path);
    }
    Ok(())
}
