use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_vcf::evidence::parsers::failure::{
    VcfParserFailureTestsReport, DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_vcf_parser_failure_audit(
    args: &parse::BenchReadinessRenderVcfParserFailureTestsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_parser_failure_audit(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_parser_failure_audit(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfParserFailureTestsReport> {
    bijux_dna_bench_vcf::evidence::parsers::failure::render_vcf_parser_failure_audit(
        repo_root,
        &output_path,
    )
}
