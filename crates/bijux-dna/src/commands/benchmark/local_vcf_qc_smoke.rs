use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};

pub(crate) fn run_vcf_qc_smoke(args: &parse::BenchLocalRunVcfQcSmokeArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_qc_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.qc_json_path);
    }
    Ok(())
}

pub(crate) use bijux_dna_bench_variants::qc::{
    LocalVcfQcSmokeHeterozygosity, LocalVcfQcSmokeHweSummary, LocalVcfQcSmokeMafSummary,
    LocalVcfQcSmokeReport, LocalVcfQcSmokeSampleMissingnessRow,
    LocalVcfQcSmokeVariantMissingnessRow,
};

pub(crate) fn run_local_vcf_qc_smoke(
    repo_root: &std::path::Path,
    tool_id: &str,
) -> Result<LocalVcfQcSmokeReport> {
    let bindings = super::vcf_benchmark_bindings::collect_vcf_qc_benchmark_binding_rows()?;
    let inputs =
        bijux_dna_bench_variants::inputs::VariantBenchmarkInputs::new(Vec::new(), bindings)?;
    bijux_dna_bench_variants::qc::run_local_vcf_qc_smoke(&inputs, repo_root, tool_id)
}
