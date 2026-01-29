use std::collections::HashMap;

use anyhow::{anyhow, Result};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};

use bijux_environment::image_qa::{ensure_image_qa_passed, ensure_tool_qa_passed};

use crate::fastq_exec::helpers::filter_tools_by_role;
use bijux_stages_fastq::fastq::screen::{normalize_screen_tool_list, plan_screen};

/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning, execution, or metric recording fails.
pub fn bench_fastq_screen<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqScreenArgs,
) -> Result<()> {
    let _ = runner_override;
    let tools = normalize_screen_tool_list(&args.tools)?;
    let registry = bijux_engine::api::load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.screen", &tools, &registry, false)?;
    for tool in &tools {
        let _plan = plan_screen(tool, &args.r1, &args.out)?;
    }
    if std::env::var("BIJUX_SCREEN_DB").is_err() {
        return Err(anyhow!("BIJUX_SCREEN_DB not set; screen cannot run"));
    }
    ensure_image_qa_passed("fastq.screen", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.screen", &tools, platform, catalog)?;
    Err(anyhow!("screen benchmarking not implemented yet"))
}
