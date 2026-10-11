use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_genotypes::readiness::gl_propagation::{
    VcfGlPropagationReadyReport, DEFAULT_VCF_GL_PROPAGATION_READY_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_vcf_gl_propagation_ready(
    args: &parse::BenchReadinessRenderVcfGlPropagationReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_gl_propagation_ready(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_GL_PROPAGATION_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_gl_propagation_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfGlPropagationReadyReport> {
    let (commands, snapshot) =
        super::vcf_stage_readiness_support::collect_vcf_readiness_evidence(repo_root)?;
    let rows = crate::commands::benchmark::local_vcf_stage_matrix::build_vcf_stage_matrix_rows()?
        .into_iter()
        .filter(|row| row.stage_id == "vcf.gl_propagation")
        .collect();
    let inputs = bijux_dna_bench_genotypes::inputs::GenotypeBenchmarkInputs::new(rows)?;
    let command_paths = bijux_dna_bench_genotypes::readiness::ReadinessCommandPaths {
        output_path: commands.output_path,
        argv_output_path: commands.argv_output_path,
    };
    bijux_dna_bench_genotypes::readiness::gl_propagation::render_vcf_gl_propagation_ready(
        &inputs,
        &snapshot,
        &command_paths,
        repo_root,
        &output_path,
    )
}
