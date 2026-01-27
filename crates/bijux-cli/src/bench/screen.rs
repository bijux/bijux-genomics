use anyhow::{anyhow, Result};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};

use crate::image_qa::ensure_image_qa_passed;

use super::helpers::normalize_screen_tool_list;

pub fn bench_fastq_screen(
    catalog: &std::collections::HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::cli::BenchFastqScreenArgs,
) -> Result<()> {
    let _ = runner_override;
    let tools = normalize_screen_tool_list(&args.tools)?;
    if std::env::var("BIJUX_SCREEN_DB").is_err() {
        println!("screen benchmarks skipped (BIJUX_SCREEN_DB not set)");
        return Ok(());
    }
    ensure_image_qa_passed("fastq.screen", &tools, platform, catalog)?;
    Err(anyhow!("screen benchmarking not implemented yet"))
}
