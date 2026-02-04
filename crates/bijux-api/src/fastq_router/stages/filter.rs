use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use bijux_engine::primitives::{
    bench_base_dir, bench_tools_dir, build_tool_execution_spec, ensure_bench_runner,
    ensure_image_qa_passed, ensure_tool_qa_passed, filter_tools_by_role, load_registry,
    PlatformSpec, RunnerKind, ToolImageSpec,
};
use bijux_stages_fastq::fastq::filter::{
    normalize_filter_tool_list, plan_filter, FilterPlanOptions,
};
use bijux_stages_fastq::FastqArtifact;
use bijux_stages_fastq::{inspect_headers, log_header_warnings, preflight_stage, RawFailure};

use crate::fastq_router::jobs::execute_plans_with_jobs;
use crate::fastq_router::jobs::{bench_jobs, normalize_tool_spec_for_jobs};
use crate::fastq_router::{write_explain_md, write_explain_plan_json, BenchOutcome};

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_filter<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqFilterArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqFilterMetrics>> {
    let tools = normalize_filter_tool_list(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage("fastq.filter", artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings("fastq.filter", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.filter", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "filter", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "filter", &args.sample_id);
    bijux_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_infra::ensure_dir(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.filter", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.filter", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.filter", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.filter", &tools, platform, catalog)?;

    let jobs = bench_jobs(args.jobs);
    let filter_options = FilterPlanOptions {
        max_n: args.max_n,
        max_n_fraction: None,
        max_n_count: None,
        low_complexity_threshold: args.low_complexity_threshold,
        entropy_threshold: None,
        kmer_ref: args.kmer_ref.clone(),
        redundant_filters: Vec::new(),
        polyx_policy: None,
    };
    let mut failures = Vec::new();
    let mut plans = Vec::new();
    let mut tool_order = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        bijux_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec =
            build_tool_execution_spec("fastq.filter", tool, &registry, catalog, platform)?;
        let tool_spec = normalize_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = plan_filter(&tool_spec, &args.r1, &out_dir, &filter_options)?;
        plans.push(plan);
        tool_order.push(tool.to_string());
    }
    let executions = execute_plans_with_jobs(plans, platform.runner, jobs)?;
    for (tool, execution) in tool_order.into_iter().zip(executions.into_iter()) {
        if execution.exit_code != 0 {
            let tool_name = tool.clone();
            failures.push(RawFailure {
                stage: "fastq.filter".to_string(),
                tool,
                reason: format!(
                    "tool {tool_name} failed with status {}",
                    execution.exit_code
                ),
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
