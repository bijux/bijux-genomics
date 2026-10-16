use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_vcf::evidence::metrics::{
    ComparableMetricDirection, VcfComparableMetricsReport, DEFAULT_VCF_COMPARABLE_METRICS_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_vcf_comparable_metrics(
    args: &parse::BenchReadinessRenderVcfComparableMetricsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_comparable_metrics(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_COMPARABLE_METRICS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_comparable_metrics(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfComparableMetricsReport> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations::new(
        &inputs, repo_root,
    );
    bijux_dna_bench_vcf::evidence::metrics::render_vcf_comparable_metrics(
        &observations,
        &output_path,
    )
}
