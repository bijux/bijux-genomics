use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_workflows::adna::{
    AdnaMicroPipelineReport, AdnaMicroPipelineRow, AdnaMicroPipelineRowStatus,
    DEFAULT_ADNA_MICRO_PIPELINE_PATH,
};
use std::path::PathBuf;

pub(crate) fn run_adna_micro_pipeline(
    args: &parse::BenchLocalRunAdnaMicroPipelineArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_adna_micro_pipeline(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_ADNA_MICRO_PIPELINE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_adna_micro_pipeline(
    repo_root: &std::path::Path,
    output_path: PathBuf,
) -> Result<AdnaMicroPipelineReport> {
    let inputs = super::micro_pipeline_inputs::alignment();
    bijux_dna_bench_workflows::adna::render_adna_micro_pipeline(&inputs, repo_root, output_path)
}
