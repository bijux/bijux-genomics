use bijux_dna_runtime::{attrs_from_json, build_telemetry_adapter, TelemetryEventV1};
use std::collections::HashMap;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_registry};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench_results_fastq::SqliteBenchResultsRepository;
use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::contract::{ExecutionEdge, ExecutionGraph};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::ContainerImageRefV1;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::RawFailure;
use bijux_dna_planner_fastq::{
    apply_preprocess_policy, preprocess_decisions, resolve_preprocess_pipeline,
    select_preprocess_tools, FastqPlanConfig, FastqPlanner, ToolSelection,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::backend::docker::executor::resolve_image_for_run;
use bijux_dna_runner::execute::StageResultV1;
use bijux_dna_runtime::recording::run_artifacts_dir_for_out;
use bijux_dna_runtime::recording::write_telemetry_event;

use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::summary::{
    render_run_summary, report_stage_step, write_run_manifest, write_scientific_provenance,
    StageExecutionSummary,
};
use crate::internal::handlers::fastq::write_explain_plan_json;
use crate::internal::handlers::fastq::{STAGE_PREPROCESS, STAGE_TRIM};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::stage_api::{
    adapter_bank_context, contaminant_bank_context, polyx_bank_context, polyx_unsupported_warning,
};

/// Run the preprocess pipeline.
///
/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_preprocess<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqPreprocessArgs,
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
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    let bench_dir_name = bench_dir_name(&STAGE_PREPROCESS)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_PREPROCESS.as_str()))?;
    let out_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&out_dir).context("create preprocess output dir")?;

    ensure_bench_runner(platform, runner_override)?;

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let decisions = preprocess_decisions(args);
    let pipeline = resolve_preprocess_pipeline(args, &decisions);
    let bench_repo = if args.auto {
        Some(SqliteBenchResultsRepository::new(args.out.clone()))
    } else {
        None
    };
    let mut selected_tools = select_preprocess_tools(
        &registry,
        &pipeline,
        args,
        bench_repo
            .as_ref()
            .map(|repo| repo as &dyn bijux_dna_planner_fastq::BenchResultsRepository),
    )?;
    let mut tool_ids: Vec<String> = selected_tools
        .iter()
        .map(|selection| selection.tool_id.clone())
        .collect();
    let mut filtered_by_role = Vec::new();
    for (stage_id, tool_id) in pipeline.stages.iter().zip(tool_ids.iter()) {
        let mut allowed =
            filter_tools_by_role(stage_id, std::slice::from_ref(tool_id), &registry, false)?;
        if let Some(selected) = allowed.pop() {
            filtered_by_role.push(selected);
        }
    }
    tool_ids = filtered_by_role;
    let mut reasons_by_tool = std::collections::HashMap::new();
    for selection in selected_tools.drain(..) {
        reasons_by_tool.insert(selection.tool_id, selection.reason);
    }
    let mut tool_reasons = Vec::new();
    let mut filtered_selections = Vec::new();
    for tool_id in &tool_ids {
        let reason = reasons_by_tool.remove(tool_id).unwrap_or_else(|| {
            bijux_dna_stage_contract::PlanDecisionReason::new(
                bijux_dna_stage_contract::PlanReasonKind::Fallback,
                "selected by role filter",
            )
        });
        tool_reasons.push(reason.clone());
        filtered_selections.push(ToolSelection {
            tool_id: tool_id.clone(),
            reason,
        });
    }
    selected_tools = filtered_selections;

    write_explain_plan_json(
        &out_dir,
        STAGE_PREPROCESS.as_str(),
        &tool_ids,
        &registry,
        None,
    )?;

    ensure_image_qa_passed(STAGE_PREPROCESS.as_str(), &tool_ids, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_PREPROCESS.as_str(), &tool_ids, platform, catalog)?;

    let jobs = bench_jobs(args.jobs);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&tools_root).context("create preprocess tools dir")?;

    let policy = apply_preprocess_policy(
        pipeline.stages.clone(),
        selected_tools
            .iter()
            .map(|selection| selection.tool_id.clone())
            .collect(),
    );

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
        if stage == STAGE_TRIM.as_str() {
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
    for aux_tool in bijux_dna_planner_fastq::stage_api::fastq::qc_post::aux_tool_ids() {
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
    let pipeline_id = args
        .profile
        .as_deref()
        .unwrap_or("fastq-to-fastq__default__v1")
        .to_string();
    let planner_config = FastqPlanConfig {
        pipeline_id,
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
        out_dir: bench_tools_dir(&args.out, bench_dir_name, &args.sample_id),
        tool_reasons: Some(tool_reasons),
    };
    let pipeline_plan = FastqPlanner::plan(&planner_config)?;
    let planned_stages = pipeline_plan.steps().to_vec();
    std::env::set_var(
        "BIJUX_PLANNER_VERSION",
        bijux_dna_planner_fastq::PLANNER_VERSION,
    );

    if args.dry_run {
        let root = bijux_dna_runtime::recording::run_artifacts_dir_for_out(&out_dir);
        bijux_dna_infra::ensure_dir(&root).context("create dry-run artifacts dir")?;
        let mut stage_runs: Vec<StageExecutionSummary> = planned_stages
            .iter()
            .map(|plan| StageExecutionSummary {
                plan: plan.clone(),
                result: StageResultV1 {
                    run_id: "dry-run".to_string(),
                    exit_code: 0,
                    runtime_s: 0.0,
                    memory_mb: 0.0,
                    outputs: plan
                        .io
                        .outputs
                        .iter()
                        .map(|artifact| artifact.path.clone())
                        .collect(),
                    metrics_path: None,
                    stdout: String::new(),
                    stderr: String::new(),
                    command: "dry-run".to_string(),
                },
            })
            .collect();
        let report_plan = report_stage_step(&args.out, &planned_stages);
        let mut steps = planned_stages.clone();
        steps.push(report_plan.clone());
        let mut edges = pipeline_plan.edges().to_vec();
        if let Some(last) = planned_stages.last() {
            edges.push(ExecutionEdge::new(
                last.step_id.clone(),
                report_plan.step_id.clone(),
            ));
        }
        let graph = ExecutionGraph::new(
            pipeline_plan.pipeline_id().to_string(),
            pipeline_plan.planner_version().to_string(),
            pipeline_plan.policy(),
            steps,
            edges,
        )?;
        let graph_path = root.join("graph.json");
        bijux_dna_infra::atomic_write_json(&graph_path, &graph).context("write graph.json")?;
        stage_runs.push(StageExecutionSummary {
            plan: report_plan,
            result: StageResultV1 {
                run_id: "dry-run".to_string(),
                exit_code: 0,
                runtime_s: 0.0,
                memory_mb: 0.0,
                outputs: Vec::new(),
                metrics_path: None,
                stdout: String::new(),
                stderr: String::new(),
                command: "dry-run".to_string(),
            },
        });
        write_run_manifest(&args.out, &stage_runs, &failures)?;
        write_scientific_provenance(&args.out, &stage_runs)?;
        return Ok(());
    }

    let telemetry = build_telemetry_adapter();
    let mut pipeline_attrs = std::collections::BTreeMap::new();
    pipeline_attrs.insert("sample_id".to_string(), args.sample_id.clone());
    pipeline_attrs.insert(
        "pipeline".to_string(),
        STAGE_PREPROCESS.as_str().to_string(),
    );
    let pipeline_span = telemetry.start_pipeline(STAGE_PREPROCESS.as_str(), &pipeline_attrs);

    let mut stage_runs = Vec::new();
    for planned in &planned_stages {
        let stage_id = planned.step_id.to_string();
        let tool = planned.image.image.clone();
        let mut stage_attrs = std::collections::BTreeMap::new();
        stage_attrs.insert("stage".to_string(), stage_id.clone());
        stage_attrs.insert("tool".to_string(), tool.clone());
        let stage_span = telemetry.start_stage(&stage_id, &stage_attrs);
        let execution = bijux_dna_runner::execute::execute_step(planned, platform.runner, None);
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
            plan: planned.clone(),
            result: execution,
        });
    }
    pipeline_span.end();

    render_run_summary(
        &args.out,
        &stage_runs,
        &failures,
        decisions.merge_decision.as_ref(),
        decisions.correct_decision.as_ref(),
        policy.adapter_inference.as_ref(),
        &policy.stage_skips,
    )?;
    let executed_steps: Vec<_> = stage_runs.iter().map(|entry| entry.plan.clone()).collect();
    let report_plan = report_stage_step(&args.out, &executed_steps);
    let report_outputs = report_plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.path.clone())
        .collect::<Vec<_>>();
    let report_run_id = stage_runs.first().map_or_else(
        || "report.aggregate".to_string(),
        |entry| entry.result.run_id.clone(),
    );
    let report_result = StageResultV1 {
        run_id: report_run_id,
        exit_code: 0,
        runtime_s: 0.0,
        memory_mb: 0.0,
        outputs: report_outputs,
        metrics_path: None,
        stdout: String::new(),
        stderr: String::new(),
        command: "report-aggregate".to_string(),
    };
    stage_runs.push(StageExecutionSummary {
        plan: report_plan,
        result: report_result,
    });
    let root = bijux_dna_runtime::recording::run_artifacts_dir_for_out(&out_dir);
    bijux_dna_infra::ensure_dir(&root).context("create run artifacts dir")?;
    let steps: Vec<_> = stage_runs.iter().map(|entry| entry.plan.clone()).collect();
    let mut edges = pipeline_plan.edges().to_vec();
    if let (Some(last), Some(report)) = (planned_stages.last(), steps.last()) {
        if last.step_id != report.step_id {
            edges.push(ExecutionEdge::new(
                last.step_id.clone(),
                report.step_id.clone(),
            ));
        }
    }
    let graph = ExecutionGraph::new(
        pipeline_plan.pipeline_id().to_string(),
        pipeline_plan.planner_version().to_string(),
        pipeline_plan.policy(),
        steps,
        edges,
    )?;
    let graph_path = root.join("graph.json");
    bijux_dna_infra::atomic_write_json(&graph_path, &graph).context("write graph.json")?;
    write_run_manifest(&args.out, &stage_runs, &failures)?;
    write_scientific_provenance(&args.out, &stage_runs)?;
    if let Some(decision) = decisions.merge_decision {
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
            stage_id: STAGE_PREPROCESS.as_str().to_string(),
            tool_id: "planner".to_string(),
            event_name: "merge_decision".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms: None,
            status: "ok".to_string(),
            trace_id: "merge-decision".to_string(),
            span_id: "merge-decision".to_string(),
            attrs: attrs_from_json(
                &serde_json::to_value(decision).unwrap_or_else(|_| serde_json::json!({})),
            ),
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
