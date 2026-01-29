use std::collections::HashMap;
use std::fs;

use anyhow::{anyhow, Context, Result};
use bijux_engine::api::{execute_stage_plan, resolve_image_for_run, StagePlan};
use bijux_engine::api::{filter_tools_by_role, load_registry};
use bijux_engine::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use bijux_stages_fastq::fastq::screen::{normalize_screen_tool_list, plan_screen};
use bijux_stages_fastq::{inspect_headers, log_header_warnings, preflight_stage, FastqArtifact};
use bijux_stages_fastq::{RawFailure, StagePlanJson};

/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_screen<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    _runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqScreenArgs,
) -> Result<()> {
    let tools = normalize_screen_tool_list(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage("fastq.screen", artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings("fastq.screen", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.screen", &tools, &registry, false)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = args.out.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let plan = plan_screen(tool, &args.r1, &out_dir)?;
        let plan_json = StagePlanJson::from_plan(&plan);
        let exec_plan = StagePlan {
            stage_id: "fastq.screen".to_string(),
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
            outputs: vec![plan.report.clone()],
            params: plan_json.parameters,
            aux_images: HashMap::new(),
        };
        let execution = execute_stage_plan(&exec_plan)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.screen".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
            });
        }
    }
    if !failures.is_empty() {
        return Err(anyhow!("fastq.screen failures: {}", failures.len()));
    }
    Ok(())
}
