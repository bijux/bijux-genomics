use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};

pub(crate) fn run_vcf_call_diploid_smoke(
    args: &parse::BenchLocalRunVcfCallDiploidSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_call_diploid_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_vcf_path);
    }
    Ok(())
}

pub(crate) use bijux_dna_bench_genotypes::call_diploid::LocalVcfCallDiploidSmokeReport;

pub(crate) fn run_local_vcf_call_diploid_smoke(
    repo_root: &std::path::Path,
    tool_id: &str,
) -> Result<LocalVcfCallDiploidSmokeReport> {
    let rows = super::local_vcf_stage_matrix::build_vcf_stage_matrix_rows()?
        .into_iter()
        .filter(|row| row.stage_id == bijux_dna_core::id_catalog::VCF_CALL_DIPLOID)
        .collect();
    let inputs = bijux_dna_bench_genotypes::inputs::GenotypeBenchmarkInputs::new(rows)?;
    bijux_dna_bench_genotypes::call_diploid::run_local_vcf_call_diploid_smoke(
        &inputs, repo_root, tool_id,
    )
}
