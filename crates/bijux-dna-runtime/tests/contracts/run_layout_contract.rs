use std::path::PathBuf;

use bijux_dna_runtime::run_layout::{
    admit_runtime_resources, apply_control_action_idempotent, apptainer_smoke_workflow_plan,
    create_run_layout, docker_smoke_workflow_plan, evaluate_fallback_safety,
    executor_descriptor_from_hpc_profile, lunarc_execution_profile,
    negotiate_executor_capabilities, restore_queue_state_for_resume, transition_slurm_submission,
    validate_run_layout_storage_isolation, ExecutorCapabilitiesV1, FallbackSafetyRequestV1,
    RunCheckpointV1, RunControlActionV1, RunControlStateV1, RunExecutionModeV1,
    RunQueueLifecycleStateV1, RunQueueStateV1, RunResourceRequestV1, RuntimeResourceLimitsV1,
    SlurmJobStateV1, SlurmSubmissionRecordV1, StageExecutionRequirementV1,
};

#[test]
fn run_layout_paths_match_contract() {
    let temp = bijux_dna_testkit::tempdir_for("run-layout");
    let temp_root = temp.path().to_path_buf();
    let (run_id, layout) = match create_run_layout(&temp_root) {
        Ok(value) => value,
        Err(err) => panic!("layout: {err}"),
    };
    assert!(run_id.starts_with("run-"), "run layout id must use the public run-* prefix");
    assert!(layout.run_dir.ends_with(&run_id), "run layout directory must end with run id");
    assert!(layout.manifests_dir.ends_with("manifests"));
    assert!(layout.logs_dir.ends_with("logs"));
    assert!(layout.reports_dir.ends_with("reports"));
    assert!(layout.assessment_path.ends_with("input_assessment.json"));
    assert!(layout.graph_path.ends_with("manifests/graph.json"));
    assert!(layout.plan_manifest_path.ends_with("manifests/plan_manifest.json"));
    assert!(layout.manifest_path.ends_with("run_manifest.json"));
    assert!(layout.environment_path.ends_with("environment.json"));
    assert!(layout.metadata_path.ends_with("run_metadata.json"));
    assert!(layout.events_path.ends_with("events.jsonl"));
    assert!(layout.run_state_path.ends_with("run_state.json"));
    assert!(layout.runtime_policy_path.ends_with("runtime_policy.json"));
    assert!(layout.executor_descriptor_path.ends_with("executor_descriptor.json"));
    assert!(layout.backend_descriptor_path.ends_with("backend_descriptor.json"));
    assert!(layout.scheduling_decision_path.ends_with("scheduling_decision.json"));
    assert!(layout.queue_state_path.ends_with("queue_state.json"));
    assert!(layout.lease_path.ends_with("run_lease.json"));
    assert!(layout.control_state_path.ends_with("run_control.json"));
    assert!(layout.health_report_path.ends_with("operator_health.json"));
    assert!(layout.slurm_submission_path.ends_with("slurm_submission.json"));
    assert!(layout.checkpoint_path.ends_with("checkpoints/checkpoint.json"));
    assert!(layout.failure_path.ends_with("run_failure.json"));
    assert!(layout.run_summary_path.ends_with("summary/run_summary.json"));
    assert!(layout.run_summary_text_path.ends_with("summary/run_summary.txt"));
    assert!(layout.artifact_inventory_path.ends_with("artifact_inventory.json"));
    assert!(layout.artifact_inventory_text_path.ends_with("artifact_inventory.txt"));
    assert!(layout.replay_manifest_path.ends_with("replay_manifest.json"));
    assert!(layout.hash_ledger_path.ends_with("hash_ledger.json"));
    assert!(layout.evidence_verification_path.ends_with("evidence_verification.json"));
    assert!(layout.evidence_bundle_path.ends_with("evidence_bundle.json"));
    assert!(layout.summary_dir.ends_with("summary"));
}

#[test]
fn docker_smoke_workflow_plan_captures_digest_mounts_and_artifacts() {
    let plan = docker_smoke_workflow_plan(
        "run-131",
        "docker.io/bijuxdna/smoke@sha256:1234",
        PathBuf::from("/tmp/bijux/smoke"),
    );
    assert_eq!(plan.runner, "docker");
    assert_eq!(plan.image_identity, "docker.io/bijuxdna/smoke@sha256:1234");
    assert_eq!(plan.mounts.len(), 2);
    assert!(plan.mounts.iter().any(|mount| mount.access == "read_only"));
    assert!(plan.mounts.iter().any(|mount| mount.access == "read_write"));
    assert!(plan.expected_artifacts.iter().any(|item| item == "smoke.stdout"));
}

#[test]
fn apptainer_smoke_workflow_plan_captures_sif_identity_bindings_and_logs() {
    let plan = apptainer_smoke_workflow_plan(
        "run-132",
        "library://bijux/smoke/tool.sif@sha256:beef",
        PathBuf::from("/tmp/bijux/apptainer"),
    );
    assert_eq!(plan.runner, "apptainer");
    assert_eq!(plan.image_identity, "library://bijux/smoke/tool.sif@sha256:beef");
    assert_eq!(plan.mounts.len(), 2);
    assert!(
        plan.log_capture_policy.contains("runtime_logs"),
        "apptainer smoke plan should preserve runtime log capture policy",
    );
}

#[test]
fn slurm_submission_lifecycle_records_submit_poll_cancel_complete_transitions() {
    let mut submission = SlurmSubmissionRecordV1 {
        schema_version: "bijux.slurm_submission.v1".to_string(),
        run_id: "run-133".to_string(),
        scheduler: "slurm".to_string(),
        submission_script_path: "submit.sh".into(),
        job_id: "12345".to_string(),
        state: SlurmJobStateV1::Submitted,
        poll_command: vec!["squeue".to_string(), "-j".to_string(), "12345".to_string()],
        cancel_command: vec!["scancel".to_string(), "12345".to_string()],
        stdout_log_path: "stdout.log".into(),
        stderr_log_path: "stderr.log".into(),
        retry_count: 0,
        exit_code: None,
        transitions: Vec::new(),
    };
    transition_slurm_submission(
        &mut submission,
        SlurmJobStateV1::Pending,
        "2026-04-30T10:00:00Z",
        Some("queued".to_string()),
    )
    .expect("submitted->pending must be valid");
    transition_slurm_submission(
        &mut submission,
        SlurmJobStateV1::Running,
        "2026-04-30T10:05:00Z",
        Some("node assigned".to_string()),
    )
    .expect("pending->running must be valid");
    transition_slurm_submission(
        &mut submission,
        SlurmJobStateV1::Cancelled,
        "2026-04-30T10:06:00Z",
        Some("operator cancelled".to_string()),
    )
    .expect("running->cancelled must be valid");

    assert_eq!(submission.state, SlurmJobStateV1::Cancelled);
    assert_eq!(submission.transitions.len(), 3);
    assert_eq!(submission.transitions[0].to_state, SlurmJobStateV1::Pending);
    assert_eq!(submission.transitions[1].to_state, SlurmJobStateV1::Running);
    assert_eq!(submission.transitions[2].to_state, SlurmJobStateV1::Cancelled);
}

#[test]
fn lunarc_profile_uses_site_isolated_configuration_and_hpc_descriptor_shape() {
    let profile = lunarc_execution_profile(PathBuf::from("/etc/bijux/sites/lunarc.toml"));
    assert_eq!(profile.profile_id, "lunarc");
    assert_eq!(profile.scheduler, "slurm");
    assert_eq!(profile.submission_mode, "batch");
    assert_eq!(profile.site_config_path, PathBuf::from("/etc/bijux/sites/lunarc.toml"));

    let descriptor =
        executor_descriptor_from_hpc_profile("run-134", RunExecutionModeV1::Enforced, &profile);
    assert_eq!(descriptor.run_id, "run-134");
    match descriptor.descriptor {
        bijux_dna_runtime::run_layout::ExecutorDescriptorV1::Hpc {
            scheduler,
            submission_mode,
            container_runtime,
            ..
        } => {
            assert_eq!(scheduler, "slurm");
            assert_eq!(submission_mode, "batch");
            assert_eq!(container_runtime.as_deref(), Some("apptainer"));
        }
        _ => panic!("lunarc profile must resolve to an HPC executor descriptor"),
    }
}

#[test]
fn executor_capability_negotiation_marks_stage_runnable_only_when_contracts_match() {
    let requirement = StageExecutionRequirementV1 {
        stage_id: "fastq.trim_reads".to_string(),
        requires_local_runtime: false,
        required_container_runtime: Some("docker".to_string()),
        required_scheduler: None,
        required_evidence_topics: vec!["tool_provenance".to_string(), "runtime_logs".to_string()],
    };
    let available = ExecutorCapabilitiesV1 {
        runner: "docker".to_string(),
        supports_local_runtime: true,
        container_runtimes: vec!["docker".to_string()],
        schedulers: vec![],
        evidence_topics: vec!["tool_provenance".to_string()],
    };
    let decision = negotiate_executor_capabilities(&requirement, &available);
    assert!(decision.admitted, "runtime capability should be admitted");
    assert!(
        decision.warnings.iter().any(|item| item.contains("runtime_logs")),
        "missing evidence topics should appear as warnings",
    );

    let unavailable = ExecutorCapabilitiesV1 {
        runner: "local".to_string(),
        supports_local_runtime: true,
        container_runtimes: vec![],
        schedulers: vec![],
        evidence_topics: vec![],
    };
    let denied = negotiate_executor_capabilities(&requirement, &unavailable);
    assert!(!denied.admitted);
    assert!(denied.refusal_codes.iter().any(|item| item == "missing_container_runtime"));
}

#[test]
fn fallback_safety_rejects_non_equivalent_outputs_or_missing_evidence_obligations() {
    let unsafe_request = FallbackSafetyRequestV1 {
        primary_runner: "docker".to_string(),
        fallback_runner: "apptainer".to_string(),
        output_contract_hash: "sha256:abc".to_string(),
        fallback_output_contract_hash: "sha256:def".to_string(),
        evidence_obligations: vec!["tool_provenance".to_string(), "runtime_logs".to_string()],
        fallback_evidence_topics: vec!["tool_provenance".to_string()],
    };
    let denied = evaluate_fallback_safety(&unsafe_request);
    assert!(!denied.safe);
    assert!(denied.refusal_codes.iter().any(|item| item == "fallback_output_contract_mismatch"));
    assert!(denied.refusal_codes.iter().any(|item| item == "fallback_evidence_obligation_gap"));

    let safe_request = FallbackSafetyRequestV1 {
        primary_runner: "docker".to_string(),
        fallback_runner: "docker".to_string(),
        output_contract_hash: "sha256:abc".to_string(),
        fallback_output_contract_hash: "sha256:abc".to_string(),
        evidence_obligations: vec!["tool_provenance".to_string()],
        fallback_evidence_topics: vec!["tool_provenance".to_string(), "runtime_logs".to_string()],
    };
    let accepted = evaluate_fallback_safety(&safe_request);
    assert!(accepted.safe);
    assert!(accepted.refusal_codes.is_empty());
}

#[test]
fn runtime_resource_admission_warns_or_refuses_before_execution() {
    let limits = RuntimeResourceLimitsV1 {
        max_cpu_threads: 16,
        max_memory_mb: Some(64_000),
        max_scratch_mb: Some(128_000),
        max_walltime_s: Some(28_800),
        allowed_io_intensity: vec!["low".to_string(), "medium".to_string(), "high".to_string()],
    };
    let request = RunResourceRequestV1 {
        cpu_threads: 12,
        memory_mb: Some(32_000),
        scratch_mb: Some(64_000),
        walltime_s: Some(14_400),
        io_intensity: "high".to_string(),
        container_runtime: Some("apptainer".to_string()),
    };
    let admitted = admit_runtime_resources(&request, &limits);
    assert!(admitted.admitted);
    assert!(admitted.refusal_codes.is_empty());
    assert_eq!(admitted.queue_class, "high_resource");

    let denied_request = RunResourceRequestV1 {
        cpu_threads: 32,
        memory_mb: Some(128_000),
        scratch_mb: Some(256_000),
        walltime_s: Some(36_000),
        io_intensity: "extreme".to_string(),
        container_runtime: None,
    };
    let denied = admit_runtime_resources(&denied_request, &limits);
    assert!(!denied.admitted);
    assert!(denied.refusal_codes.iter().any(|item| item == "cpu_threads_exceed_limit"));
    assert!(denied.refusal_codes.iter().any(|item| item == "io_intensity_not_allowed"));
    assert!(denied.warnings.iter().any(|item| item == "container_runtime_unspecified"));
}

#[test]
fn queue_restore_persists_resume_state_without_duplicate_dispatch() {
    let queue_state = RunQueueStateV1 {
        schema_version: "bijux.run_queue_state.v1".to_string(),
        run_id: "run-138".to_string(),
        dedup_key: "sha256:graph".to_string(),
        state: RunQueueLifecycleStateV1::Running,
        transitions: Vec::new(),
        active_step_id: Some("fastq.trim_reads".to_string()),
    };
    let checkpoint = RunCheckpointV1 {
        schema_version: "bijux.run_checkpoint.v1".to_string(),
        run_id: "run-138".to_string(),
        mode: RunExecutionModeV1::Enforced,
        updated_at: "2026-04-30T11:00:00Z".to_string(),
        completed_stage_ids: vec!["fastq.trim_reads".to_string()],
        pending_stage_ids: vec!["fastq.trim_reads".to_string(), "fastq.filter_reads".to_string()],
        next_stage_id: Some("fastq.filter_reads".to_string()),
    };

    let restored = restore_queue_state_for_resume(&queue_state, &checkpoint);
    assert!(restored.deduplicated_dispatch);
    assert_eq!(restored.restored_state.active_step_id, None);
    assert_eq!(restored.restored_state.state, RunQueueLifecycleStateV1::Queued);
    assert_eq!(restored.resumed_stage_ids, vec!["fastq.filter_reads".to_string()]);
    assert_eq!(restored.blocked_stage_ids, vec!["fastq.trim_reads".to_string()]);
}

#[test]
fn pause_resume_cancel_controls_are_idempotent_and_audited() {
    let mut queue_state = RunQueueStateV1 {
        schema_version: "bijux.run_queue_state.v1".to_string(),
        run_id: "run-139".to_string(),
        dedup_key: "sha256:graph".to_string(),
        state: RunQueueLifecycleStateV1::Running,
        transitions: Vec::new(),
        active_step_id: Some("bam.align".to_string()),
    };
    let mut control_state = RunControlStateV1 {
        schema_version: "bijux.run_control.v1".to_string(),
        run_id: "run-139".to_string(),
        requested_action: None,
        observed_state: RunQueueLifecycleStateV1::Running,
        updated_at: "2026-04-30T12:00:00Z".to_string(),
        audit_log: Vec::new(),
    };

    let changed = apply_control_action_idempotent(
        &mut control_state,
        &mut queue_state,
        RunControlActionV1::Pause,
        "2026-04-30T12:01:00Z",
        Some("pause for operator inspection".to_string()),
    );
    assert!(changed);
    assert_eq!(queue_state.state, RunQueueLifecycleStateV1::Paused);

    let changed_again = apply_control_action_idempotent(
        &mut control_state,
        &mut queue_state,
        RunControlActionV1::Pause,
        "2026-04-30T12:02:00Z",
        None,
    );
    assert!(!changed_again);

    let cancelled = apply_control_action_idempotent(
        &mut control_state,
        &mut queue_state,
        RunControlActionV1::Cancel,
        "2026-04-30T12:03:00Z",
        Some("operator cancelled".to_string()),
    );
    assert!(cancelled);
    assert_eq!(queue_state.state, RunQueueLifecycleStateV1::Cancelled);
    assert!(control_state.audit_log.len() >= 3);
}

#[test]
fn storage_isolation_rejects_paths_outside_run_root_or_workspace() {
    let temp = bijux_dna_testkit::tempdir_for("run-layout-isolation");
    let root = temp.path().to_path_buf();
    let (_, mut layout) = create_run_layout(&root).expect("create run layout");

    let valid = validate_run_layout_storage_isolation(&layout, &root);
    assert!(valid.valid, "fresh run layout should be isolated");

    layout.run_summary_path = root.join("outside_summary.json");
    let invalid = validate_run_layout_storage_isolation(&layout, &root);
    assert!(!invalid.valid);
    assert!(invalid.refusal_codes.iter().any(|item| item == "layout_path_outside_run_dir"));
}
