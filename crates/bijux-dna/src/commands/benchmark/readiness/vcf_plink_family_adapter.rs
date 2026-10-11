use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_vcf::adapters::plink::{
    VcfPlinkFamilyAdapterReport, VcfPlinkFamilyAdapterRow, DEFAULT_VCF_PLINK2_ADAPTER_PATH,
    DEFAULT_VCF_PLINK_ADAPTER_PATH,
};
use std::path::{Path, PathBuf};

pub(crate) fn run_render_vcf_plink_adapter(
    args: &parse::BenchReadinessRenderVcfPlinkAdapterArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_plink_family_adapter(
        &repo_root,
        "plink",
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_PLINK_ADAPTER_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn run_render_vcf_plink2_adapter(
    args: &parse::BenchReadinessRenderVcfPlink2AdapterArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_plink_family_adapter(
        &repo_root,
        "plink2",
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_PLINK2_ADAPTER_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_plink_family_adapter(
    repo_root: &Path,
    tool_id: &str,
    output_path: PathBuf,
) -> Result<VcfPlinkFamilyAdapterReport> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    bijux_dna_bench_vcf::adapters::plink::render_vcf_plink_family_adapter(
        &inputs,
        repo_root,
        tool_id,
        &output_path,
    )
}

pub(crate) fn collect_vcf_plink_family_adapter_rows_for_tool(
    repo_root: &Path,
    tool_id: &str,
) -> Result<Vec<VcfPlinkFamilyAdapterRow>> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    bijux_dna_bench_vcf::adapters::plink::collect_vcf_plink_family_adapter_rows_for_tool(
        &inputs, repo_root, tool_id,
    )
}
