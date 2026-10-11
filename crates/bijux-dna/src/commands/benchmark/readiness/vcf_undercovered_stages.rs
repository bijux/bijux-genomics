use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_vcf::evidence::registry::undercoverage::{
    VcfUndercoveredStagesReport, DEFAULT_VCF_UNDERCOVERED_STAGES_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_vcf_undercovered_stages(
    args: &parse::BenchReadinessRenderVcfUndercoveredStagesArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_undercovered_stages(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_UNDERCOVERED_STAGES_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_undercovered_stages(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfUndercoveredStagesReport> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    bijux_dna_bench_vcf::evidence::registry::undercoverage::render_vcf_undercovered_stages(
        &inputs,
        repo_root,
        &output_path,
    )
}
