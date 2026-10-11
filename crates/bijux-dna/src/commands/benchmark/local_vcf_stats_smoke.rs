use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_variants::statistics::LocalVcfStatsSmokeReport;
pub(crate) fn run_vcf_stats_smoke(args: &parse::BenchLocalRunVcfStatsSmokeArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_stats_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.stats_json_path);
    }
    Ok(())
}
pub(crate) fn run_local_vcf_stats_smoke(
    repo_root: &std::path::Path,
    tool_id: &str,
) -> Result<LocalVcfStatsSmokeReport> {
    let rows = super::local_vcf_stage_matrix::build_vcf_stage_matrix_rows()?
        .into_iter()
        .filter(|row| row.stage_id == "vcf.stats")
        .collect();
    let inputs = bijux_dna_bench_variants::inputs::VariantBenchmarkInputs::new(rows, Vec::new())?;
    bijux_dna_bench_variants::statistics::run_local_vcf_stats_smoke(&inputs, repo_root, tool_id)
}
