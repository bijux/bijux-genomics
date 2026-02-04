use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use bijux_core::ErrorCategory;
use bijux_engine::primitives::{
    bench_base_dir, bench_tools_dir, ensure_bench_runner, ensure_image_qa_passed,
    ensure_tool_qa_passed, filter_tools_by_role, load_registry, PlatformSpec, RunnerKind,
    ToolImageSpec,
};
use bijux_runner_docker::primitives::{build_tool_execution_spec, resolve_image_for_run};
use bijux_planner_fastq::normalize_qc_post_tool_list;
use bijux_stages_fastq::fastq::qc_post::{aux_tool_ids, plan_qc_post};
use bijux_stages_fastq::FastqArtifact;
use bijux_stages_fastq::{inspect_headers, log_header_warnings, preflight_stage, RawFailure};

use super::jobs::execute_plans_with_jobs;
use super::jobs::{bench_jobs, normalize_tool_spec_for_jobs};
use super::{write_explain_md, write_explain_plan_json, BenchOutcome};

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_qc_post<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqQcPostArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqQcPostMetrics>> {
    let tools = normalize_qc_post_tool_list(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage("fastq.qc_post", artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings("fastq.qc_post", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.qc_post", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "qc_post", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "qc_post", &args.sample_id);
    bijux_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_infra::ensure_dir(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.qc_post", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.qc_post", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.qc_post", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.qc_post", &tools, platform, catalog)?;

    let jobs = bench_jobs(args.jobs);
    let mut aux_tools = std::collections::BTreeMap::new();
    for aux_tool in aux_tool_ids() {
        let spec = catalog
            .get(*aux_tool)
            .ok_or_else(|| anyhow!("tool {aux_tool} missing from images.toml"))?;
        let image = resolve_image_for_run(spec, platform)?;
        aux_tools.insert(
            (*aux_tool).to_string(),
            bijux_core::ContainerImageRefV1 {
                image: image.full_name,
                digest: spec.digest.clone(),
            },
        );
    }

    let mut failures = Vec::new();
    let mut plans = Vec::new();
    let mut tool_order = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        bijux_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec =
            build_tool_execution_spec("fastq.qc_post", tool, &registry, catalog, platform)?;
        let tool_spec = normalize_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = plan_qc_post(&tool_spec, &args.r1, &out_dir, aux_tools.clone(), None)?;
        plans.push(plan);
        tool_order.push(tool.to_string());
    }
    let executions = execute_plans_with_jobs(plans, platform.runner, jobs)?;
    for (tool, execution) in tool_order.into_iter().zip(executions.into_iter()) {
        if execution.exit_code != 0 {
            let tool_name = tool.clone();
            failures.push(RawFailure {
                stage: "fastq.qc_post".to_string(),
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
