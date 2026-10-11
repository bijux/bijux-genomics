use crate::commands::benchmark::schema_paths::{
    DEFAULT_VCF_NORMALIZED_METRICS_SCHEMA_PATH, DEFAULT_VCF_NORMALIZED_METRICS_STAGE_DIR,
};
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_artifacts::schemas::vcf_report::{
    collect_vcf_normalized_metrics_schema_report_rows, VcfNormalizedMetricsSchemaReport,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_vcf_normalized_metrics_schema(
    args: &parse::BenchReadinessRenderVcfNormalizedMetricsSchemaArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_normalized_metrics_schema(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_NORMALIZED_METRICS_SCHEMA_PATH)),
        args.stage_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_NORMALIZED_METRICS_STAGE_DIR)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_normalized_metrics_schema(
    repo_root: &Path,
    output_path: PathBuf,
    stage_dir: PathBuf,
) -> Result<VcfNormalizedMetricsSchemaReport> {
    bijux_dna_bench_artifacts::schemas::vcf_report::render_vcf_normalized_metrics_schema(
        repo_root,
        &output_path,
        &stage_dir,
    )
}
