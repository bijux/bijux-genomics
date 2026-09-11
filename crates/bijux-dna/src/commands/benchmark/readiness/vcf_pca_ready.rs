use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_population::readiness::pca::{
    VcfPcaReadyReport, DEFAULT_VCF_PCA_READY_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_vcf_pca_ready(
    args: &parse::BenchReadinessRenderVcfPcaReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_pca_ready(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_PCA_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_pca_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfPcaReadyReport> {
    let adapter_inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations::new(
        &adapter_inputs,
        repo_root,
    );
    let (_, snapshot) =
        super::vcf_stage_readiness_support::collect_vcf_readiness_evidence_with_observations(
            &observations,
        )?;
    let prepared = bijux_dna_bench_vcf::evidence::pca::prepare_pca_adapter_evidence(&observations)?;
    let command_report =
        bijux_dna_bench_vcf::evidence::pca::publish_pca_commands(repo_root, &prepared)?;
    let inputs = crate::commands::benchmark::population_inputs::supported(repo_root)?;
    let command_paths = bijux_dna_bench_population::readiness::ReadinessCommandPaths {
        output_path: command_report.output_path,
        argv_output_path: command_report.argv_output_path,
    };
    bijux_dna_bench_population::readiness::pca::render_vcf_pca_ready(
        &inputs,
        &snapshot,
        prepared.rows(),
        &command_paths,
        repo_root,
        &output_path,
    )
}
