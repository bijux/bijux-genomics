use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_population::readiness::population_structure::{
    VcfPopulationStructureReadyReport, DEFAULT_VCF_POPULATION_STRUCTURE_READY_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_vcf_population_structure_ready(
    args: &parse::BenchReadinessRenderVcfPopulationStructureReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_population_structure_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_POPULATION_STRUCTURE_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_population_structure_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfPopulationStructureReadyReport> {
    let (commands, snapshot) =
        super::vcf_stage_readiness_support::collect_vcf_readiness_evidence(repo_root)?;
    render_vcf_population_structure_ready_with_evidence(
        repo_root,
        output_path,
        &commands,
        &snapshot,
    )
}

pub(crate) fn render_vcf_population_structure_ready_with_evidence(
    repo_root: &Path,
    output_path: PathBuf,
    commands: &bijux_dna_bench_vcf_evidence::model::command_reports::VcfRenderedCommandsReport,
    snapshot: &bijux_dna_bench_vcf_evidence::snapshot::VcfEvidenceSnapshot,
) -> Result<VcfPopulationStructureReadyReport> {
    let inputs = crate::commands::benchmark::population_inputs::supported(repo_root)?;
    let command_paths = bijux_dna_bench_population::readiness::ReadinessCommandPaths {
        output_path: commands.output_path.clone(),
        argv_output_path: commands.argv_output_path.clone(),
    };
    bijux_dna_bench_population::readiness::population_structure::render_vcf_population_structure_ready(&inputs,snapshot,&command_paths,repo_root,&output_path)
}
