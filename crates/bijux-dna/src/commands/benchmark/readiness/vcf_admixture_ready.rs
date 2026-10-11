use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_population::readiness::admixture::{
    VcfAdmixtureReadyReport, DEFAULT_VCF_ADMIXTURE_READY_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_vcf_admixture_ready(
    args: &parse::BenchReadinessRenderVcfAdmixtureReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_admixture_ready(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_ADMIXTURE_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_admixture_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfAdmixtureReadyReport> {
    let (commands, snapshot) =
        super::vcf_stage_readiness_support::collect_vcf_readiness_evidence(repo_root)?;
    let inputs = crate::commands::benchmark::population_inputs::supported(repo_root)?;
    let command_paths = bijux_dna_bench_population::readiness::ReadinessCommandPaths {
        output_path: commands.output_path,
        argv_output_path: commands.argv_output_path,
    };
    bijux_dna_bench_population::readiness::admixture::render_vcf_admixture_ready(
        &inputs,
        &snapshot,
        &command_paths,
        repo_root,
        &output_path,
    )
}
