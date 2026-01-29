use std::collections::HashMap;
use std::fs;

use anyhow::{anyhow, Context, Result};
use bijux_engine::api::{bench_base_dir, bench_tools_dir, PlatformSpec, RunnerKind, ToolImageSpec};
use bijux_engine::api::{ensure_bench_runner, filter_tools_by_role, load_registry};
use bijux_engine::api::{ensure_image_qa_passed, ensure_tool_qa_passed};
use bijux_engine::api::{execute_stage_plan, resolve_image_for_run, StagePlan};
use bijux_stages_fastq::fastq::preprocess::{plan_preprocess, plan_preprocess_pipeline};
use bijux_stages_fastq::{bench_corpus, canonical_tool_defaults, RawFailure};

use crate::fastq_exec::helpers::write_explain_plan_json;

/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_preprocess<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    fastq_preprocess_run(catalog, platform, runner_override, args)
}

/// Execute the preprocess pipeline.
///
/// # Errors
/// Returns an error if any stage fails.
pub fn fastq_preprocess_run<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    let out_dir = bench_base_dir(&args.out, "preprocess", &args.sample_id);
    fs::create_dir_all(&out_dir).context("create preprocess output dir")?;

    ensure_bench_runner(platform, runner_override)?;

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let pipeline = plan_preprocess(args).pipeline;
    let mut selected_tools = select_preprocess_tools(&registry, &pipeline, args)?;
    selected_tools = filter_tools_by_role("fastq.preprocess", &selected_tools, &registry, false)?;

    write_explain_plan_json(
        &out_dir,
        "fastq.preprocess",
        &selected_tools,
        &registry,
        None,
    )?;

    ensure_image_qa_passed("fastq.preprocess", &selected_tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.preprocess", &selected_tools, platform, catalog)?;

    let mut failures = Vec::new();
    let planned_stages = plan_preprocess_pipeline(
        &pipeline.stages,
        &selected_tools,
        &args.r1,
        args.r2.as_deref(),
        |stage, tool, _r1, _r2| {
            let stage_dir = stage.trim_start_matches("fastq.");
            let stage_root = bench_tools_dir(&args.out, stage_dir, &args.sample_id);
            let out_dir = stage_root.join(tool);
            fs::create_dir_all(&out_dir).context("create stage output dir")?;
            Ok(out_dir)
        },
    )?;

    for planned in planned_stages {
        let tool = planned.tool.0.clone();
        let image = resolve_image_for_run(
            catalog
                .get(&tool)
                .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?,
            platform,
        )?;
        let mut aux_images = HashMap::new();
        if planned.stage.0 == bijux_stages_fastq::fastq::qc_post::STAGE_ID {
            for aux_tool in bijux_stages_fastq::fastq::qc_post::aux_tool_ids() {
                let spec = catalog
                    .get(*aux_tool)
                    .ok_or_else(|| anyhow!("{aux_tool} missing from images.yaml"))?;
                let image = resolve_image_for_run(spec, platform)?;
                aux_images.insert((*aux_tool).to_string(), image);
            }
        }
        let exec_plan = StagePlan {
            stage_id: planned.stage.0.clone(),
            tool: tool.clone(),
            image,
            runner: platform.runner,
            inputs: planned.inputs.clone(),
            out_dir: planned.out_dir.clone(),
            outputs: planned.outputs.clone(),
            params: planned.plan.parameters.clone(),
            aux_images,
        };
        let execution = execute_stage_plan(&exec_plan)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: planned.stage.0.clone(),
                tool,
                reason: format!("tool failed with status {}", execution.exit_code),
            });
        }
    }

    if !failures.is_empty() {
        return Err(anyhow!(
            "preprocess pipeline failed: {} failures",
            failures.len()
        ));
    }

    Ok(())
}

fn select_preprocess_tools(
    registry: &bijux_core::ToolRegistry,
    pipeline: &bijux_core::domain::PipelineSpec,
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> Result<Vec<String>> {
    let defaults = canonical_tool_defaults();
    let mut selected_tools: Vec<String> = pipeline
        .stages
        .iter()
        .map(|stage| {
            defaults
                .get(stage.as_str())
                .map(|tool| (*tool).to_string())
                .or_else(|| {
                    registry
                        .tools_for_stage(stage)
                        .first()
                        .map(|tool| tool.tool_id.clone())
                })
                .ok_or_else(|| anyhow!("no default tool for stage {stage}"))
        })
        .collect::<Result<_>>()?;

    if args.auto {
        let corpus_id = args
            .bench_corpus
            .ok_or_else(|| anyhow!("--bench-corpus is required with --auto"))?;
        let corpus = bench_corpus(corpus_id);
        let objective = bijux_analyze::selection::objective_spec(args.objective);
        let mut selections = Vec::new();
        for stage in &pipeline.stages {
            let tool_ids: Vec<String> = registry
                .tools_for_stage(stage)
                .iter()
                .map(|tool| tool.tool_id.clone())
                .collect();
            let mut tool_records = Vec::new();
            for tool in &tool_ids {
                let records = bijux_stages_fastq::get_results(stage, tool, &corpus, &args.out)?;
                tool_records.push((tool.clone(), records));
            }
            let selection = bijux_analyze::selection::select_stage(
                stage,
                &tool_records,
                &objective,
                args.allow_partial,
            );
            if selection.selected.is_none() {
                return Err(anyhow!(
                    "no eligible tools for {stage}; check bench corpus/results"
                ));
            }
            selections.push(selection);
        }
        selected_tools = selections
            .into_iter()
            .filter_map(|selection| selection.selected)
            .collect();
    }

    Ok(selected_tools)
}
