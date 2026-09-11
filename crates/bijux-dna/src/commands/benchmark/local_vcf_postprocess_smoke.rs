use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};

pub(crate) fn run_vcf_postprocess_smoke(
    args: &parse::BenchLocalRunVcfPostprocessSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_postprocess_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_vcf_path);
    }
    Ok(())
}

pub(crate) use bijux_dna_bench_variants::postprocess::LocalVcfPostprocessSmokeReport;

pub(crate) fn run_local_vcf_postprocess_smoke(
    repo_root: &std::path::Path,
    tool_id: &str,
) -> Result<LocalVcfPostprocessSmokeReport> {
    let rows = super::local_vcf_stage_matrix::build_vcf_stage_matrix_rows()?
        .into_iter()
        .filter(|row| row.stage_id == bijux_dna_core::id_catalog::VCF_POSTPROCESS)
        .collect();
    let inputs = bijux_dna_bench_variants::inputs::VariantBenchmarkInputs::new(rows, Vec::new())?;
    bijux_dna_bench_variants::postprocess::run_local_vcf_postprocess_smoke(
        &inputs, repo_root, tool_id,
    )
}
