use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_vcf::adapters::bcftools::{
    VcfBcftoolsAdapterReport, VcfBcftoolsAdapterRow, DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH,
};
use std::path::{Path, PathBuf};

pub(crate) fn run_render_vcf_bcftools_adapter(
    args: &parse::BenchReadinessRenderVcfBcftoolsAdapterArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_bcftools_adapter(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_bcftools_adapter(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfBcftoolsAdapterReport> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    bijux_dna_bench_vcf::adapters::bcftools::render_vcf_bcftools_adapter(
        &inputs,
        repo_root,
        &output_path,
    )
}

pub(crate) fn collect_vcf_bcftools_adapter_rows(
    repo_root: &Path,
) -> Result<Vec<VcfBcftoolsAdapterRow>> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    bijux_dna_bench_vcf::adapters::bcftools::collect_vcf_bcftools_adapter_rows(&inputs, repo_root)
}
