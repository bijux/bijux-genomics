use std::collections::HashMap;

use anyhow::{anyhow, Result};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};

use crate::image_qa::ensure_image_qa_passed;

use super::helpers::normalize_screen_tool_list;

/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning, execution, or metric recording fails.
pub fn bench_fastq_screen<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::bench::args::BenchFastqScreenArgs,
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
