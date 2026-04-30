use super::evidence_support::materialize_governed_evidence;
use super::operations::{
    acquire_run_lease, build_backend_record, build_health_report, build_scheduling_decision,
    default_control_state, initial_queue_state, maybe_mock_slurm_submission, release_run_lease,
};
use super::planner_manifest_support::plan_manifest_from_request;
use super::{summary_artifact, Result};
use crate::request_args::PlanRequest;
use crate::request_args::{DryRunRequest, DryRunResponse};
use bijux_dna_environment::api::RuntimeKind;
use bijux_dna_runtime::run_layout::{
    CancellationPolicyV1, CheckpointPolicyV1, ExecutorDescriptorV1, RunCheckpointV1,
    RunExecutionModeV1, RunExecutorDescriptorV1, RunLifecycleStateV1, RunStateTransitionV1,
    RunStateV1, RuntimePolicyV1, RunEnvironment, ToolImageDigest,
};

/// # Errors
/// Returns an error if dry-run output cannot be written.
pub fn dry_run(request: &DryRunRequest) -> Result<DryRunResponse> {
    bijux_dna_infra::ensure_dir(&request.run_dir)?;
    let layout = bijux_dna_runtime::run_layout::RunLayout::from_run_dir(request.run_dir.clone());
    for dir in [
        &layout.run_dir,
        &layout.manifests_dir,
        &layout.summary_dir,
        &layout.reports_dir,
        &layout.logs_dir,
        &layout.run_artifacts_dir,
        &layout.checkpoints_dir,
        &layout.stages_dir,
    ] {
        bijux_dna_infra::ensure_dir(dir)?;
    }

    let graph_hash = request.graph.hash()?;
    let correlation_id = format!("dry_run:{graph_hash}");
    let graph_payload =
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&request.graph)?;
    bijux_dna_infra::atomic_write_bytes(&layout.graph_path, graph_payload.as_slice())?;

    let plan_request = PlanRequest {
        graph: request.graph.clone(),
        profile_id: request.profile_id.clone(),
        workflow_manifest: None,
        stage_plans: Vec::new(),
        parameter_traces: Vec::new(),
        planner_refusals: Vec::new(),
        planner_warnings: Vec::new(),
        compare_against: None,
    };
    let plan_manifest = plan_manifest_from_request(&plan_request)?;
    let plan_manifest_payload =
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&plan_manifest)?;
    bijux_dna_infra::atomic_write_bytes(&layout.plan_manifest_path, plan_manifest_payload.as_slice())?;

    let run_id = "dry-run".to_string();
    let executor_descriptor = RunExecutorDescriptorV1 {
        schema_version: "bijux.run_executor_descriptor.v1".to_string(),
        run_id: run_id.clone(),
        mode: RunExecutionModeV1::DryRun,
        descriptor: ExecutorDescriptorV1::Local {
            runtime: "local".to_string(),
            execution_model: "no_process_spawn".to_string(),
            working_directory_policy: "governed_run_layout_only".to_string(),
            image_policy: "declared_for_provenance_only".to_string(),
        },
    };
    let runtime_policy = RuntimePolicyV1 {
        schema_version: "bijux.runtime_policy.v1".to_string(),
        run_id: run_id.clone(),
        mode: RunExecutionModeV1::DryRun,
        deterministic_scheduler: request.graph.deterministic_scheduler(),
        retry_policy: request.graph.retry_policy().clone(),
        step_timeout_s: request.graph.step_timeout_s(),
        cancellation: CancellationPolicyV1 {
            supports_external_cancellation: false,
            checkpoint_before_cancel: true,
        },
        checkpoint: CheckpointPolicyV1 {
            strategy: "stage_boundary".to_string(),
            granularity: "planned_stage_set".to_string(),
            resume_from_latest_completed_stage: true,
        },
    };
    let environment = run_environment(&request.graph);
    let (_lease_lock, lease) = acquire_run_lease(&layout, &run_id)?;
    let released_lease = release_run_lease(&lease);
    let backend_descriptor =
        build_backend_record(&run_id, RunExecutionModeV1::DryRun, RuntimeKind::Local, &request.graph, &layout);
    let scheduling_decision =
        build_scheduling_decision(&run_id, &request.graph, RuntimeKind::Local);
    let mut queue_state = initial_queue_state(&run_id, &request.graph);
    queue_state.state = bijux_dna_runtime::run_layout::RunQueueLifecycleStateV1::Succeeded;
    queue_state.transitions.push(
        bijux_dna_runtime::run_layout::RunQueueTransitionV1 {
            from_state: Some(bijux_dna_runtime::run_layout::RunQueueLifecycleStateV1::Queued),
            to_state: bijux_dna_runtime::run_layout::RunQueueLifecycleStateV1::Succeeded,
            occurred_at: bijux_dna_runtime::run_layout::now_string(),
            detail: Some("dry-run completed without process execution".to_string()),
        },
    );
    let mut control_state = default_control_state(&run_id);
    control_state.observed_state =
        bijux_dna_runtime::run_layout::RunQueueLifecycleStateV1::Succeeded;
    control_state.updated_at = bijux_dna_runtime::run_layout::now_string();
    let slurm_submission =
        maybe_mock_slurm_submission(&layout, &run_id, RuntimeKind::Local, &scheduling_decision);
    let checkpoint = RunCheckpointV1 {
        schema_version: "bijux.run_checkpoint.v1".to_string(),
        run_id: run_id.clone(),
        mode: RunExecutionModeV1::DryRun,
        updated_at: bijux_dna_runtime::run_layout::now_string(),
        completed_stage_ids: Vec::new(),
        pending_stage_ids: request
            .graph
            .steps()
            .iter()
            .map(|step| step.stage_id.to_string())
            .collect(),
        next_stage_id: request.graph.steps().first().map(|step| step.stage_id.to_string()),
    };
    let transitions = vec![
        RunStateTransitionV1 {
            from_state: None,
            to_state: RunLifecycleStateV1::Planned,
            occurred_at: bijux_dna_runtime::run_layout::now_string(),
            detail: Some("dry-run request accepted".to_string()),
        },
        RunStateTransitionV1 {
            from_state: Some(RunLifecycleStateV1::Planned),
            to_state: RunLifecycleStateV1::Prepared,
            occurred_at: bijux_dna_runtime::run_layout::now_string(),
            detail: Some("dry-run artifacts materialized".to_string()),
        },
        RunStateTransitionV1 {
            from_state: Some(RunLifecycleStateV1::Prepared),
            to_state: RunLifecycleStateV1::Succeeded,
            occurred_at: bijux_dna_runtime::run_layout::now_string(),
            detail: Some("dry-run completed without process execution".to_string()),
        },
    ];
    bijux_dna_runtime::run_layout::write_executor_descriptor(&layout, &executor_descriptor)?;
    bijux_dna_runtime::run_layout::write_backend_descriptor(&layout, &backend_descriptor)?;
    bijux_dna_runtime::run_layout::write_scheduling_decision(&layout, &scheduling_decision)?;
    bijux_dna_runtime::run_layout::write_queue_state(&layout, &queue_state)?;
    bijux_dna_runtime::run_layout::write_lease(&layout, &released_lease)?;
    bijux_dna_runtime::run_layout::write_control_state(&layout, &control_state)?;
    bijux_dna_runtime::run_layout::write_runtime_policy(&layout, &runtime_policy)?;
    bijux_dna_runtime::run_layout::write_environment(&layout, &environment)?;
    bijux_dna_runtime::run_layout::write_checkpoint(&layout, &checkpoint)?;
    if let Some(slurm_submission) = slurm_submission.as_ref() {
        bijux_dna_runtime::run_layout::write_slurm_submission(&layout, slurm_submission)?;
    }
    bijux_dna_runtime::run_layout::write_run_state(
        &layout,
        &RunStateV1 {
            schema_version: "bijux.run_state.v1".to_string(),
            run_id: run_id.clone(),
            mode: RunExecutionModeV1::DryRun,
            state: RunLifecycleStateV1::Succeeded,
            transitions,
            manifest_path: Some(layout.manifest_path.clone()),
            checkpoint_path: Some(layout.checkpoint_path.clone()),
            failure_path: None,
        },
    )?;

    summary_artifact::write_run_summary_artifact(
        &layout.run_summary_path,
        "dry_run",
        request.graph.pipeline_id().as_str(),
        &layout.manifest_path,
    )?;

    let manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v3",
        "contract_version": bijux_dna_core::contract::ContractVersion::v1(),
        "run_id": run_id,
        "correlation_id": correlation_id,
        "mode": RunExecutionModeV1::DryRun,
        "state": RunLifecycleStateV1::Succeeded,
        "pipeline_id": request.graph.pipeline_id().to_string(),
        "profile_id": request.profile_id,
        "graph_hash": graph_hash,
        "cache_key": serde_json::Value::Null,
        "toolchain_versions": [],
        "dataset_fingerprints": [],
        "tool_invocations": [],
        "output_artifacts": vec![
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
                "environment",
                "bijux.run_environment.v1",
                &layout.environment_path,
            )?,
            artifact_entry(
                &layout.run_dir,
                "executor_descriptor",
                "bijux.run_executor_descriptor.v1",
                &layout.executor_descriptor_path,
            )?,
            artifact_entry(
                &layout.run_dir,
                "backend_descriptor",
                "bijux.run_backend.v1",
                &layout.backend_descriptor_path,
            )?,
            artifact_entry(
                &layout.run_dir,
                "scheduling_decision",
                "bijux.run_scheduling_decision.v1",
                &layout.scheduling_decision_path,
            )?,
            artifact_entry(
                &layout.run_dir,
                "queue_state",
                "bijux.run_queue_state.v1",
                &layout.queue_state_path,
            )?,
            artifact_entry(
                &layout.run_dir,
                "run_lease",
                "bijux.run_lease.v1",
                &layout.lease_path,
            )?,
            artifact_entry(
                &layout.run_dir,
                "run_control",
                "bijux.run_control.v1",
                &layout.control_state_path,
            )?,
            artifact_entry(
                &layout.run_dir,
                "checkpoint",
                "bijux.run_checkpoint.v1",
                &layout.checkpoint_path,
            )?,
            serde_json::json!({
                "name": "operator_health",
                "kind": "operator_health",
                "schema": "bijux.operator_health.v1",
                "path": summary_artifact::relative_path_string(&layout.run_dir, &layout.health_report_path),
                "sha256": serde_json::Value::Null,
            }),
            artifact_entry(
                &layout.run_dir,
                "run_summary",
                "bijux.run_summary.v1",
                &layout.run_summary_path,
            )?,
            serde_json::json!({
                "name": "run_manifest",
                "kind": "run_manifest",
                "schema": "bijux.run_manifest.v3",
                "path": summary_artifact::relative_path_string(&layout.run_dir, &layout.manifest_path),
                "sha256": serde_json::Value::Null,
            }),
        ],
        "stages": summary_artifact::planned_stage_manifest(&request.graph),
        "failures": [],
    });
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)?;
    bijux_dna_infra::atomic_write_bytes(&layout.manifest_path, payload.as_slice())?;
    let health_report = build_health_report(&layout, &run_id, RuntimeKind::Local);
    bijux_dna_runtime::run_layout::write_health_report(&layout, &health_report)?;
    summary_artifact::attach_output_artifact(
        &layout.manifest_path,
        &request.run_dir,
        &correlation_id,
        "operator_health",
        "bijux.operator_health.v1",
        &layout.health_report_path,
    )?;
    if slurm_submission.is_some() {
        summary_artifact::attach_output_artifact(
            &layout.manifest_path,
            &request.run_dir,
            &correlation_id,
            "slurm_submission",
            "bijux.slurm_submission.v1",
            &layout.slurm_submission_path,
        )?;
    }

    let governed = materialize_governed_evidence(
        &layout,
        &request.graph,
        &run_id,
        RunExecutionModeV1::DryRun,
        RunLifecycleStateV1::Succeeded,
        &run_id,
        Vec::new(),
        vec![bijux_dna_runtime::run_layout::CacheDecisionV1 {
            stage_id: "dry_run".to_string(),
            status: "miss".to_string(),
            cache_key: None,
            reason_code: Some("no_runtime_cache_in_dry_run".to_string()),
            message: "dry-run records cache semantics but does not attempt cache reuse".to_string(),
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
            &request.run_dir,
            &correlation_id,
            name,
            schema,
            path,
        )?;
    }

    let evidence_bundle_path =
        bijux_dna_analyze::write_evidence_bundle_json(&request.run_dir, None)?;
    summary_artifact::attach_output_artifact(
        &layout.manifest_path,
        &request.run_dir,
        &correlation_id,
        "evidence_bundle",
        "bijux.evidence_bundle.v1",
        &evidence_bundle_path,
    )?;
    let evidence_verification = bijux_dna_analyze::verify_evidence_bundle(&evidence_bundle_path)?;
    bijux_dna_infra::atomic_write_json(&layout.evidence_verification_path, &evidence_verification)?;
    summary_artifact::attach_output_artifact(
        &layout.manifest_path,
        &request.run_dir,
        &correlation_id,
        "evidence_verification",
        "bijux.evidence_verification.v1",
        &layout.evidence_verification_path,
    )?;
    bijux_dna_runtime::recording::write_profile_and_lock_manifests(&layout.manifest_path)?;
    Ok(DryRunResponse {
        graph_path: layout.graph_path,
        manifest_path: layout.manifest_path,
        run_summary_path: layout.run_summary_path,
        run_summary_text_path: layout.run_summary_text_path,
        run_state_path: layout.run_state_path,
        runtime_policy_path: layout.runtime_policy_path,
        executor_descriptor_path: layout.executor_descriptor_path,
        backend_descriptor_path: layout.backend_descriptor_path,
        scheduling_decision_path: layout.scheduling_decision_path,
        queue_state_path: layout.queue_state_path,
        lease_path: layout.lease_path,
        control_state_path: layout.control_state_path,
        health_report_path: layout.health_report_path,
        checkpoint_path: layout.checkpoint_path,
        mode: RunExecutionModeV1::DryRun,
        state: RunLifecycleStateV1::Succeeded,
        evidence_bundle_path,
        evidence_verification_path: layout.evidence_verification_path,
        artifact_inventory_path: layout.artifact_inventory_path,
        replay_manifest_path: layout.replay_manifest_path,
        hash_ledger_path: layout.hash_ledger_path,
        slurm_submission_path: slurm_submission.map(|_| layout.slurm_submission_path),
        correlation_id,
    })
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
        "sha256": bijux_dna_infra::hash_file_sha256(path)?,
    }))
}

fn run_environment(graph: &bijux_dna_core::contract::ExecutionGraph) -> RunEnvironment {
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    let mut tool_images = graph
        .steps()
        .iter()
        .map(|step| ToolImageDigest {
            tool: step.image.image.clone(),
            image: step.image.image.clone(),
            digest: step.image.digest.clone().unwrap_or_else(|| "unresolved".to_string()),
        })
        .collect::<Vec<_>>();
    tool_images.sort_by(|left, right| left.tool.cmp(&right.tool));
    tool_images.dedup_by(|left, right| left.tool == right.tool && left.digest == right.digest);
    RunEnvironment {
        hostname,
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        runner: "local".to_string(),
        platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        tool_images,
    }
}
