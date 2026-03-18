use std::collections::HashMap;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_registry};
use anyhow::{anyhow, Context, Result};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir};
use bijux_dna_planner_fastq::select_umi_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::umi::plan_umi;
use bijux_dna_planner_fastq::stage_api::FastqArtifact;
use bijux_dna_planner_fastq::stage_api::{
    ensure_umi_headers, inspect_headers, log_header_warnings, preflight_stage, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;

use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_UMI,
};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_umi<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqUmiArgs,
) -> Result<BenchOutcome<bijux_dna_analyze::FastqUmiMetrics>> {
    let tools = select_umi_tools(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage(STAGE_UMI.as_str(), artifact.kind)?;
    let r2 = args
        .r2
        .as_deref()
        .ok_or_else(|| anyhow!("umi stage requires paired-end input"))?;
    let header = inspect_headers(&args.r1, Some(r2), false)?;
    log_header_warnings(STAGE_UMI.as_str(), &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_UMI.as_str(), &tools, &registry, false)?;

    let bench_dir_name = bench_dir_name(&STAGE_UMI)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_UMI.as_str()))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_dna_infra::ensure_dir(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, STAGE_UMI.as_str(), &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, STAGE_UMI.as_str(), &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed(STAGE_UMI.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_UMI.as_str(), &tools, platform, catalog)?;

    ensure_umi_headers(&args.r1, args.r2.as_deref())?;

    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut plans = Vec::new();
    let mut tool_order: Vec<String> = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec =
            build_tool_execution_spec(STAGE_UMI.as_str(), tool, &registry, catalog, platform)?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = plan_umi(&tool_spec, &args.r1, r2, &out_dir)?;
        plans.push(bijux_dna_stage_contract::execution_step_from_stage_plan(
            &plan,
        ));
        tool_order.push(tool.clone());
    }
    let executions = execute_plans_with_jobs(plans, platform.runner, jobs)?;
    for (tool, execution) in tool_order.into_iter().zip(executions.into_iter()) {
        if execution.exit_code != 0 {
            let tool_name = tool.clone();
            failures.push(RawFailure {
                stage: STAGE_UMI.as_str().to_string(),
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
