use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_vcf::evidence::inputs::{
    VcfAdapterMissingInputTestsReport, DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_vcf_adapter_missing_input_audit(
    args: &parse::BenchReadinessRenderVcfAdapterMissingInputTestsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_adapter_missing_input_audit(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_adapter_missing_input_audit(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfAdapterMissingInputTestsReport> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    bijux_dna_bench_vcf::evidence::inputs::render_vcf_adapter_missing_input_audit(
        &inputs,
        repo_root,
        &output_path,
    )
}
