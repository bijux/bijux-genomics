use std::collections::HashMap;

use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_registry};
use anyhow::{anyhow, Context, Result};
use bijux_core::execution_plan::PlanPolicy;
use bijux_core::ContainerImageRefV1;
use bijux_core::ErrorCategory;
use bijux_core::TelemetryEventV1;
use bijux_engine::services::run_artifacts::run_artifacts_dir_for_out;
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use bijux_environment::image_qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use bijux_exec::run_artifacts::write_telemetry_event;
use bijux_pipelines::fastq::canonical_tool_defaults;
use bijux_pipelines::registry;
use bijux_pipelines::Domain;
use bijux_planner_fastq::{
    apply_preprocess_policy, default_pipeline_spec, DefaultPipelineOptions, FastqPlanConfig,
    FastqPlanner,
};
use bijux_runner::primitives::{build_tool_execution_spec, resolve_image_for_run};
use bijux_stages_fastq::fastq::preprocess::plan_preprocess;
use bijux_stages_fastq::{bench_corpus, RawFailure};

use super::jobs::bench_jobs;
use super::summary::{write_run_summary, StageExecutionSummary};
use super::write_explain_plan_json;
use bijux_domain_fastq::banks::{
    adapter_bank_context, contaminant_bank_context, polyx_bank_context, polyx_unsupported_warning,
};
use bijux_infra::{bench_base_dir, bench_tools_dir};
use bijux_planner_fastq::scale_tool_spec_for_jobs;

#[must_use]
fn resolve_preprocess_pipeline(
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
    decisions: &bijux_stages_fastq::fastq::preprocess::PreprocessDecisions,
) -> bijux_core::domain::PipelineSpec {
    let enable_merge = decisions.enable_merge;
    let enable_correct = decisions.enable_correct;
    let enable_qc_post = !args.no_qc_post;
    let enable_screen = args.contaminant_preset.is_some();
    if let Some(profile_id) = args.profile.as_deref() {
        match registry::profile_by_id(Domain::Fastq, profile_id) {
            Ok(profile) => {
                let mut stages: Vec<String> = profile
                    .graph
                    .into_iter()
                    .map(|node| node.stage_id)
                    .collect();
                if !enable_merge {
                    stages.retain(|stage| stage != "fastq.merge");
                }
                if !enable_correct {
                    stages.retain(|stage| stage != "fastq.correct");
                }
                if !enable_qc_post {
                    stages.retain(|stage| stage != "fastq.qc_post");
                }
                if !enable_screen {
                    stages.retain(|stage| stage != "fastq.screen");
                }
                bijux_core::domain::PipelineSpec { stages }
            }
            Err(err) => {
                eprintln!("unknown fastq profile {profile_id}: {err}; using default pipeline");
                default_pipeline_spec(DefaultPipelineOptions {
                    paired: args.r2.is_some(),
                    enable_merge,
                    enable_correct,
                    enable_qc_post,
                    enable_screen,
                })
            }
        }
    } else {
        default_pipeline_spec(DefaultPipelineOptions {
            paired: args.r2.is_some(),
            enable_merge,
            enable_correct,
            enable_qc_post,
            enable_screen,
        })
    }
}

/// Build the preprocess pipeline plan.
#[must_use]
pub fn fastq_preprocess_plan(
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> bijux_core::domain::PipelineSpec {
    let decisions = bijux_stages_fastq::fastq::preprocess::preprocess_decisions(args);
    let pipeline = resolve_preprocess_pipeline(args, &decisions);
    plan_preprocess(args, pipeline.clone(), decisions).pipeline
}

/// Run the preprocess pipeline.
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
/// Returns an error if planning or execution fails.
#[allow(clippy::too_many_lines)]
pub fn fastq_preprocess_run<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    let out_dir = bench_base_dir(&args.out, "preprocess", &args.sample_id);
    bijux_infra::ensure_dir(&out_dir).context("create preprocess output dir")?;

    ensure_bench_runner(platform, runner_override)?;

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let decisions = bijux_stages_fastq::fastq::preprocess::preprocess_decisions(args);
    let pipeline = resolve_preprocess_pipeline(args, &decisions);
    let preprocess_plan = plan_preprocess(args, pipeline.clone(), decisions);
    let pipeline = preprocess_plan.pipeline.clone();
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

    let jobs = bench_jobs(args.jobs);
    let tools_root = bench_tools_dir(&args.out, "preprocess", &args.sample_id);
    bijux_infra::ensure_dir(&tools_root).context("create preprocess tools dir")?;

    let policy = apply_preprocess_policy(pipeline.stages.clone(), selected_tools.clone());

    let adapter_bank = adapter_bank_context(
        policy
            .adapter_bank_preset_override
            .as_deref()
            .or(args.adapter_bank_preset.as_deref()),
        args.adapter_bank.as_deref(),
        args.adapter_bank_file.as_deref(),
        &args.enable_adapters,
        &args.disable_adapters,
    )?;
    let polyx_bank = polyx_bank_context(args.polyx_preset.as_deref())?;
    let contaminant_bank = contaminant_bank_context(args.contaminant_preset.as_deref())?;

    let mut failures = Vec::new();
    let mut tool_specs = Vec::new();
    for (stage, tool) in policy
        .pipeline_stages
        .iter()
        .zip(policy.pipeline_tools.iter())
    {
        let spec = build_tool_execution_spec(stage, tool, &registry, catalog, platform)?;
        let spec = scale_tool_spec_for_jobs(&spec, jobs);
        if stage == "fastq.trim" {
            if let Some(msg) = polyx_unsupported_warning(
                &spec.tool_id.0,
                polyx_bank.as_ref(),
                args.polyx_preset.is_some(),
            ) {
                eprintln!("{msg}");
            }
        }
        tool_specs.push(spec);
    }
    let mut aux_tools = std::collections::BTreeMap::new();
    for aux_tool in bijux_stages_fastq::fastq::qc_post::aux_tool_ids() {
        let spec = catalog
            .get(*aux_tool)
            .ok_or_else(|| anyhow!("tool {aux_tool} missing from images.toml"))?;
        let image = resolve_image_for_run(spec, platform)?;
        aux_tools.insert(
            (*aux_tool).to_string(),
            ContainerImageRefV1 {
                image: image.full_name,
                digest: spec.digest.clone(),
            },
        );
    }
    let planner_config = FastqPlanConfig {
        pipeline_id: "fastq.preprocess".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        stages: policy.pipeline_stages.clone(),
        tools: tool_specs.clone(),
        aux_images: aux_tools.clone(),
        adapter_bank: adapter_bank.clone(),
        polyx_bank: polyx_bank.clone(),
        contaminant_bank: contaminant_bank.clone(),
        enable_contaminant_removal: args.enable_contaminant_removal,
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out_dir: bench_tools_dir(&args.out, "preprocess", &args.sample_id),
    };
    let pipeline_plan = FastqPlanner::plan(&planner_config)?;
    let planned_stages = pipeline_plan.stages().to_vec();
    std::env::set_var(
        "BIJUX_PLANNER_VERSION",
        bijux_planner_fastq::PLANNER_VERSION,
    );

    let telemetry = bijux_engine::services::telemetry::build_telemetry_adapter();
    let mut pipeline_attrs = std::collections::BTreeMap::new();
    pipeline_attrs.insert("sample_id".to_string(), args.sample_id.clone());
    pipeline_attrs.insert("pipeline".to_string(), "fastq.preprocess".to_string());
    let pipeline_span = telemetry.start_pipeline("fastq.preprocess", &pipeline_attrs);

    let mut stage_runs = Vec::new();
    for planned in planned_stages {
        let stage_id = planned.stage_id.0.clone();
        let tool = planned.tool_id.0.clone();
        let mut stage_attrs = std::collections::BTreeMap::new();
        stage_attrs.insert("stage".to_string(), stage_id.clone());
        stage_attrs.insert("tool".to_string(), tool.clone());
        let stage_span = telemetry.start_stage(&stage_id, &stage_attrs);
        let execution = bijux_exec::primitives::execute_stage_plan(&planned, platform.runner, None);
        stage_span.end();
        let execution = execution?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: stage_id,
                tool: tool.clone(),
                reason: format!("tool failed with status {}", execution.exit_code),
                category: ErrorCategory::ToolError,
            });
        }
        stage_runs.push(StageExecutionSummary {
            plan: planned,
            result: execution,
        });
    }
    pipeline_span.end();

    write_run_summary(
        &args.out,
        &stage_runs,
        &failures,
        preprocess_plan.merge_decision.as_ref(),
        preprocess_plan.correct_decision.as_ref(),
        policy.adapter_inference.as_ref(),
        &policy.stage_skips,
    )?;
    if let Some(decision) = preprocess_plan.merge_decision {
        let run_id = stage_runs
            .first()
            .map(|entry| entry.result.run_id.clone())
            .unwrap_or_default();
        let telemetry_path = run_artifacts_dir_for_out(&out_dir)
            .join("telemetry")
            .join("events.jsonl");
        let event = TelemetryEventV1 {
            schema_version: "bijux.telemetry.v1".to_string(),
            run_id,
            stage_id: "fastq.preprocess".to_string(),
            tool_id: "planner".to_string(),
            event_name: "merge_decision".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms: None,
            status: "ok".to_string(),
            trace_id: "merge-decision".to_string(),
            span_id: "merge-decision".to_string(),
            attrs: serde_json::to_value(decision).unwrap_or_else(|_| serde_json::json!({})),
        };
        let _ = write_telemetry_event(&telemetry_path, &event);
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
        let objective = bijux_core::selection::objective_spec(args.objective);
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
            let selection = bijux_core::selection::select_stage(
                stage,
                &tool_records,
                &objective,
                args.allow_partial,
            );
            selections.push(selection);
        }
        for (idx, selection) in selections.into_iter().enumerate() {
            if let Some(selected) = selection.selected {
                selected_tools[idx] = selected;
            }
        }
    }

    Ok(selected_tools)
}
