use crate::commands::cli::parse;
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_workflows::amplicon::{
    render_amplicon_micro_pipeline, AmpliconMicroPipelineReport, AmpliconMicroPipelineRow,
    DEFAULT_AMPLICON_MICRO_PIPELINE_PATH,
};
use std::path::PathBuf;

pub(crate) fn run_amplicon_micro_pipeline(
    args: &parse::BenchLocalRunAmpliconMicroPipelineArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_amplicon_micro_pipeline(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_AMPLICON_MICRO_PIPELINE_PATH)),
    )?;
    if args.json {
        println!("{}", serde_json::to_string(&report)?);
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
