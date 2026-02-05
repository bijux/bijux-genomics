use std::collections::HashMap;

use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_registry};
use anyhow::{anyhow, Context, Result};
use bijux_core::primitives::errors::ErrorCategory;
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use bijux_environment::image_qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use bijux_infra::{bench_base_dir, bench_tools_dir};
use bijux_planner_fastq::select_trim_tools;
use bijux_planner_fastq::stage_api::bench_dir_name;
use bijux_planner_fastq::stage_api::fastq::trim::plan;
use bijux_planner_fastq::stage_api::FastqArtifact;
use bijux_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, RawFailure,
};
use bijux_runner::primitives::build_tool_execution_spec;

use super::super::jobs::bench_jobs;
use super::super::jobs::execute_plans_with_jobs;
use super::super::{write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_TRIM};
use bijux_planner_fastq::scale_tool_spec_for_jobs;
use bijux_planner_fastq::stage_api::{
    adapter_bank_context, contaminant_bank_context, polyx_bank_context, polyx_unsupported_warning,
};

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_trim<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_planner_fastq::stage_api::args::BenchFastqTrimArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqTrimMetrics>> {
    let allow_experimental = std::env::var("BIJUX_EXPERIMENTAL_TOOLS").is_ok();
    let tools = select_trim_tools(&args.tools, allow_experimental)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage(STAGE_TRIM.as_str(), artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings(STAGE_TRIM.as_str(), &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_TRIM.as_str(), &tools, &registry, false)?;

    let bench_dir_name = bench_dir_name(&STAGE_TRIM)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_TRIM.as_str()))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_infra::ensure_dir(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, STAGE_TRIM.as_str(), &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, STAGE_TRIM.as_str(), &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed(STAGE_TRIM.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_TRIM.as_str(), &tools, platform, catalog)?;

    let jobs = bench_jobs(args.jobs);
    let adapter_bank = adapter_bank_context(
        args.adapter_bank_preset.as_deref(),
        args.adapter_bank.as_deref(),
        args.adapter_bank_file.as_deref(),
        &args.enable_adapters,
        &args.disable_adapters,
    )?;
    let polyx_bank = polyx_bank_context(args.polyx_preset.as_deref())?;
    let contaminant_bank = contaminant_bank_context(args.contaminant_preset.as_deref())?;

    let mut failures = Vec::new();
    let mut plans = Vec::new();
    let mut tool_order = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        bijux_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec =
            build_tool_execution_spec(STAGE_TRIM.as_str(), tool, &registry, catalog, platform)?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        if let Some(msg) = polyx_unsupported_warning(
            &tool_spec.tool_id.0,
            polyx_bank.as_ref(),
            args.polyx_preset.is_some(),
        ) {
            eprintln!("{msg}");
        }
        let plan = plan(
            &tool_spec,
            &args.r1,
            &out_dir,
            adapter_bank.as_ref(),
            polyx_bank.as_ref(),
            contaminant_bank.as_ref(),
        )?;
        plans.push(plan);
        tool_order.push(tool.clone());
    }
    let executions = execute_plans_with_jobs(plans, platform.runner, jobs)?;
    for (tool, execution) in tool_order.into_iter().zip(executions.into_iter()) {
        if execution.exit_code != 0 {
            let tool_name = tool.clone();
            failures.push(RawFailure {
                stage: STAGE_TRIM.as_str().to_string(),
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
