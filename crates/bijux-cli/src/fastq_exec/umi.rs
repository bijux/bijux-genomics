use std::collections::HashMap;
use std::fs;

use anyhow::{anyhow, Context, Result};
use bijux_engine::api::{bench_base_dir, bench_tools_dir, PlatformSpec, RunnerKind, ToolImageSpec};
use bijux_engine::api::{ensure_bench_runner, filter_tools_by_role, load_registry};
use bijux_engine::api::{ensure_image_qa_passed, ensure_tool_qa_passed};
use bijux_engine::api::{execute_stage_plan, resolve_image_for_run, StagePlan};
use bijux_stages_fastq::fastq::umi::{normalize_umi_tool_list, plan_umi};
use bijux_stages_fastq::{
    ensure_umi_headers, inspect_headers, log_header_warnings, preflight_stage,
};
use bijux_stages_fastq::{FastqArtifactKind, RawFailure, StagePlanJson};

use crate::fastq_exec::helpers::{write_explain_md, write_explain_plan_json, BenchOutcome};

/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_umi<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqUmiArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqUmiMetrics>> {
    let tools = normalize_umi_tool_list(&args.tools)?;
    let r2 = args
        .r2
        .as_ref()
        .ok_or_else(|| anyhow!("r2 required for fastq.umi"))?;
    preflight_stage("fastq.umi", FastqArtifactKind::PairedEnd)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    ensure_umi_headers(&args.r1, args.r2.as_deref())?;
    log_header_warnings("fastq.umi", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.umi", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "umi", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "umi", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.umi", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.umi", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.umi", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.umi", &tools, platform, catalog)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let plan = plan_umi(tool, &args.r1, r2, &out_dir)?;
        let plan_json = StagePlanJson::from_plan(&plan);
        let exec_plan = StagePlan {
            stage_id: "fastq.umi".to_string(),
            tool: tool.to_string(),
            image: resolve_image_for_run(
                catalog
                    .get(tool)
                    .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?,
                platform,
            )?,
            runner: platform.runner,
            inputs: vec![args.r1.clone(), r2.clone()],
            out_dir: out_dir.clone(),
            outputs: vec![plan.output_r1.clone(), plan.output_r2.clone()],
            params: plan_json.parameters,
            aux_images: HashMap::new(),
        };
        let execution = execute_stage_plan(&exec_plan)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.umi".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
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
