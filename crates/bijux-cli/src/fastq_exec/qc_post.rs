use std::collections::HashMap;
use std::fs;

use anyhow::{anyhow, Context, Result};
use bijux_engine::api::{bench_base_dir, bench_tools_dir, PlatformSpec, RunnerKind, ToolImageSpec};
use bijux_engine::api::{ensure_bench_runner, filter_tools_by_role, load_registry};
use bijux_engine::api::{ensure_image_qa_passed, ensure_tool_qa_passed};
use bijux_engine::api::{execute_stage_plan, resolve_image_for_run, StagePlan};
use bijux_stages_fastq::fastq::qc_post::{aux_tool_ids, normalize_qc_post_tool_list, plan_qc_post};
use bijux_stages_fastq::{inspect_headers, log_header_warnings, preflight_stage, FastqArtifact};
use bijux_stages_fastq::{RawFailure, StagePlanJson};

use crate::fastq_exec::helpers::{write_explain_md, write_explain_plan_json, BenchOutcome};

/// Run the FASTQ benchmark stage.
///
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
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.qc_post", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.qc_post", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.qc_post", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.qc_post", &tools, platform, catalog)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let plan = plan_qc_post(tool, &args.r1, &out_dir)?;
        let plan_json = StagePlanJson::from_plan(&plan);
        let mut aux_images = HashMap::new();
        for aux_tool in aux_tool_ids() {
            let image = resolve_image_for_run(
                catalog
                    .get(*aux_tool)
                    .ok_or_else(|| anyhow!("tool {aux_tool} missing from images.yaml"))?,
                platform,
            )?;
            aux_images.insert((*aux_tool).to_string(), image);
        }
        let exec_plan = StagePlan {
            stage_id: "fastq.qc_post".to_string(),
            tool: tool.to_string(),
            image: resolve_image_for_run(
                catalog
                    .get(tool)
                    .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?,
                platform,
            )?,
            runner: platform.runner,
            inputs: vec![args.r1.clone()],
            out_dir: out_dir.clone(),
            outputs: Vec::new(),
            params: plan_json.parameters,
            aux_images,
        };
        let execution = execute_stage_plan(&exec_plan)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.qc_post".to_string(),
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
