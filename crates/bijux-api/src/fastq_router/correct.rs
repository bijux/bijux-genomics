use std::collections::HashMap;

use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_registry};
use anyhow::{anyhow, Context, Result};
use bijux_core::ErrorCategory;
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use bijux_environment::image_qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use bijux_infra::{bench_base_dir, bench_tools_dir};
use bijux_planner_fastq::select_correct_tools;
use bijux_runner::primitives::build_tool_execution_spec;
use bijux_stages_fastq::fastq::correct::plan_correct;
use bijux_stages_fastq::FastqArtifactKind;
use bijux_stages_fastq::{inspect_headers, log_header_warnings, preflight_stage, RawFailure};

use super::jobs::execute_plans_with_jobs;

use super::jobs::bench_jobs;
use super::{write_explain_md, write_explain_plan_json, BenchOutcome};
use bijux_planner_fastq::scale_tool_spec_for_jobs;

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_correct<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqCorrectArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqCorrectMetrics>> {
    let tools = select_correct_tools(&args.tools)?;
    let artifact = FastqArtifactKind::PairedEnd;
    preflight_stage("fastq.correct", artifact)?;
    let r2 = args
        .r2
        .as_deref()
        .ok_or_else(|| anyhow!("paired-end correction requires r2 input"))?;
    let header = inspect_headers(&args.r1, Some(r2), false)?;
    log_header_warnings("fastq.correct", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.correct", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "correct", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "correct", &args.sample_id);
    bijux_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_infra::ensure_dir(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.correct", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.correct", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.correct", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.correct", &tools, platform, catalog)?;

    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut plans = Vec::new();
    let mut tool_order = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        bijux_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec =
            build_tool_execution_spec("fastq.correct", tool, &registry, catalog, platform)?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = plan_correct(&tool_spec, &args.r1, r2, &out_dir)?;
        plans.push(plan);
        tool_order.push(tool.clone());
    }
    let executions = execute_plans_with_jobs(plans, platform.runner, jobs)?;
    for (tool, execution) in tool_order.into_iter().zip(executions.into_iter()) {
        if execution.exit_code != 0 {
            let tool_name = tool.clone();
            failures.push(RawFailure {
                stage: "fastq.correct".to_string(),
                tool,
                reason: format!(
                    "tool {tool_name} failed with status {}",
                    execution.exit_code
                ),
                category: ErrorCategory::ToolError,
            });
        }
    }

    Ok(BenchOutcome {
        records: Vec::new(),
        failures,
        bench_dir,
        explain: args.explain,
    })
}
