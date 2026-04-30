use super::evidence_support::materialize_governed_evidence;
use super::planner_manifest_support::plan_manifest_from_request;
use super::{summary_artifact, Result};
use crate::request_args::{DryRunRequest, DryRunResponse};
use crate::request_args::PlanRequest;
use bijux_dna_runtime::run_layout::{
    CancellationPolicyV1, CheckpointPolicyV1, ExecutorDescriptorV1, RunCheckpointV1,
    RunExecutionModeV1, RunExecutorDescriptorV1, RunLifecycleStateV1, RunStateTransitionV1,
    RunStateV1, RuntimePolicyV1,
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
    bijux_dna_runtime::run_layout::write_runtime_policy(&layout, &runtime_policy)?;
    bijux_dna_runtime::run_layout::write_checkpoint(&layout, &checkpoint)?;
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
        checkpoint_path: layout.checkpoint_path,
        mode: RunExecutionModeV1::DryRun,
        state: RunLifecycleStateV1::Succeeded,
        evidence_bundle_path,
        evidence_verification_path: layout.evidence_verification_path,
        artifact_inventory_path: layout.artifact_inventory_path,
        replay_manifest_path: layout.replay_manifest_path,
        hash_ledger_path: layout.hash_ledger_path,
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
