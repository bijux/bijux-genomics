use crate::commands::cli::{parse, render};
use anyhow::Result;
use bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations;
pub(crate) use bijux_dna_bench_vcf::evidence::reports::{
    collect_vcf_report_map_rows_from_expected, DEFAULT_VCF_REPORT_MAP_PATH,
};
pub(crate) use bijux_dna_bench_vcf_evidence::model::reports::{
    VcfReportMapReport, VcfReportMapRow,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_vcf_report_map(
    args: &parse::BenchReadinessRenderVcfReportMapArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let report = render_vcf_report_map(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_REPORT_MAP_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_report_map(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfReportMapReport> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = VcfAdapterObservations::new(&inputs, repo_root);
    bijux_dna_bench_vcf::evidence::reports::render_vcf_report_map(&observations, &output_path)
}
pub(crate) fn collect_vcf_report_map_rows(repo_root: &Path) -> Result<Vec<VcfReportMapRow>> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = VcfAdapterObservations::new(&inputs, repo_root);
    let expected =
        bijux_dna_bench_vcf::evidence::expectations::collect_vcf_expected_benchmark_result_rows(
            &observations,
        )?;
    collect_vcf_report_map_rows_from_expected(&inputs, expected)
}
