use crate::commands::cli::{parse, render};
use anyhow::Result;
pub(crate) use bijux_dna_bench_workflows::core_germline::{
    CoreGermlineMicroPipelineReport, CoreGermlineMicroPipelineRow,
    DEFAULT_CORE_GERMLINE_MICRO_PIPELINE_PATH,
};
use std::path::PathBuf;

pub(crate) fn run_core_germline_micro_pipeline(
    args: &parse::BenchLocalRunCoreGermlineMicroPipelineArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let report = render_core_germline_micro_pipeline(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CORE_GERMLINE_MICRO_PIPELINE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_core_germline_micro_pipeline(
    repo_root: &std::path::Path,
    output_path: PathBuf,
) -> Result<CoreGermlineMicroPipelineReport> {
    let inputs = super::micro_pipeline_inputs::core_germline()?;
    bijux_dna_bench_workflows::core_germline::render_core_germline_micro_pipeline(
        &inputs,
        repo_root,
        output_path,
    )
}
