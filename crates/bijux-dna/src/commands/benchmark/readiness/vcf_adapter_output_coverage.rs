use crate::commands::benchmark::local_slurm_run_paths::LOCAL_SLURM_DRY_RUN_RUN_ID;
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations;
pub(crate) use bijux_dna_bench_vcf::evidence::output_coverage::DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH;
pub(crate) use bijux_dna_bench_vcf_evidence::model::outputs::{
    VcfAdapterOutputCoverageReport, VcfAdapterOutputCoverageRow, VcfAdapterOutputCoverageStatus,
};
use std::path::{Path, PathBuf};

pub(crate) fn run_render_vcf_adapter_output_coverage(
    args: &parse::BenchReadinessRenderVcfAdapterOutputCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_adapter_output_coverage(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_adapter_output_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfAdapterOutputCoverageReport> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = VcfAdapterObservations::new(&inputs, repo_root);
    bijux_dna_bench_vcf::evidence::output_coverage::render_vcf_adapter_output_coverage(
        &observations,
        LOCAL_SLURM_DRY_RUN_RUN_ID,
        &output_path,
    )
}

pub(crate) fn collect_vcf_adapter_output_coverage_rows(
    repo_root: &Path,
) -> Result<Vec<VcfAdapterOutputCoverageRow>> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = VcfAdapterObservations::new(&inputs, repo_root);
    collect_vcf_adapter_output_coverage_rows_with_observations(&observations)
}

pub(crate) fn collect_vcf_adapter_output_coverage_rows_with_observations(
    observations: &VcfAdapterObservations<'_>,
) -> Result<Vec<VcfAdapterOutputCoverageRow>> {
    bijux_dna_bench_vcf::evidence::output_coverage::collect_vcf_adapter_output_coverage_rows(
        observations,
        LOCAL_SLURM_DRY_RUN_RUN_ID,
    )
}
