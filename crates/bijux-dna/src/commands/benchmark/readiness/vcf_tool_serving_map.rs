use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_vcf::evidence::serving::{
    VcfToolServingMapReport, VcfToolServingMapRow, DEFAULT_VCF_TOOL_SERVING_MAP_PATH,
};
use std::path::{Path, PathBuf};

pub(crate) fn run_render_vcf_tool_serving_map(
    args: &parse::BenchReadinessRenderVcfToolServingMapArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_tool_serving_map(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_TOOL_SERVING_MAP_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_tool_serving_map(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfToolServingMapReport> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    bijux_dna_bench_vcf::evidence::serving::render_vcf_tool_serving_map(
        &inputs,
        repo_root,
        &output_path,
    )
}

pub(crate) fn collect_vcf_tool_serving_map_rows() -> Result<Vec<VcfToolServingMapRow>> {
    let root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let inputs = super::vcf_adapter_inputs::load(&root)?;
    bijux_dna_bench_vcf::evidence::serving::collect_vcf_tool_serving_map_rows(&inputs)
}
