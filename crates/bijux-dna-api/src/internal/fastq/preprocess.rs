use bijux_dna_runtime::{
    attrs_from_json, build_telemetry_adapter, TelemetryEventName, TelemetryEventV1,
};
use std::collections::HashMap;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_registry};
use crate::{execution_kernel, execution_kernel::NetworkPolicy};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench_results_fastq::SqliteBenchResultsRepository;
use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::contract::{ExecutionEdge, ExecutionGraph, ExecutionStep};
use bijux_dna_core::ids::{StageId, ToolId};
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
use crate::internal::handlers::fastq::{STAGE_PREPROCESS, STAGE_QC_POST, STAGE_TRIM};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::stage_api::{
    adapter_bank_context, contaminant_bank_context, polyx_bank_context, polyx_unsupported_warning,
};
use std::io::BufRead;
use std::path::PathBuf;


include!("preprocess/helpers.rs");
include!("preprocess/stage_artifacts.rs");

fn enforce_stage_applicability(
    planned: &ExecutionStep,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqPreprocessArgs,
    contaminant_bank: Option<&serde_json::Value>,
) -> Result<()> {
    let stage = planned.step_id.as_str();
    if stage == "fastq.merge" && args.r2.is_none() {
        return Err(anyhow!(
            "stage fastq.merge requires paired-end input (missing R2)"
        ));
    }
    if stage == "fastq.correct"
        && matches!(
            args.mode,
            bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::EdnaAmplicon
                | bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::PollenAmplicon
        )
    {
        return Err(anyhow!(
            "stage fastq.correct refused for amplicon mode; unsupported library type"
        ));
    }
    if matches!(
        stage,
        "fastq.primer_normalization"
            | "fastq.chimera_detection"
            | "fastq.asv_inference"
            | "fastq.otu_clustering"
            | "fastq.abundance_normalization"
    ) && !matches!(
        args.mode,
        bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::EdnaAmplicon
            | bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::PollenAmplicon
    ) {
        return Err(anyhow!(
            "stage {stage} is only applicable in eDNA/pollen amplicon modes"
        ));
    }
    if stage == "fastq.contaminant_screen" {
        let template = planned.command.template.join(" ");
        if !template.contains("assets/reference/contaminants/") {
            return Err(anyhow!(
                "fastq.contaminant_screen requires contaminant assets under assets/reference/contaminants"
            ));
        }
        if contaminant_bank.is_none() {
            return Err(anyhow!(
                "fastq.contaminant_screen requires contaminant bank context"
            ));
        }
    }
    Ok(())
}

fn write_stage_governance_artifacts(
    stage_root: &std::path::Path,
    planned: &ExecutionStep,
    contaminant_bank: Option<&serde_json::Value>,
) -> Result<()> {
    let stage = planned.step_id.as_str();
    if !matches!(
        stage,
        "fastq.screen" | "fastq.rrna" | "fastq.host_depletion" | "fastq.contaminant_screen"
    ) {
        return Ok(());
    }
    let template = planned.command.template.join(" ");
    let lower = template.to_ascii_lowercase();
    let db_flags_present = [
        " --db ",
        "--database",
        "--index",
        "kraken_db",
        "db_path",
        "--ref",
        "--reference",
    ]
    .iter()
    .any(|needle| lower.contains(needle));
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.governance.v1",
        "stage_id": stage,
        "db_flags_present": db_flags_present,
        "command_template": planned.command.template,
        "contaminant_bank": if stage == "fastq.contaminant_screen" { contaminant_bank.cloned() } else { None::<serde_json::Value> },
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("stage.governance.json"), &payload)
        .context("write stage.governance.json")
}

fn write_fastq_output_contract(
    stage_root: &std::path::Path,
    planned: &ExecutionStep,
    execution: &StageResultV1,
) -> Result<()> {
    let declared_outputs = planned
        .io
        .outputs
        .iter()
        .map(|artifact| {
            serde_json::json!({
                "name": artifact.name,
                "role": artifact.role.as_str(),
                "path": artifact.path,
            })
        })
        .collect::<Vec<_>>();
    let emitted_outputs = execution
        .outputs
        .iter()
        .map(|path| serde_json::json!({ "path": path }))
        .collect::<Vec<_>>();
    let expected_ecological_outputs = match planned.stage_id.as_str() {
        "fastq.primer_normalization" => vec!["primer_orientation_report"],
        "fastq.chimera_detection" => vec!["chimera_metrics_json"],
        "fastq.asv_inference" => vec!["asv_table_tsv", "asv_sequences_fasta"],
        "fastq.otu_clustering" => vec!["otu_table_tsv", "otu_sequences_fasta"],
        "fastq.abundance_normalization" => vec!["normalized_abundance_tsv"],
        _ => Vec::new(),
    };
    let ecological_checksums = planned
        .io
        .outputs
        .iter()
        .filter(|artifact| {
            expected_ecological_outputs
                .iter()
                .any(|name| *name == artifact.name.as_str())
        })
        .map(|artifact| {
            let sha256 = if artifact.path.exists() {
                bijux_dna_infra::hash_file_sha256(&artifact.path).ok()
            } else {
                None
            };
            serde_json::json!({
                "name": artifact.name,
                "path": artifact.path,
                "sha256": sha256
            })
        })
        .collect::<Vec<_>>();
    let contract = serde_json::json!({
        "schema_version": "bijux.fastq.output_contract.v1",
        "stage_id": planned.stage_id,
        "step_id": planned.step_id,
        "declared_outputs": declared_outputs,
        "emitted_outputs": emitted_outputs,
        "expected_ecological_outputs": expected_ecological_outputs,
        "ecological_output_checksums": ecological_checksums,
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("stage.output.contract.json"), &contract)
        .context("write stage output contract")
}

fn write_taxonomy_db_drift_report(
    run_root: &std::path::Path,
    contaminant_bank: Option<&serde_json::Value>,
) -> Result<()> {
    let report_path = run_root.join("taxonomy_db_drift.json");
    let current = contaminant_bank.cloned().unwrap_or_else(|| serde_json::json!({}));
    let lock_path = run_root.join("taxonomy_db.lock.json");
    let previous = if lock_path.exists() {
        let raw = std::fs::read_to_string(&lock_path).unwrap_or_default();
        serde_json::from_str::<serde_json::Value>(&raw).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    let current_hash =
        bijux_dna_core::prelude::params_hash(&current).unwrap_or_else(|_| "unknown".to_string());
    let previous_hash = previous
        .get("current_hash")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let drift_detected = lock_path.exists() && current_hash != previous_hash;
    let report = serde_json::json!({
        "schema_version": "bijux.taxonomy_db_drift.v1",
        "drift_detected": drift_detected,
        "current_hash": current_hash,
        "previous_hash": previous_hash,
        "current": current,
    });
    bijux_dna_infra::atomic_write_json(&report_path, &report).context("write taxonomy_db_drift")?;
    bijux_dna_infra::atomic_write_json(&lock_path, &report).context("write taxonomy_db lock")?;
    Ok(())
}


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
    let normalized_sample_id = normalize_sample_identity(&args.sample_id);
    let bench_dir_name = bench_dir_name(&STAGE_PREPROCESS)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_PREPROCESS.as_str()))?;
    let out_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&out_dir).context("create preprocess output dir")?;

    ensure_bench_runner(platform, runner_override)?;

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let decisions = preprocess_decisions(args);
    let pipeline = resolve_preprocess_pipeline(args, &decisions);
    if args.mode == bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::Shotgun {
        let amplicon_only = [
            "fastq.primer_normalization",
            "fastq.chimera_detection",
            "fastq.asv_inference",
            "fastq.otu_clustering",
            "fastq.abundance_normalization",
        ];
        if let Some(stage) = pipeline
            .stages
            .iter()
            .find(|stage| amplicon_only.contains(&stage.as_str()))
        {
            return Err(anyhow!(
                "stage {stage} is not applicable in shotgun mode; use --mode edna_amplicon or --mode pollen_amplicon"
            ));
        }
    }
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
        pipeline
            .stages
            .iter()
            .map(|stage| StageId::new(stage.clone()))
            .collect(),
        selected_tools
            .iter()
            .map(|selection| ToolId::new(selection.tool_id.clone()))
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
        let spec =
            build_tool_execution_spec(stage.as_str(), tool.as_str(), &registry, catalog, platform)?;
        let spec = scale_tool_spec_for_jobs(&spec, jobs);
        if stage == &STAGE_TRIM {
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
    if policy
        .pipeline_stages
        .iter()
        .any(|stage| stage == &STAGE_QC_POST)
    {
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
    }
    let pipeline_id = args
        .profile
        .as_deref()
        .unwrap_or("fastq-to-fastq__default__v1")
        .to_string();
    let planner_config = FastqPlanConfig {
        pipeline_id,
        policy: PlanPolicy::PreferAccuracy,
        stages: policy
            .pipeline_stages
            .iter()
            .map(|stage| stage.as_str().to_string())
            .collect(),
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
        allow_planned: args.allow_planned,
    };
    let pipeline_plan = FastqPlanner::plan(&planner_config)?;
    let planned_stages = pipeline_plan.steps().to_vec();
    let required_tools = required_fastq_tools()?;
    for planned in &planned_stages {
        let stage_id = planned.step_id.to_string();
        let tool_id = planned
            .command
            .template
            .first()
            .map(String::as_str)
            .unwrap_or_default();
        enforce_stage_applicability(planned, args, contaminant_bank.as_ref())?;
        enforce_fastq_backend_allowlist(&stage_id, tool_id)?;
        if !required_tools.contains(tool_id) {
            return Err(anyhow!(
                "tool `{tool_id}` for stage `{stage_id}` is not declared in configs/ci/tools/required_tools.toml"
            ));
        }
        enforce_screen_db_governance(planned)?;
    }
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
        let decision_trace = serde_json::json!({
            "schema_version": "bijux.decision_trace.v1",
            "stage": STAGE_PREPROCESS.as_str(),
            "selections": selected_tools
                .iter()
                .map(|selection| serde_json::json!({
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
    pipeline_attrs.insert(
        "pipeline".to_string(),
        STAGE_PREPROCESS.as_str().to_string(),
    );
    let pipeline_span = telemetry.start_pipeline(STAGE_PREPROCESS.as_str(), &pipeline_attrs);

    let mut stage_runs = Vec::new();
    let mut fail_fast_triggered = false;
    for planned in &planned_stages {
        let stage_id = planned.step_id.to_string();
        let tool = planned.image.image.clone();
        let mut stage_attrs = std::collections::BTreeMap::new();
        stage_attrs.insert("stage".to_string(), stage_id.clone());
        stage_attrs.insert("tool".to_string(), tool.clone());
        let stage_span = telemetry.start_stage(&stage_id, &stage_attrs);
        let stage_root = run_artifacts_dir_for_out(&out_dir).join(planned.step_id.as_str());
        let invocation = execution_kernel::ToolExec::invoke(&execution_kernel::ToolInvocationRequest {
            step: planned.clone(),
            runner: platform.runner,
            context: execution_kernel::ToolContext {
                run_id: format!("fastq-preprocess-{}", planned.step_id),
                stage_id: planned.step_id.to_string(),
                tool_id: planned.image.image.clone(),
                sample_id: Some(normalized_sample_id.clone()),
                stage_root: stage_root.clone(),
                input_root: args
                    .r1
                    .parent()
                    .map(std::path::Path::to_path_buf)
                    .unwrap_or_else(|| out_dir.clone()),
                output_root: out_dir.clone(),
                tmp_root: stage_root.join("tmp"),
                threads: 1,
                memory_hint_mb: None,
                seed: None,
                network_policy: stage_network_policy(&stage_id),
            },
            timeout: None,
            mode: execution_kernel::ToolExecMode::Execute,
        });
        stage_span.end();
        let execution = invocation?.stage_result;
        write_stage_standardized_metrics(&stage_root, &stage_id, &planned.out_dir, &execution)?;
        emit_fastq_stage_extra_artifacts(&stage_root, &stage_id, &execution)?;
        write_stage_governance_artifacts(&stage_root, planned, contaminant_bank.as_ref())?;
        enforce_metrics_schema(&stage_root, &stage_id)?;
        write_fastq_output_contract(&stage_root, planned, &execution)?;
        write_retention_report(&stage_root, planned)?;
        if execution.exit_code != 0 {
            let hint = classify_failure_hint(&stage_id, &execution.stdout, &execution.stderr);
            let hint_path = stage_root.join("common_failure_hint.json");
            bijux_dna_infra::atomic_write_json(
                &hint_path,
                &serde_json::json!({
                    "schema_version": "bijux.failure_hint.v1",
                    "stage_id": stage_id,
                    "hint": hint,
                    "exit_code": execution.exit_code,
                }),
            )?;
            if stage_id == "fastq.validate_pre" {
                return Err(anyhow!(
                    "strict validation failed in fastq.validate_pre; refusing pipeline execution"
                ));
            }
            failures.push(RawFailure {
                stage: stage_id,
                tool: tool.clone(),
                reason: format!("tool failed with status {}. hint: {}", execution.exit_code, hint),
                category: ErrorCategory::ToolError,
            });
            fail_fast_triggered = true;
        }
        stage_runs.push(StageExecutionSummary {
            plan: planned.clone(),
            result: execution,
        });
        if fail_fast_triggered {
            break;
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
    write_retry_policy(&root)?;
    write_taxonomy_db_drift_report(&root, contaminant_bank.as_ref())?;
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
        "stage": STAGE_PREPROCESS.as_str(),
        "merge_decision": decisions.merge_decision.as_ref(),
        "correct_decision": decisions.correct_decision.as_ref(),
        "adapter_inference": policy.adapter_inference.as_ref(),
        "stage_skips": &policy.stage_skips,
    });
    bijux_dna_infra::atomic_write_json(&decision_trace_path, &decision_trace)
        .context("write decision_trace.json")?;
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
    if let Some(decision) = decisions.merge_decision.as_ref() {
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
        return Err(anyhow!(
            "preprocess pipeline failed: {} failures",
            failures.len()
        ));
    }

    Ok(())
}
