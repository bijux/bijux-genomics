use crate::commands::cli::parse;
use crate::commands::cli::render;
use anyhow::Context;
use anyhow::Result;

pub(crate) fn run_vcf_prepare_reference_panel_smoke(
    args: &parse::BenchLocalRunVcfPrepareReferencePanelSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_prepare_reference_panel_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.panel_vcf_path);
    }
    Ok(())
}

pub(crate) use bijux_dna_bench_haplotypes::reference_panel::LocalVcfPrepareReferencePanelSmokeReport;

pub(crate) fn run_local_vcf_prepare_reference_panel_smoke(
    repo_root: &std::path::Path,
    tool_id: &str,
) -> Result<LocalVcfPrepareReferencePanelSmokeReport> {
    let inputs = super::haplotype_inputs::governed()?;
    bijux_dna_bench_haplotypes::reference_panel::run_local_vcf_prepare_reference_panel_smoke(
        &inputs, repo_root, tool_id,
    )
}
