use std::path::PathBuf;

use bijux_dna_runtime::run_layout::{
    apptainer_smoke_workflow_plan, create_run_layout, docker_smoke_workflow_plan,
    executor_descriptor_from_hpc_profile, lunarc_execution_profile, transition_slurm_submission,
    RunExecutionModeV1, SlurmJobStateV1, SlurmSubmissionRecordV1,
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
