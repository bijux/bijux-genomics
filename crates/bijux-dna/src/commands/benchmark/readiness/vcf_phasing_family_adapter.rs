use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_vcf::adapters::phasing::{
    VcfPhasingFamilyAdapterReport, VcfPhasingFamilyAdapterRow, DEFAULT_VCF_BEAGLE_ADAPTER_PATH,
    DEFAULT_VCF_EAGLE_ADAPTER_PATH, DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH,
    DEFAULT_VCF_SHAPEIT_ADAPTER_PATH,
};
use std::path::{Path, PathBuf};

pub(crate) fn run_render_vcf_shapeit5_adapter(
    args: &parse::BenchReadinessRenderVcfShapeit5AdapterArgs,
) -> Result<()> {
    run_render_tool_adapter(
        "shapeit5",
        args.output.clone(),
        args.json,
        DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH,
    )
}

pub(crate) fn run_render_vcf_eagle_adapter(
    args: &parse::BenchReadinessRenderVcfEagleAdapterArgs,
) -> Result<()> {
    run_render_tool_adapter("eagle", args.output.clone(), args.json, DEFAULT_VCF_EAGLE_ADAPTER_PATH)
}

pub(crate) fn run_render_vcf_beagle_adapter(
    args: &parse::BenchReadinessRenderVcfBeagleAdapterArgs,
) -> Result<()> {
    run_render_tool_adapter(
        "beagle",
        args.output.clone(),
        args.json,
        DEFAULT_VCF_BEAGLE_ADAPTER_PATH,
    )
}

fn run_render_tool_adapter(
    tool_id: &str,
    output: Option<PathBuf>,
    json: bool,
    default_output_path: &str,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_phasing_family_adapter(
        &repo_root,
        tool_id,
        output.unwrap_or_else(|| PathBuf::from(default_output_path)),
    )?;
    if json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_phasing_family_adapter(
    repo_root: &Path,
    tool_id: &str,
    output_path: PathBuf,
) -> Result<VcfPhasingFamilyAdapterReport> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    bijux_dna_bench_vcf::adapters::phasing::render_vcf_phasing_family_adapter(
        &inputs,
        repo_root,
        tool_id,
        &output_path,
    )
}

pub(crate) fn collect_vcf_phasing_family_adapter_rows_for_tool(
    repo_root: &Path,
    tool_id: &str,
) -> Result<Vec<VcfPhasingFamilyAdapterRow>> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    bijux_dna_bench_vcf::adapters::phasing::collect_vcf_phasing_family_adapter_rows_for_tool(
        &inputs, repo_root, tool_id,
    )
}
