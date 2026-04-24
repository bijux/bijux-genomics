#![allow(clippy::wildcard_imports)]

use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};

use super::contracts::{write_merge_join_contract, write_stage_resume_contract};
use super::governance::{
    enforce_stage_applicability, write_fastq_output_contract, write_stage_governance_artifacts,
    write_taxonomy_db_drift_report,
};
use crate::internal::fastq::stages::preprocess::*;

mod batch_execution;
mod graph;
mod selection;

use self::batch_execution::execute_preprocess_batch;
use self::graph::{execution_step_batches, terminal_step_ids};
use self::selection::{
    planner_selection_surfaces, preprocess_selection_mode, report_qc_aux_tool_ids,
    PreprocessSelectionMode,
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
    let normalized_sample_id = canonical_sample_identity(&args.sample_id);
    let bench_dir_name = bench_dir_name(&STAGE_PREPROCESS_SUMMARY)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_PREPROCESS_SUMMARY.as_str()))?;
    let out_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&out_dir).context("create preprocess output dir")?;
    let run_root = bijux_dna_runtime::recording::run_artifacts_dir_for_out(&out_dir);
    bijux_dna_infra::ensure_dir(&run_root).context("create preprocess run dir")?;
    let entry_invariants = write_fastq_entry_invariants(&run_root, &args.r1, args.r2.as_deref())?;
    maybe_write_fastq_coverage_classifier(&run_root, &entry_invariants)?;
    let primer_governance = enforce_primer_governance(&run_root, args, &entry_invariants)?;
    write_reference_db_validation_artifact(&run_root, None, primer_governance.as_ref())?;
    write_contamination_controls_report(&run_root, &normalized_sample_id)?;
    write_batch_effect_summary(&run_root, &normalized_sample_id, &entry_invariants)?;
    if args.r2.is_some() && !entry_invariants.paired_consistent {
        return Err(anyhow!(
            "fastq entry invariants failed: {}",
            entry_invariants
                .paired_reason
                .unwrap_or_else(|| "paired-end consistency failed".to_string())
        ));
    }

    ensure_bench_runner(platform, runner_override)?;

    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let decisions = preprocess_decisions(args);
    let pipeline = resolve_preprocess_pipeline(args, &decisions);
    if args.mode == bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::Shotgun {
        let amplicon_only = [
            "fastq.normalize_primers",
            "fastq.remove_chimeras",
            "fastq.infer_asvs",
            "fastq.cluster_otus",
            "fastq.normalize_abundance",
        ];
        if let Some(stage) =
            pipeline.stage_catalog().iter().find(|stage| amplicon_only.contains(&stage.as_str()))
        {
            return Err(anyhow!(
                "stage {stage} is not applicable in shotgun mode; use --mode edna_amplicon or --mode pollen_amplicon"
            ));
        }
    }
    let bench_repo =
        if args.auto { Some(SqliteBenchResultsRepository::new(args.out.clone())) } else { None };
    let jobs = bench_jobs(args.jobs);
    let runtime_pipeline = pipeline.clone();
    let paired_end = args.r2.is_some();
    let mut planner_stage_toolsets = Vec::new();
    let mut selected_stage_tools = match preprocess_selection_mode(args) {
        PreprocessSelectionMode::RunAllGovernedTools => {
            let toolsets = bijux_dna_planner_fastq::select_preprocess_toolsets(
                &runtime_pipeline,
                bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::GovernedExecution,
                args.allow_planned,
            )?;
            let filtered_toolsets = toolsets
                .into_iter()
                .map(|toolset| {
                    let filtered = filter_tools_by_role(
                        &toolset.stage_id,
                        &toolset.tool_ids,
                        &registry,
                        false,
                    )?;
                    let filtered =
                        bijux_dna_planner_fastq::stage_api::filter_tools_for_input_layout(
                            &bijux_dna_core::ids::StageId::new(toolset.stage_id.clone()),
                            filtered.into_iter().map(bijux_dna_core::ids::ToolId::new).collect(),
                            paired_end,
                        )
                        .into_iter()
                        .map(|tool_id| tool_id.to_string())
                        .collect::<Vec<_>>();
                    Ok(bijux_dna_planner_fastq::ToolsetSelection {
                        stage_id: toolset.stage_id,
                        stage_instance_id: toolset.stage_instance_id,
                        tool_ids: filtered,
                        reason: toolset.reason,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            planner_stage_toolsets = filtered_toolsets
                .iter()
                .map(|toolset| {
                    let tools = toolset
                        .tool_ids
                        .iter()
                        .map(|tool_id| {
                            let spec = build_tool_execution_spec(
                                toolset.stage_id.as_str(),
                                tool_id.as_str(),
                                &registry,
                                catalog,
                                platform,
                            )?;
                            Ok(scale_tool_spec_for_jobs(&spec, jobs))
                        })
                        .collect::<Result<Vec<_>>>()?;
                    Ok(bijux_dna_planner_fastq::FastqStageToolsetBinding {
                        stage_id: toolset.stage_id.clone(),
                        stage_instance_id: toolset.stage_instance_id.clone(),
                        tools,
                        reason: Some(toolset.reason.clone()),
                        params: None,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let (_expanded_pipeline, expanded_stage_tools) =
                bijux_dna_planner_fastq::expand_pipeline_stage_tool_routes(
                    &runtime_pipeline,
                    &filtered_toolsets,
                )?;
            expanded_stage_tools
        }
        PreprocessSelectionMode::DefaultChoice => {
            let toolsets = bijux_dna_planner_fastq::select_preprocess_toolsets(
                &runtime_pipeline,
                bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::DefaultChoice,
                args.allow_planned,
            )?;
            let filtered_toolsets = toolsets
                .into_iter()
                .map(|toolset| {
                    let filtered = filter_tools_by_role(
                        &toolset.stage_id,
                        &toolset.tool_ids,
                        &registry,
                        false,
                    )?;
                    let filtered =
                        bijux_dna_planner_fastq::stage_api::filter_tools_for_input_layout(
                            &bijux_dna_core::ids::StageId::new(toolset.stage_id.clone()),
                            filtered.into_iter().map(bijux_dna_core::ids::ToolId::new).collect(),
                            paired_end,
                        )
                        .into_iter()
                        .map(|tool_id| tool_id.to_string())
                        .collect::<Vec<_>>();
                    Ok(bijux_dna_planner_fastq::ToolsetSelection {
                        stage_id: toolset.stage_id,
                        stage_instance_id: toolset.stage_instance_id,
                        tool_ids: filtered,
                        reason: toolset.reason,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            planner_stage_toolsets = filtered_toolsets
                .iter()
                .map(|toolset| {
                    let tools = toolset
                        .tool_ids
                        .iter()
                        .map(|tool_id| {
                            let spec = build_tool_execution_spec(
                                toolset.stage_id.as_str(),
                                tool_id.as_str(),
                                &registry,
                                catalog,
                                platform,
                            )?;
                            Ok(scale_tool_spec_for_jobs(&spec, jobs))
                        })
                        .collect::<Result<Vec<_>>>()?;
                    Ok(bijux_dna_planner_fastq::FastqStageToolsetBinding {
                        stage_id: toolset.stage_id.clone(),
                        stage_instance_id: toolset.stage_instance_id.clone(),
                        tools,
                        reason: Some(toolset.reason.clone()),
                        params: None,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            filtered_toolsets
                .into_iter()
                .map(|toolset| {
                    let tool_id = toolset.tool_ids.into_iter().next().ok_or_else(|| {
                        anyhow!(
                            "default preprocess toolset for {} did not resolve to a runnable tool",
                            toolset.stage_id
                        )
                    })?;
                    Ok(StageToolSelection {
                        stage_id: toolset.stage_id,
                        stage_instance_id: toolset.stage_instance_id,
                        tool_id,
                        reason: toolset.reason,
                    })
                })
                .collect::<Result<Vec<_>>>()?
        }
        PreprocessSelectionMode::AutoSelect => select_preprocess_stage_tools(
            &registry,
            &runtime_pipeline,
            args,
            bench_repo
                .as_ref()
                .map(|repo| repo as &dyn bijux_dna_planner_fastq::BenchResultsRepository),
        )?,
    };
    let mut filtered_stage_tools = Vec::new();
    for selection in &selected_stage_tools {
        let mut allowed = filter_tools_by_role(
            &selection.stage_id,
            std::slice::from_ref(&selection.tool_id),
            &registry,
            false,
        )?;
        if let Some(selected) = allowed.pop() {
            filtered_stage_tools.push(StageToolSelection {
                stage_id: selection.stage_id.clone(),
                stage_instance_id: selection.stage_instance_id.clone(),
                tool_id: selected,
                reason: selection.reason.clone(),
            });
        }
    }
    selected_stage_tools = filtered_stage_tools;
    let tool_ids: Vec<String> =
        selected_stage_tools.iter().map(|selection| selection.tool_id.clone()).collect();

    write_explain_plan_json(
        &out_dir,
        STAGE_PREPROCESS_SUMMARY.as_str(),
        &tool_ids,
        &registry,
        None,
    )?;

    ensure_image_qa_passed(STAGE_PREPROCESS_SUMMARY.as_str(), &tool_ids, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_PREPROCESS_SUMMARY.as_str(), &tool_ids, platform, catalog)?;
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&tools_root).context("create preprocess tools dir")?;

    let policy = apply_preprocess_policy(
        selected_stage_tools
            .iter()
            .map(|selection| StageId::new(selection.stage_id.clone()))
            .collect(),
        selected_stage_tools
            .iter()
            .map(|selection| ToolId::new(selection.tool_id.clone()))
            .collect(),
    );

    let adapter_bank = adapter_bank_context(
        policy.adapter_bank_preset_override.as_deref().or(args.adapter_bank_preset.as_deref()),
        args.adapter_bank.as_deref(),
        args.adapter_bank_file.as_deref(),
        &args.enable_adapters,
        &args.disable_adapters,
    )?;
    let polyx_bank = polyx_bank_context(args.polyx_preset.as_deref())?;
    let contaminant_bank = contaminant_bank_context(args.contaminant_preset.as_deref())?;

    let mut failures = Vec::new();
    let mut tool_specs = Vec::new();
    for selection in &selected_stage_tools {
        let stage = StageId::new(selection.stage_id.clone());
        let tool = ToolId::new(selection.tool_id.clone());
        let spec =
            build_tool_execution_spec(stage.as_str(), tool.as_str(), &registry, catalog, platform)?;
        let spec = scale_tool_spec_for_jobs(&spec, jobs);
        if stage == STAGE_TRIM_READS {
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
    if policy.pipeline_stages.iter().any(|stage| stage == &STAGE_REPORT_QC) {
        for aux_tool in report_qc_aux_tool_ids(&runtime_pipeline, &selected_stage_tools) {
            let spec = catalog
                .get(aux_tool.as_str())
                .ok_or_else(|| anyhow!("tool {aux_tool} missing from images.toml"))?;
            let image = resolve_image_for_run(spec, platform)?;
            aux_tools.insert(
                aux_tool,
                ContainerImageRefV1 { image: image.full_name, digest: spec.digest.clone() },
            );
        }
    }
    let pipeline_id = args.profile.as_deref().unwrap_or("fastq-to-fastq__default__v1").to_string();
    let planner_stage_toolsets =
        planner_selection_surfaces(&selected_stage_tools, &tool_specs, planner_stage_toolsets);
    let planner_config = FastqPlanConfig {
        pipeline_id,
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: args.objective,
        pipeline_spec: Some(runtime_pipeline.clone()),
        stage_bindings: Vec::new(),
        stage_toolsets: planner_stage_toolsets,
        aux_images: aux_tools.clone(),
        adapter_bank: adapter_bank.clone(),
        polyx_bank: polyx_bank.clone(),
        contaminant_bank: contaminant_bank.clone(),
        enable_contaminant_removal: args.enable_contaminant_removal,
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        reference_fasta: args.reference_fasta.clone(),
        out_dir: bench_tools_dir(&args.out, bench_dir_name, &args.sample_id),
        allow_planned: args.allow_planned,
    };
    let pipeline_plan = FastqPlanner::plan(&planner_config)?;
    let planned_stage_batches = execution_step_batches(&pipeline_plan)?;
    let planned_stages =
        planned_stage_batches.iter().flat_map(|batch| batch.iter().cloned()).collect::<Vec<_>>();
    if matches!(
        args.mode,
        bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::EdnaAmplicon
            | bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::PollenAmplicon
    ) && planned_stages.iter().any(|step| {
        step.step_id.as_str().starts_with("bam.") || step.step_id.as_str().starts_with("vcf.")
    }) {
        return Err(anyhow!(
            "amplicon mode refusal: BAM/VCF stages are not schedulable in eDNA/pollen preprocess pipeline"
        ));
    }
    let required_tools = required_fastq_tools()?;
    for planned in &planned_stages {
        let stage_id = planned.step_id.to_string();
        let tool_id = planned
            .command
            .template
            .first()
            .map(String::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                anyhow!("stage `{stage_id}` is missing a declared command entrypoint")
            })?;
        enforce_stage_applicability(planned, args, contaminant_bank.as_ref())?;
        enforce_fastq_backend_allowlist(&stage_id, tool_id)?;
        if !required_tools.contains(tool_id) {
            return Err(anyhow!(
                "tool `{tool_id}` for stage `{stage_id}` is not declared in configs/ci/tools/required_tools.toml"
            ));
        }
        enforce_screen_db_governance(planned)?;
    }
    std::env::set_var("BIJUX_PLANNER_VERSION", bijux_dna_planner_fastq::PLANNER_VERSION);

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
                    outputs: plan.io.outputs.iter().map(|artifact| artifact.path.clone()).collect(),
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
        for terminal in terminal_step_ids(&pipeline_plan) {
            edges.push(ExecutionEdge::new(terminal, report_plan.step_id.clone()));
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
        let decision_trace = serde_json::json!({
            "schema_version": "bijux.decision_trace.v1",
            "stage": STAGE_PREPROCESS_SUMMARY.as_str(),
            "selections": selected_stage_tools
                .iter()
                .map(|selection| serde_json::json!({
                    "stage_id": selection.stage_id,
                    "stage_instance_id": selection.stage_instance_id,
                    "tool_id": selection.tool_id,
                    "reason": selection.reason,
                }))
                .collect::<Vec<_>>(),
            "merge_decision": decisions.merge_decision.as_ref(),
            "correct_decision": decisions.correct_decision.as_ref(),
            "adapter_inference": policy.adapter_inference.as_ref(),
            "stage_skips": &policy.stage_skips,
        });
        bijux_dna_infra::atomic_write_json(&root.join("decision_trace.json"), &decision_trace)
            .context("write decision_trace.json")?;
        let artifact_manifest = serde_json::json!({
            "schema_version": "bijux.plan_artifacts.v1",
            "pipeline_id": pipeline_plan.pipeline_id(),
            "artifacts": planned_stages
                .iter()
                .map(|plan| serde_json::json!({
                    "stage_id": plan.step_id.to_string(),
                    "image": plan.image.image,
                    "outputs": plan
                        .io
                        .outputs
                        .iter()
                        .map(|artifact| serde_json::json!({
                            "name": artifact.name,
                            "kind": artifact.role.as_str(),
                            "path": artifact.path,
                        }))
                        .collect::<Vec<_>>(),
                }))
                .collect::<Vec<_>>(),
        });
        bijux_dna_infra::atomic_write_json(
            &root.join("plan_artifact_manifest.json"),
            &artifact_manifest,
        )
        .context("write plan_artifact_manifest.json")?;
        write_run_manifest(&args.out, &stage_runs, &failures)?;
        write_scientific_provenance(&args.out, &stage_runs)?;
        return Ok(());
    }

    let telemetry = build_telemetry_adapter();
    let mut pipeline_attrs = std::collections::BTreeMap::new();
    pipeline_attrs.insert("sample_id".to_string(), normalized_sample_id.clone());
    pipeline_attrs.insert("pipeline".to_string(), STAGE_PREPROCESS_SUMMARY.as_str().to_string());
    let pipeline_span =
        telemetry.start_pipeline(STAGE_PREPROCESS_SUMMARY.as_str(), &pipeline_attrs);

    let mut stage_runs = Vec::new();
    let mut fail_fast_triggered = false;
    for batch in planned_stage_batches {
        if fail_fast_triggered {
            break;
        }
        let batch_results = execute_preprocess_batch(
            &batch,
            platform.runner,
            jobs,
            &out_dir,
            &normalized_sample_id,
            args,
        )?;
        for (planned, batch_result) in batch.into_iter().zip(batch_results.into_iter()) {
            let stage_id = planned.step_id.to_string();
            let tool = planned.image.image.clone();
            let stage_root = run_artifacts_dir_for_out(&out_dir).join(planned.step_id.as_str());
            let resume_hit = batch_result.stdout == "resumed" && batch_result.command == "resume";
            let mut stage_attrs = std::collections::BTreeMap::new();
            stage_attrs.insert("stage".to_string(), stage_id.clone());
            stage_attrs.insert("tool".to_string(), tool.clone());
            let stage_span = telemetry.start_stage(&stage_id, &stage_attrs);
            capture_tool_version(
                &stage_root,
                planned.command.template.first().map(String::as_str),
            )?;
            write_stage_standardized_metrics(
                &stage_root,
                &stage_id,
                &planned.out_dir,
                &batch_result,
            )?;
            emit_fastq_stage_extra_artifacts(&stage_root, &stage_id, &batch_result)?;
            write_stage_governance_artifacts(&stage_root, &planned, contaminant_bank.as_ref())?;
            enforce_metrics_schema(&stage_root, &stage_id)?;
            write_fastq_output_contract(&stage_root, &planned, &batch_result)?;
            write_stage_resume_contract(&stage_root, &stage_id, &batch_result, resume_hit)?;
            if matches!(
                stage_id.as_str(),
                "fastq.trim_terminal_damage"
                    | "fastq.normalize_primers"
                    | "fastq.remove_chimeras"
                    | "fastq.cluster_otus"
                    | "fastq.infer_asvs"
                    | "fastq.normalize_abundance"
            ) {
                let stage_metrics = materialize_amplicon_stage_outputs(&stage_root, &planned)?;
                enforce_amplicon_qc_thresholds(&stage_root, &stage_id, &stage_metrics)?;
            }
            if stage_id == "fastq.merge_pairs" {
                write_merge_join_contract(
                    &stage_root,
                    &batch_result,
                    entry_invariants.paired_consistent,
                )?;
                enforce_amplicon_merge_determinism(&stage_root, args.mode, &batch_result)?;
            }
            write_retention_report(&stage_root, &planned)?;
            if batch_result.exit_code != 0 {
                let hint =
                    classify_failure_hint(&stage_id, &batch_result.stdout, &batch_result.stderr);
                let hint_path = stage_root.join("common_failure_hint.json");
                bijux_dna_infra::atomic_write_json(
                    &hint_path,
                    &serde_json::json!({
                        "schema_version": "bijux.failure_hint.v1",
                        "stage_id": stage_id,
                        "hint": hint,
                        "exit_code": batch_result.exit_code,
                    }),
                )?;
                if stage_id == "fastq.validate_reads" {
                    return Err(anyhow!(
                        "strict validation failed in fastq.validate_reads; refusing pipeline execution"
                    ));
                }
                failures.push(RawFailure {
                    stage: stage_id,
                    tool: tool.clone(),
                    reason: format!(
                        "tool failed with status {}. hint: {}",
                        batch_result.exit_code, hint
                    ),
                    category: ErrorCategory::ToolError,
                });
                fail_fast_triggered = true;
            }
            stage_span.end();
            stage_runs.push(StageExecutionSummary { plan: planned, result: batch_result });
        }
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
    let report_outputs =
        report_plan.io.outputs.iter().map(|artifact| artifact.path.clone()).collect::<Vec<_>>();
    let report_run_id = stage_runs
        .first()
        .map_or_else(|| "report.aggregate".to_string(), |entry| entry.result.run_id.clone());
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
    stage_runs.push(StageExecutionSummary { plan: report_plan, result: report_result });
    let root = bijux_dna_runtime::recording::run_artifacts_dir_for_out(&out_dir);
    bijux_dna_infra::ensure_dir(&root).context("create run artifacts dir")?;
    write_retry_policy(&root)?;
    write_taxonomy_db_drift_report(&root, contaminant_bank.as_ref())?;
    write_reference_db_validation_artifact(
        &root,
        contaminant_bank.as_ref(),
        primer_governance.as_ref(),
    )?;
    let decision_trace_path = root.join("decision_trace.json");
    let identity_norm = serde_json::json!({
        "schema_version": "bijux.identity_normalization.v1",
        "sample_id_raw": args.sample_id.clone(),
        "sample_id_normalized": normalized_sample_id,
        "lane_id": "L001",
    });
    bijux_dna_infra::atomic_write_json(&root.join("identity_normalization.json"), &identity_norm)
        .context("write identity_normalization.json")?;
    let decision_trace = serde_json::json!({
        "schema_version": "bijux.decision_trace.v1",
        "stage": STAGE_PREPROCESS_SUMMARY.as_str(),
        "merge_decision": decisions.merge_decision.as_ref(),
        "correct_decision": decisions.correct_decision.as_ref(),
        "adapter_inference": policy.adapter_inference.as_ref(),
        "stage_skips": &policy.stage_skips,
    });
    bijux_dna_infra::atomic_write_json(&decision_trace_path, &decision_trace)
        .context("write decision_trace.json")?;
    let steps: Vec<_> = stage_runs.iter().map(|entry| entry.plan.clone()).collect();
    let mut edges = pipeline_plan.edges().to_vec();
    if let Some(report) = steps.last() {
        for terminal in terminal_step_ids(&pipeline_plan) {
            if terminal != report.step_id {
                edges.push(ExecutionEdge::new(terminal, report.step_id.clone()));
            }
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
    write_edna_report_summary(&root, &stage_runs)?;
    if let Some(decision) = decisions.merge_decision.as_ref() {
        let run_id =
            stage_runs.first().map(|entry| entry.result.run_id.clone()).ok_or_else(|| {
                anyhow!("preprocess telemetry requires at least one recorded stage run")
            })?;
        let telemetry_path =
            run_artifacts_dir_for_out(&out_dir).join("telemetry").join("events.jsonl");
        let event = TelemetryEventV1 {
            schema_version: "bijux.telemetry.v1".to_string(),
            run_id,
            stage_id: STAGE_PREPROCESS_SUMMARY.as_str().to_string(),
            tool_id: "planner".to_string(),
            event_name: TelemetryEventName::MergeDecision,
            timestamp: chrono::Utc::now(),
            duration_ms: None,
            status: "ok".to_string(),
            trace_id: "merge-decision".to_string(),
            span_id: "merge-decision".to_string(),
            attrs: attrs_from_json(
                &serde_json::to_value(decision).unwrap_or_else(|_| serde_json::json!({})),
            ),
            failure_code: None,
        };
        let _ = write_telemetry_event(&telemetry_path, &event);
    }
    if !failures.is_empty() {
        return Err(anyhow!("preprocess pipeline failed: {} failures", failures.len()));
    }

    Ok(())
}
#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod pipeline_run_tests {
    use super::{
        enforce_stage_applicability, execution_step_batches, planner_selection_surfaces,
        preprocess_selection_mode, report_qc_aux_tool_ids, terminal_step_ids,
        PreprocessSelectionMode,
    };
    use anyhow::Result;
    use bijux_dna_core::contract::{
        ExecutionEdge, ExecutionGraph, PipelineEdgeSpec, PipelineNodeSpec, PipelineSpec,
        PlanPolicy, StageIO, ToolConstraints,
    };
    use bijux_dna_core::prelude::ToolExecutionSpecV1;
    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRef, ArtifactRole, CommandSpecV1, ContainerImageRefV1, StageId, StepId,
        ToolId,
    };
    use bijux_dna_planner_fastq::stage_api::args::{BenchFastqPreprocessArgs, FastqPlannerMode};
    use bijux_dna_planner_fastq::StageToolSelection;
    use bijux_dna_stage_contract::{PlanDecisionReason, PlanReasonKind};

    fn step(id: &str) -> bijux_dna_core::contract::ExecutionStep {
        bijux_dna_core::contract::ExecutionStep {
            step_id: StepId::new(id.to_string()),
            stage_id: StageId::from_static(bijux_dna_core::id_catalog::FASTQ_TRIM),
            command: CommandSpecV1 { template: vec!["echo".to_string(), id.to_string()] },
            image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
            io: StageIO {
                inputs: vec![ArtifactRef::required(
                    ArtifactId::new("reads_r1".to_string()),
                    std::path::PathBuf::from("reads_R1.fastq"),
                    ArtifactRole::Reads,
                )],
                outputs: vec![ArtifactRef::required(
                    ArtifactId::new("report_json".to_string()),
                    std::path::PathBuf::from(format!("{id}.json")),
                    ArtifactRole::SummaryJson,
                )],
            },
            out_dir: std::path::PathBuf::from(id),
            aux_images: std::collections::BTreeMap::new(),
            expected_artifact_ids: vec![ArtifactId::new("report_json".to_string())],
            metrics_schema_ids: Vec::new(),
        }
    }

    fn tool_spec(tool_id: &str) -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::new(tool_id),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 { image: format!("bijux/{tool_id}:latest"), digest: None },
            command: CommandSpecV1 { template: vec![tool_id.to_string()] },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
        }
    }

    fn preprocess_args() -> BenchFastqPreprocessArgs {
        BenchFastqPreprocessArgs {
            sample_id: "sample".to_string(),
            profile: None,
            r1: std::path::PathBuf::from("reads_R1.fastq.gz"),
            r2: None,
            reference_fasta: None,
            out: std::path::PathBuf::from("out"),
            strict: false,
            auto: false,
            objective: bijux_dna_core::contract::Objective::default(),
            bench_corpus: None,
            allow_partial: false,
            dry_run: false,
            replicates: 1,
            jobs: 1,
            ci_bootstrap: None,
            adapter_bank_preset: None,
            adapter_bank: None,
            adapter_bank_file: None,
            enable_adapters: Vec::new(),
            disable_adapters: Vec::new(),
            polyx_preset: None,
            contaminant_preset: None,
            enable_contaminant_removal: false,
            no_qc_post: false,
            force_merge: false,
            enable_correct: false,
            run_all_governed_tools: false,
            allow_planned: false,
            mode: FastqPlannerMode::Shotgun,
        }
    }

    #[test]
    fn execution_step_batches_group_independent_roots_together() -> Result<()> {
        let graph = ExecutionGraph::new(
            "fastq-to-fastq__runtime_batches__v1",
            "planner.test",
            PlanPolicy::default(),
            vec![step("a"), step("b"), step("c")],
            vec![
                ExecutionEdge::new(StepId::new("a".to_string()), StepId::new("c".to_string())),
                ExecutionEdge::new(StepId::new("b".to_string()), StepId::new("c".to_string())),
            ],
        )?;

        let batches = execution_step_batches(&graph)?;
        assert_eq!(batches.len(), 2);
        assert_eq!(
            batches[0].iter().map(|step| step.step_id.as_str()).collect::<Vec<_>>(),
            vec!["a", "b"]
        );
        assert_eq!(
            batches[1].iter().map(|step| step.step_id.as_str()).collect::<Vec<_>>(),
            vec!["c"]
        );
        Ok(())
    }

    #[test]
    fn terminal_step_ids_ignore_non_terminal_compare_parents() -> Result<()> {
        let graph = ExecutionGraph::new(
            "fastq-to-fastq__terminal_steps__v1",
            "planner.test",
            PlanPolicy::default(),
            vec![step("trim.fastp"), step("trim.cutadapt"), step("trim.compare")],
            vec![
                ExecutionEdge::new(
                    StepId::new("trim.fastp".to_string()),
                    StepId::new("trim.compare".to_string()),
                ),
                ExecutionEdge::new(
                    StepId::new("trim.cutadapt".to_string()),
                    StepId::new("trim.compare".to_string()),
                ),
            ],
        )?;
        let terminals = terminal_step_ids(&graph);
        assert_eq!(terminals.len(), 1);
        assert_eq!(terminals[0].as_str(), "trim.compare");
        Ok(())
    }

    #[test]
    fn execution_step_batches_keep_select_rejoin_after_branch_candidates() -> Result<()> {
        let graph = ExecutionGraph::new(
            "fastq-to-fastq__select_rejoin_batches__v1",
            "planner.test",
            PlanPolicy::default(),
            vec![
                step("trim.fastp"),
                step("trim.cutadapt"),
                step("trim.select"),
                step("filter.selected"),
            ],
            vec![
                ExecutionEdge::new(
                    StepId::new("trim.fastp".to_string()),
                    StepId::new("trim.select".to_string()),
                ),
                ExecutionEdge::new(
                    StepId::new("trim.cutadapt".to_string()),
                    StepId::new("trim.select".to_string()),
                ),
                ExecutionEdge::new(
                    StepId::new("trim.select".to_string()),
                    StepId::new("filter.selected".to_string()),
                ),
            ],
        )?;

        let batches = execution_step_batches(&graph)?;
        assert_eq!(batches.len(), 3);
        assert_eq!(
            batches[0].iter().map(|step| step.step_id.as_str()).collect::<Vec<_>>(),
            vec!["trim.cutadapt", "trim.fastp"]
        );
        assert_eq!(
            batches[1].iter().map(|step| step.step_id.as_str()).collect::<Vec<_>>(),
            vec!["trim.select"]
        );
        assert_eq!(
            batches[2].iter().map(|step| step.step_id.as_str()).collect::<Vec<_>>(),
            vec!["filter.selected"]
        );
        Ok(())
    }

    #[test]
    fn planner_selection_surfaces_build_singleton_toolsets_for_selected_tools() {
        let selected = vec![StageToolSelection {
            stage_id: "fastq.trim_reads".to_string(),
            stage_instance_id: Some("trim.fastp".to_string()),
            tool_id: "fastp".to_string(),
            reason: PlanDecisionReason::new(PlanReasonKind::Default, "governed"),
        }];
        let tool_specs = vec![tool_spec("fastp")];
        let toolsets = planner_selection_surfaces(&selected, &tool_specs, Vec::new());

        assert_eq!(toolsets.len(), 1);
        assert_eq!(toolsets[0].stage_id, "fastq.trim_reads");
        assert_eq!(toolsets[0].stage_instance_id.as_deref(), Some("trim.fastp"));
        assert_eq!(toolsets[0].tools.len(), 1);
        assert_eq!(toolsets[0].tools[0].tool_id.as_str(), "fastp");
    }

    #[test]
    fn planner_selection_surfaces_preserve_existing_graph_toolsets() {
        let toolsets = vec![bijux_dna_planner_fastq::FastqStageToolsetBinding {
            stage_id: "fastq.trim_reads".to_string(),
            stage_instance_id: Some("trim.branch".to_string()),
            tools: vec![tool_spec("fastp")],
            reason: Some(PlanDecisionReason::new(PlanReasonKind::Default, "fanout")),
            params: None,
        }];

        let planned_toolsets = planner_selection_surfaces(&[], &[], toolsets);

        assert_eq!(planned_toolsets.len(), 1);
    }

    #[test]
    #[should_panic(expected = "selected preprocess stage tools and tool specs must stay aligned")]
    fn planner_selection_surfaces_reject_mismatched_selected_tools_and_specs() {
        let selected = vec![StageToolSelection {
            stage_id: "fastq.trim_reads".to_string(),
            stage_instance_id: Some("trim.fastp".to_string()),
            tool_id: "fastp".to_string(),
            reason: PlanDecisionReason::new(PlanReasonKind::Default, "governed"),
        }];

        let _ = planner_selection_surfaces(&selected, &[], Vec::new());
    }

    #[test]
    fn preprocess_selection_mode_prefers_governed_fanout_over_auto() {
        let mut args = preprocess_args();
        args.auto = true;
        args.run_all_governed_tools = true;

        assert_eq!(preprocess_selection_mode(&args), PreprocessSelectionMode::RunAllGovernedTools);
    }

    #[test]
    fn preprocess_selection_mode_uses_auto_when_governed_fanout_is_disabled() {
        let mut args = preprocess_args();
        args.auto = true;

        assert_eq!(preprocess_selection_mode(&args), PreprocessSelectionMode::AutoSelect);
    }

    #[test]
    fn report_qc_aux_tools_follow_selected_upstream_branches() {
        let pipeline = PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.validate_reads".to_string(),
                    stage_instance_id: Some("fastq.validate_reads.validation".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.trim.fastp".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.report_qc".to_string(),
                    stage_instance_id: Some("fastq.report_qc.aggregate".to_string()),
                },
            ],
            vec![
                PipelineEdgeSpec {
                    from: "fastq.validate_reads.validation".to_string(),
                    to: "fastq.report_qc.aggregate".to_string(),
                    from_output_id: None,
                    to_input_id: None,
                },
                PipelineEdgeSpec {
                    from: "fastq.trim_reads.trim.fastp".to_string(),
                    to: "fastq.report_qc.aggregate".to_string(),
                    from_output_id: None,
                    to_input_id: None,
                },
            ],
        );
        let tool_ids = report_qc_aux_tool_ids(
            &pipeline,
            &[
                StageToolSelection {
                    stage_id: "fastq.validate_reads".to_string(),
                    stage_instance_id: Some("fastq.validate_reads.validation".to_string()),
                    tool_id: "fastqvalidator".to_string(),
                    reason: PlanDecisionReason::new(PlanReasonKind::Default, "governed"),
                },
                StageToolSelection {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.trim.fastp".to_string()),
                    tool_id: "fastp".to_string(),
                    reason: PlanDecisionReason::new(PlanReasonKind::Default, "governed"),
                },
                StageToolSelection {
                    stage_id: "fastq.report_qc".to_string(),
                    stage_instance_id: Some("fastq.report_qc.aggregate".to_string()),
                    tool_id: "multiqc".to_string(),
                    reason: PlanDecisionReason::new(PlanReasonKind::Default, "governed"),
                },
            ],
        );

        assert_eq!(tool_ids, vec!["fastp".to_string(), "fastqvalidator".to_string()]);
    }

    #[test]
    fn report_qc_aux_tools_ignore_non_qc_producer_stages() {
        let pipeline = PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.index_reference".to_string(),
                    stage_instance_id: Some("fastq.index_reference.reference".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.report_qc".to_string(),
                    stage_instance_id: Some("fastq.report_qc.aggregate".to_string()),
                },
            ],
            Vec::new(),
        );
        let tool_ids = report_qc_aux_tool_ids(
            &pipeline,
            &[
                StageToolSelection {
                    stage_id: "fastq.index_reference".to_string(),
                    stage_instance_id: Some("fastq.index_reference.reference".to_string()),
                    tool_id: "bwa".to_string(),
                    reason: PlanDecisionReason::new(PlanReasonKind::Default, "governed"),
                },
                StageToolSelection {
                    stage_id: "fastq.report_qc".to_string(),
                    stage_instance_id: Some("fastq.report_qc.aggregate".to_string()),
                    tool_id: "multiqc".to_string(),
                    reason: PlanDecisionReason::new(PlanReasonKind::Default, "governed"),
                },
            ],
        );

        assert!(tool_ids.is_empty());
    }

    #[test]
    fn amplicon_preprocess_allows_correct_errors_stage() {
        let mut args = preprocess_args();
        args.mode = bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::EdnaAmplicon;
        let planned = step("fastq.correct_errors");

        enforce_stage_applicability(&planned, &args, None)
            .expect("amplicon preprocess should admit fastq.correct_errors when planned");
    }

    #[test]
    fn single_end_preprocess_still_rejects_merge_pairs_stage() {
        let args = preprocess_args();
        let planned = step("fastq.merge_pairs");

        let error = enforce_stage_applicability(&planned, &args, None)
            .expect_err("single-end preprocess must still reject paired-only merge");
        assert!(error.to_string().contains("requires paired-end input"));
    }
}
