use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_vcf::adapters::eigensoft::{
    VcfEigensoftAdapterArtifact, VcfEigensoftAdapterReport, VcfEigensoftAdapterRow,
    DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH,
};
use std::path::{Path, PathBuf};

pub(crate) fn run_render_vcf_eigensoft_adapter(
    args: &parse::BenchReadinessRenderVcfEigensoftAdapterArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_eigensoft_adapter(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_eigensoft_adapter(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfEigensoftAdapterReport> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    bijux_dna_bench_vcf::adapters::eigensoft::render_vcf_eigensoft_adapter(
        &inputs,
        repo_root,
        &output_path,
    )
}

pub(crate) fn collect_vcf_eigensoft_adapter_rows_for_tool(
    repo_root: &Path,
) -> Result<Vec<VcfEigensoftAdapterRow>> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    bijux_dna_bench_vcf::adapters::eigensoft::collect_vcf_eigensoft_adapter_rows_for_tool(
        &inputs, repo_root,
    )
}
