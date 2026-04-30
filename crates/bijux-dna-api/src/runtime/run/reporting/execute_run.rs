use super::evidence_support::materialize_governed_evidence;
use super::planner_manifest_support::plan_manifest_from_request;
use super::{summary_artifact, Result};
use crate::request_args::{ExecuteRequest, ExecuteResponse, PlanRequest};
use anyhow::{anyhow, Context};
use bijux_dna_engine::Engine;
use bijux_dna_environment::api::RuntimeKind;
use bijux_dna_runner::{DockerRunner, LocalRunner};
use bijux_dna_runtime::run_layout::{
    CancellationPolicyV1, CheckpointPolicyV1, ExecutorDescriptorV1, RunCheckpointV1,
    RunExecutionModeV1, RunExecutorDescriptorV1, RunFailureV1, RunLifecycleStateV1, RunStateTransitionV1,
    RunStateV1, RuntimePolicyV1,
};
use bijux_dna_runtime::{ensure_stage_supported_by_runner, RunnerContractKind};

/// # Errors
/// Returns an error if execution fails.
pub fn execute(request: &ExecuteRequest) -> Result<ExecuteResponse> {
    let runner_contract = runner_contract(request.runner);
    for step in request.graph.steps() {
        ensure_stage_supported_by_runner(runner_contract, step.stage_id.as_str())?;
    }

    let (run_id, layout) = bijux_dna_runtime::run_layout::create_run_layout(&request.run_dir)?;
    let graph_hash = request.graph.hash()?;
    let correlation_id = format!("{}:{run_id}", request.mode);
    let now = bijux_dna_runtime::run_layout::now_string();
    let first_stage_id = request.graph.steps().first().map(|step| step.stage_id.to_string());
    let all_stage_ids: Vec<String> =
        request.graph.steps().iter().map(|step| step.stage_id.to_string()).collect();

    write_graph_and_plan_manifest(request, &layout)?;
    let executor_descriptor = executor_descriptor(&run_id, request.mode, request.runner);
    let runtime_policy = runtime_policy(&run_id, request.mode, &request.graph);
    let mut transitions = vec![
        transition(None, RunLifecycleStateV1::Planned, "run request accepted"),
        transition(
            Some(RunLifecycleStateV1::Planned),
            RunLifecycleStateV1::Prepared,
            "run layout and contracts materialized",
        ),
    ];
    let prepared_checkpoint = RunCheckpointV1 {
        schema_version: "bijux.run_checkpoint.v1".to_string(),
        run_id: run_id.clone(),
        mode: request.mode,
        updated_at: now.clone(),
        completed_stage_ids: Vec::new(),
        pending_stage_ids: all_stage_ids.clone(),
        next_stage_id: first_stage_id.clone(),
    };

    bijux_dna_runtime::run_layout::write_executor_descriptor(&layout, &executor_descriptor)?;
    bijux_dna_runtime::run_layout::write_runtime_policy(&layout, &runtime_policy)?;
    bijux_dna_runtime::run_layout::write_checkpoint(&layout, &prepared_checkpoint)?;
    write_run_state(
        &layout,
        &run_id,
        request.mode,
        RunLifecycleStateV1::Prepared,
        transitions.clone(),
        None,
    )?;

    let (state, checkpoint, failure_path) = match request.mode {
        RunExecutionModeV1::DryRun => {
            return Err(anyhow!("dry_run requests must use the dry_run endpoint"));
        }
        RunExecutionModeV1::Simulation | RunExecutionModeV1::Advisory => {
            transitions.push(transition(
                Some(RunLifecycleStateV1::Prepared),
                RunLifecycleStateV1::Succeeded,
                "execution intentionally skipped for non-enforced mode",
            ));
            (
                RunLifecycleStateV1::Succeeded,
                prepared_checkpoint,
                None,
            )
        }
        RunExecutionModeV1::Enforced => {
            transitions.push(transition(
                Some(RunLifecycleStateV1::Prepared),
                RunLifecycleStateV1::Running,
                "runner execution started",
            ));
            write_run_state(
                &layout,
                &run_id,
                request.mode,
                RunLifecycleStateV1::Running,
                transitions.clone(),
                None,
            )?;
            let runner = build_runner(request.runner)?;
            match Engine::default().execute(&request.graph, runner.as_ref(), &layout, None, None) {
                Ok(record) => {
                    let completed_stage_ids = record
                        .stages
                        .iter()
                        .filter(|entry| entry.success)
                        .map(|entry| entry.stage_id.clone())
                        .collect::<Vec<_>>();
                    transitions.push(transition(
                        Some(RunLifecycleStateV1::Running),
                        RunLifecycleStateV1::Succeeded,
                        "runner execution completed successfully",
                    ));
                    (
                        RunLifecycleStateV1::Succeeded,
                        RunCheckpointV1 {
                            schema_version: "bijux.run_checkpoint.v1".to_string(),
                            run_id: run_id.clone(),
                            mode: request.mode,
                            updated_at: bijux_dna_runtime::run_layout::now_string(),
                            completed_stage_ids,
                            pending_stage_ids: Vec::new(),
                            next_stage_id: None,
                        },
                        None,
                    )
                }
                Err(err) => {
                    let failure = failure_record(&run_id, request.mode, &err.to_string());
                    bijux_dna_runtime::run_layout::write_failure_record(&layout, &failure)?;
                    transitions.push(transition(
                        Some(RunLifecycleStateV1::Running),
                        RunLifecycleStateV1::Failed,
                        failure.message.clone(),
                    ));
                    let failed_checkpoint = RunCheckpointV1 {
                        schema_version: "bijux.run_checkpoint.v1".to_string(),
                        run_id: run_id.clone(),
                        mode: request.mode,
                        updated_at: bijux_dna_runtime::run_layout::now_string(),
                        completed_stage_ids: Vec::new(),
                        pending_stage_ids: all_stage_ids.clone(),
                        next_stage_id: first_stage_id.clone(),
                    };
                    bijux_dna_runtime::run_layout::write_checkpoint(&layout, &failed_checkpoint)?;
                    write_run_state(
                        &layout,
                        &run_id,
                        request.mode,
                        RunLifecycleStateV1::Failed,
                        transitions,
                        Some(layout.failure_path.clone()),
                    )?;
                    write_manifest(
                        &layout,
                        &request.graph,
                        &run_id,
                        &correlation_id,
                        request.mode,
                        RunLifecycleStateV1::Failed,
                        &graph_hash,
                        Some(&failure),
                    )?;
                    let governed = materialize_governed_evidence(
                        &layout,
                        &request.graph,
                        &run_id,
                        request.mode,
                        RunLifecycleStateV1::Failed,
                        &run_id,
                        Vec::new(),
                        vec![bijux_dna_runtime::run_layout::CacheDecisionV1 {
                            stage_id: failure.step_id.clone().unwrap_or_else(|| "runtime".to_string()),
                            status: "miss".to_string(),
                            cache_key: None,
                            reason_code: Some("failed_execution_cannot_be_reused".to_string()),
                            message: "failed executions are recorded as unsafe cache misses".to_string(),
                        }],
                        Vec::new(),
                    )?;
                    for (name, schema, path) in [
                        (
                            "artifact_inventory",
                            "bijux.artifact_inventory.v1",
                            governed.artifact_inventory_path.as_path(),
                        ),
                        (
                            "artifact_inventory_text",
                            "bijux.artifact_inventory_text.v1",
                            governed.artifact_inventory_text_path.as_path(),
                        ),
                        (
                            "replay_manifest",
                            "bijux.replay_manifest.v1",
                            governed.replay_manifest_path.as_path(),
                        ),
                        ("hash_ledger", "bijux.hash_ledger.v1", governed.hash_ledger_path.as_path()),
                        (
                            "run_summary_text",
                            "bijux.run_summary_text.v1",
                            governed.run_summary_text_path.as_path(),
                        ),
                    ] {
                        summary_artifact::attach_output_artifact(
                            &layout.manifest_path,
                            &layout.run_dir,
                            &correlation_id,
                            name,
                            schema,
                            path,
                        )?;
                    }
                    let evidence_bundle_path =
                        bijux_dna_analyze::write_evidence_bundle_json(&layout.run_dir, None)?;
                    summary_artifact::attach_output_artifact(
                        &layout.manifest_path,
                        &layout.run_dir,
                        &correlation_id,
                        "evidence_bundle",
                        "bijux.evidence_bundle.v1",
                        &evidence_bundle_path,
                    )?;
                    let evidence_verification =
                        bijux_dna_analyze::verify_evidence_bundle(&evidence_bundle_path)?;
                    bijux_dna_infra::atomic_write_json(
                        &layout.evidence_verification_path,
                        &evidence_verification,
                    )?;
                    summary_artifact::attach_output_artifact(
                        &layout.manifest_path,
                        &layout.run_dir,
                        &correlation_id,
                        "evidence_verification",
                        "bijux.evidence_verification.v1",
                        &layout.evidence_verification_path,
                    )?;
                    bijux_dna_runtime::recording::write_profile_and_lock_manifests(
                        &layout.manifest_path,
                    )?;
                    return Ok(ExecuteResponse {
                        run_id,
                        correlation_id,
                        manifest_path: layout.manifest_path,
                        run_state_path: layout.run_state_path,
                        runtime_policy_path: layout.runtime_policy_path,
                        executor_descriptor_path: layout.executor_descriptor_path,
                        checkpoint_path: layout.checkpoint_path,
                        failure_path: Some(layout.failure_path),
                        mode: request.mode,
                        state: RunLifecycleStateV1::Failed,
                        report_path: None,
                        evidence_bundle_path,
                        evidence_verification_path: layout.evidence_verification_path,
                        artifact_inventory_path: layout.artifact_inventory_path,
                        replay_manifest_path: layout.replay_manifest_path,
                        hash_ledger_path: layout.hash_ledger_path,
                        run_summary_text_path: layout.run_summary_text_path,
                    });
                }
            }
        }
    };

    bijux_dna_runtime::run_layout::write_checkpoint(&layout, &checkpoint)?;
    write_run_state(&layout, &run_id, request.mode, state, transitions, failure_path.clone())?;
    summary_artifact::write_run_summary_artifact(
        &layout.run_summary_path,
        &request.mode.to_string(),
        request.graph.pipeline_id().as_str(),
        &layout.manifest_path,
    )?;
    write_manifest(
        &layout,
        &request.graph,
        &run_id,
        &correlation_id,
        request.mode,
        state,
        &graph_hash,
        None,
    )?;
    let governed = materialize_governed_evidence(
        &layout,
        &request.graph,
        &run_id,
        request.mode,
        state,
        &run_id,
        Vec::new(),
        vec![bijux_dna_runtime::run_layout::CacheDecisionV1 {
            stage_id: "runtime".to_string(),
            status: if matches!(request.mode, RunExecutionModeV1::Enforced) {
                "miss".to_string()
            } else {
                "advisory".to_string()
            },
            cache_key: None,
            reason_code: Some(match request.mode {
                RunExecutionModeV1::DryRun => "dry_run_uses_dedicated_endpoint",
                RunExecutionModeV1::Simulation => "simulation_skips_cache_reuse",
                RunExecutionModeV1::Advisory => "advisory_skips_cache_reuse",
                RunExecutionModeV1::Enforced => "runner_execution_materialized_fresh_outputs",
            }
            .to_string()),
            message: "runtime cache decisions are recorded explicitly for governed replay".to_string(),
        }],
        Vec::new(),
    )?;
    for (name, schema, path) in [
        (
            "artifact_inventory",
            "bijux.artifact_inventory.v1",
            governed.artifact_inventory_path.as_path(),
        ),
        (
            "artifact_inventory_text",
            "bijux.artifact_inventory_text.v1",
            governed.artifact_inventory_text_path.as_path(),
        ),
        (
            "replay_manifest",
            "bijux.replay_manifest.v1",
            governed.replay_manifest_path.as_path(),
        ),
        ("hash_ledger", "bijux.hash_ledger.v1", governed.hash_ledger_path.as_path()),
        (
            "run_summary_text",
            "bijux.run_summary_text.v1",
            governed.run_summary_text_path.as_path(),
        ),
    ] {
        summary_artifact::attach_output_artifact(
            &layout.manifest_path,
            &layout.run_dir,
            &correlation_id,
            name,
            schema,
            path,
        )?;
    }
    let evidence_bundle_path = bijux_dna_analyze::write_evidence_bundle_json(&layout.run_dir, None)?;
    summary_artifact::attach_output_artifact(
        &layout.manifest_path,
        &layout.run_dir,
        &correlation_id,
        "evidence_bundle",
        "bijux.evidence_bundle.v1",
        &evidence_bundle_path,
    )?;
    let evidence_verification = bijux_dna_analyze::verify_evidence_bundle(&evidence_bundle_path)?;
    bijux_dna_infra::atomic_write_json(&layout.evidence_verification_path, &evidence_verification)?;
    summary_artifact::attach_output_artifact(
        &layout.manifest_path,
        &layout.run_dir,
        &correlation_id,
        "evidence_verification",
        "bijux.evidence_verification.v1",
        &layout.evidence_verification_path,
    )?;
    bijux_dna_runtime::recording::write_profile_and_lock_manifests(&layout.manifest_path)?;
    Ok(ExecuteResponse {
        run_id,
        correlation_id,
        manifest_path: layout.manifest_path,
        run_state_path: layout.run_state_path,
        runtime_policy_path: layout.runtime_policy_path,
        executor_descriptor_path: layout.executor_descriptor_path,
        checkpoint_path: layout.checkpoint_path,
        failure_path,
        mode: request.mode,
        state,
        report_path: None,
        evidence_bundle_path,
        evidence_verification_path: layout.evidence_verification_path,
        artifact_inventory_path: layout.artifact_inventory_path,
        replay_manifest_path: layout.replay_manifest_path,
        hash_ledger_path: layout.hash_ledger_path,
        run_summary_text_path: layout.run_summary_text_path,
    })
}

fn runner_contract(runner: RuntimeKind) -> RunnerContractKind {
    match runner {
        RuntimeKind::Local => RunnerContractKind::Local,
        RuntimeKind::Docker => RunnerContractKind::Docker,
        RuntimeKind::Apptainer | RuntimeKind::Singularity => RunnerContractKind::Apptainer,
    }
}

fn build_runner(runner: RuntimeKind) -> Result<Box<dyn bijux_dna_runtime::Runner>> {
    match runner {
        RuntimeKind::Local => Ok(Box::new(LocalRunner::new(None))),
        RuntimeKind::Docker => Ok(Box::new(DockerRunner::new(None))),
        RuntimeKind::Apptainer | RuntimeKind::Singularity => {
            Err(anyhow!("runner {runner} is not yet supported for enforced execute"))
        }
    }
}

fn write_graph_and_plan_manifest(
    request: &ExecuteRequest,
    layout: &bijux_dna_runtime::run_layout::RunLayout,
) -> Result<()> {
    let graph_payload =
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&request.graph)?;
    bijux_dna_infra::atomic_write_bytes(&layout.graph_path, graph_payload.as_slice())?;
    let plan_request = PlanRequest {
        graph: request.graph.clone(),
        profile_id: request.graph.pipeline_id().to_string(),
        workflow_manifest: None,
        stage_plans: Vec::new(),
        parameter_traces: Vec::new(),
        planner_refusals: Vec::new(),
        planner_warnings: Vec::new(),
        compare_against: None,
    };
    let plan_manifest = plan_manifest_from_request(&plan_request)?;
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&plan_manifest)?;
    bijux_dna_infra::atomic_write_bytes(&layout.plan_manifest_path, payload.as_slice())?;
    Ok(())
}

fn executor_descriptor(
    run_id: &str,
    mode: RunExecutionModeV1,
    runner: RuntimeKind,
) -> RunExecutorDescriptorV1 {
    let descriptor = match runner {
        RuntimeKind::Local => ExecutorDescriptorV1::Local {
            runtime: "local".to_string(),
            execution_model: "host_process".to_string(),
            working_directory_policy: "stage_out_dir".to_string(),
            image_policy: "declared_for_provenance_only".to_string(),
        },
        RuntimeKind::Docker => ExecutorDescriptorV1::Container {
            runtime: "docker".to_string(),
            image_resolution_policy: "resolved_before_execute".to_string(),
            bind_mount_policy: "readonly_inputs_writable_output".to_string(),
        },
        RuntimeKind::Apptainer | RuntimeKind::Singularity => ExecutorDescriptorV1::Container {
            runtime: runner.to_string(),
            image_resolution_policy: "resolved_before_execute".to_string(),
            bind_mount_policy: "readonly_inputs_writable_output".to_string(),
        },
    };
    RunExecutorDescriptorV1 {
        schema_version: "bijux.run_executor_descriptor.v1".to_string(),
        run_id: run_id.to_string(),
        mode,
        descriptor,
    }
}

fn runtime_policy(
    run_id: &str,
    mode: RunExecutionModeV1,
    graph: &bijux_dna_core::contract::ExecutionGraph,
) -> RuntimePolicyV1 {
    RuntimePolicyV1 {
        schema_version: "bijux.runtime_policy.v1".to_string(),
        run_id: run_id.to_string(),
        mode,
        deterministic_scheduler: graph.deterministic_scheduler(),
        retry_policy: graph.retry_policy().clone(),
        step_timeout_s: graph.step_timeout_s(),
        cancellation: CancellationPolicyV1 {
            supports_external_cancellation: false,
            checkpoint_before_cancel: true,
        },
        checkpoint: CheckpointPolicyV1 {
            strategy: "stage_boundary".to_string(),
            granularity: "completed_stage_set".to_string(),
            resume_from_latest_completed_stage: true,
        },
    }
}

fn write_run_state(
    layout: &bijux_dna_runtime::run_layout::RunLayout,
    run_id: &str,
    mode: RunExecutionModeV1,
    state: RunLifecycleStateV1,
    transitions: Vec<RunStateTransitionV1>,
    failure_path: Option<std::path::PathBuf>,
) -> Result<()> {
    bijux_dna_runtime::run_layout::write_run_state(
        layout,
        &RunStateV1 {
            schema_version: "bijux.run_state.v1".to_string(),
            run_id: run_id.to_string(),
            mode,
            state,
            transitions,
            manifest_path: Some(layout.manifest_path.clone()),
            checkpoint_path: Some(layout.checkpoint_path.clone()),
            failure_path,
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn write_manifest(
    layout: &bijux_dna_runtime::run_layout::RunLayout,
    graph: &bijux_dna_core::contract::ExecutionGraph,
    run_id: &str,
    correlation_id: &str,
    mode: RunExecutionModeV1,
    state: RunLifecycleStateV1,
    graph_hash: &str,
    failure: Option<&RunFailureV1>,
) -> Result<()> {
    let summary_exists = layout.run_summary_path.exists();
    let mut artifacts = vec![
        artifact_entry(&layout.run_dir, "graph", "bijux.execution_graph.v1", &layout.graph_path)?,
        artifact_entry(
            &layout.run_dir,
            "plan_manifest",
            "bijux.plan_manifest.v1",
            &layout.plan_manifest_path,
        )?,
        artifact_entry(
            &layout.run_dir,
            "run_state",
            "bijux.run_state.v1",
            &layout.run_state_path,
        )?,
        artifact_entry(
            &layout.run_dir,
            "runtime_policy",
            "bijux.runtime_policy.v1",
            &layout.runtime_policy_path,
        )?,
        artifact_entry(
            &layout.run_dir,
            "executor_descriptor",
            "bijux.run_executor_descriptor.v1",
            &layout.executor_descriptor_path,
        )?,
        artifact_entry(
            &layout.run_dir,
            "checkpoint",
            "bijux.run_checkpoint.v1",
            &layout.checkpoint_path,
        )?,
    ];
    if summary_exists {
        artifacts.push(artifact_entry(
            &layout.run_dir,
            "run_summary",
            "bijux.run_summary.v1",
            &layout.run_summary_path,
        )?);
    }
    if layout.failure_path.exists() {
        artifacts.push(artifact_entry(
            &layout.run_dir,
            "run_failure",
            "bijux.run_failure.v1",
            &layout.failure_path,
        )?);
    }
    artifacts.push(serde_json::json!({
        "name": "run_manifest",
        "kind": "run_manifest",
        "schema": "bijux.run_manifest.v3",
        "path": summary_artifact::relative_path_string(&layout.run_dir, &layout.manifest_path),
        "sha256": serde_json::Value::Null,
    }));
    let manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v3",
        "contract_version": bijux_dna_core::contract::ContractVersion::v1(),
        "run_id": run_id,
        "correlation_id": correlation_id,
        "mode": mode,
        "state": state,
        "pipeline_id": graph.pipeline_id().to_string(),
        "profile_id": graph.pipeline_id().to_string(),
        "graph_hash": graph_hash,
        "cache_key": serde_json::Value::Null,
        "toolchain_versions": [],
        "dataset_fingerprints": [],
        "tool_invocations": [],
        "output_artifacts": artifacts,
        "stages": summary_artifact::planned_stage_manifest(graph),
        "failures": failure.into_iter().map(|entry| serde_json::json!({
            "failure_code": entry.failure_code,
            "message": entry.message,
            "path": summary_artifact::relative_path_string(&layout.run_dir, &layout.failure_path),
            "retryable": entry.retryable,
        })).collect::<Vec<_>>(),
    });
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)?;
    bijux_dna_infra::atomic_write_bytes(&layout.manifest_path, payload.as_slice())?;
    Ok(())
}

fn artifact_entry(
    base_dir: &std::path::Path,
    name: &str,
    schema: &str,
    path: &std::path::Path,
) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "name": name,
        "kind": name,
        "schema": schema,
        "path": summary_artifact::relative_path_string(base_dir, path),
        "sha256": bijux_dna_infra::hash_file_sha256(path)
            .with_context(|| format!("hash artifact {}", path.display()))?,
    }))
}

fn transition(
    from_state: Option<RunLifecycleStateV1>,
    to_state: RunLifecycleStateV1,
    detail: impl Into<String>,
) -> RunStateTransitionV1 {
    RunStateTransitionV1 {
        from_state,
        to_state,
        occurred_at: bijux_dna_runtime::run_layout::now_string(),
        detail: Some(detail.into()),
    }
}

fn failure_record(run_id: &str, mode: RunExecutionModeV1, message: &str) -> RunFailureV1 {
    let (failure_code, step_id) = if let Some(step_id) = message.strip_prefix("step failed after retries: ") {
        ("step_failed_after_retries".to_string(), Some(step_id.to_string()))
    } else if let Some(step_id) = message
        .strip_prefix("execution cancelled during ")
        .or_else(|| message.strip_prefix("execution cancelled before "))
    {
        ("execution_cancelled".to_string(), Some(step_id.to_string()))
    } else if message.contains("timeout") {
        ("step_timeout_exceeded".to_string(), None)
    } else {
        ("runner_execution_failed".to_string(), None)
    };
    RunFailureV1 {
        schema_version: "bijux.run_failure.v1".to_string(),
        run_id: run_id.to_string(),
        mode,
        state: if failure_code == "execution_cancelled" {
            RunLifecycleStateV1::Cancelled
        } else {
            RunLifecycleStateV1::Failed
        },
        failure_code,
        message: message.to_string(),
        stage_id: step_id.clone(),
        step_id,
        attempt: None,
        observed_at: bijux_dna_runtime::run_layout::now_string(),
        retryable: message.contains("retry"),
    }
}
