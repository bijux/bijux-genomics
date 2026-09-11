use crate::commands::cli::{parse, render};
use anyhow::Result;
pub(crate) use bijux_dna_bench_vcf::evidence::results::missing::{
    VcfMissingResultReport, DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_vcf_missing_result_report(
    args: &parse::BenchReadinessRenderVcfMissingResultReportArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let report = render_vcf_missing_result_report(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_missing_result_report(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfMissingResultReport> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations::new(
        &inputs, repo_root,
    );
    let expected =
        bijux_dna_bench_vcf::evidence::expectations::collect_vcf_expected_benchmark_result_rows(
            &observations,
        )?;
    bijux_dna_bench_vcf::evidence::results::missing::render_vcf_missing_result_report(
        &inputs,
        &expected,
        repo_root,
        &output_path,
    )
}
