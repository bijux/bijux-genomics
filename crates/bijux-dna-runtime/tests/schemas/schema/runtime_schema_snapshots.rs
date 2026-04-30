use std::collections::BTreeMap;

use bijux_dna_core::contract::ContractVersion;
use bijux_dna_core::metrics::ToolInvocationV1;
use bijux_dna_runtime::observability::RunProvenanceV1;
use bijux_dna_runtime::run_layout::{
    ExecutorDescriptorV1, OperatorHealthCheckV1, OperatorHealthReportV1, RunBackendDescriptorV1,
    RunBackendRecordV1, RunControlActionV1, RunControlAuditEntryV1, RunControlStateV1,
    RunExecutionModeV1, RunFailureV1, RunLayoutV1, RunLeaseV1, RunLifecycleStateV1,
    RunManifest, RunQueueLifecycleStateV1, RunQueueStateV1, RunQueueTransitionV1,
    RunSchedulingDecisionV1, RunStateV1, SlurmJobStateV1, SlurmJobTransitionV1,
    SlurmSubmissionRecordV1,
};

#[path = "../../support/workspace_paths.rs"]
mod support;

#[test]
fn run_layout_schema_snapshot() {
    let layout = RunLayoutV1 {
        schema_version: "bijux.run_layout.v1".to_string(),
        run_dir: "run".to_string(),
        stages_dir: "stages".to_string(),
        manifests_dir: "manifests".to_string(),
        logs_dir: "logs".to_string(),
        reports_dir: "reports".to_string(),
        summary_dir: "summary".to_string(),
        run_artifacts_dir: "run_artifacts".to_string(),
        checkpoints_dir: "checkpoints".to_string(),
        assessment_path: "input_assessment.json".to_string(),
        graph_path: "graph.json".to_string(),
        plan_manifest_path: "plan_manifest.json".to_string(),
        manifest_path: "run_manifest.json".to_string(),
        environment_path: "environment.json".to_string(),
        metadata_path: "run_metadata.json".to_string(),
        events_path: "events.jsonl".to_string(),
        run_state_path: "run_state.json".to_string(),
        runtime_policy_path: "runtime_policy.json".to_string(),
        executor_descriptor_path: "executor_descriptor.json".to_string(),
        backend_descriptor_path: "backend_descriptor.json".to_string(),
        scheduling_decision_path: "scheduling_decision.json".to_string(),
        queue_state_path: "queue_state.json".to_string(),
        lease_path: "run_lease.json".to_string(),
        control_state_path: "run_control.json".to_string(),
        health_report_path: "operator_health.json".to_string(),
        slurm_submission_path: "slurm_submission.json".to_string(),
        checkpoint_path: "checkpoint.json".to_string(),
        failure_path: "run_failure.json".to_string(),
        run_summary_path: "run_summary.json".to_string(),
        run_summary_text_path: "run_summary.txt".to_string(),
        artifact_inventory_path: "artifact_inventory.json".to_string(),
        artifact_inventory_text_path: "artifact_inventory.txt".to_string(),
        replay_manifest_path: "replay_manifest.json".to_string(),
        hash_ledger_path: "hash_ledger.json".to_string(),
        evidence_verification_path: "evidence_verification.json".to_string(),
        evidence_bundle_path: "evidence_bundle.json".to_string(),
    };
    let expected = include_str!("../../fixtures/runtime_schema/default/run_layout_v1.json");
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&layout)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(actual.trim_end(), expected.trim_end());
}

#[test]
fn run_record_schema_snapshot() {
    let record = bijux_dna_core::contract::RunRecordV1::new(vec![
        bijux_dna_core::contract::StageExecutionRecordV1 {
            stage_id: "fastq.trim_reads".to_string(),
            attempt: 0,
            success: true,
            cached: false,
        },
    ]);
    let expected = include_str!("../../fixtures/runtime_schema/default/run_record_v1.json");
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&record)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(actual, expected);
}

#[test]
fn run_provenance_schema_snapshot() {
    let provenance = RunProvenanceV1 {
        schema_version: "bijux.run_provenance.v1".to_string(),
        tool_image_digest: Some("sha256:img".to_string()),
        tool_version: "1.0".to_string(),
        params_hash: "sha256:params".to_string(),
        input_hashes: vec!["sha256:input".to_string()],
        reference_genome: None,
        pipeline_id: "fastq-to-fastq__default__v1".to_string(),
        git_commit: "abc".to_string(),
        build_profile: "dev".to_string(),
        plan_hash: None,
    };
    let expected = include_str!("../../fixtures/runtime_schema/default/run_provenance_v1.json");
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&provenance)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(actual, expected);
}

#[test]
fn run_manifest_schema_snapshot() {
    let invocation = ToolInvocationV1 {
        schema_version: "bijux.tool_invocation.v1".to_string(),
        contract_version: ContractVersion::v1(),
        stage_id: bijux_dna_core::ids::StageId::new("fastq.trim_reads"),
        tool_id: bijux_dna_core::ids::ToolId::new("fastp"),
        tool_version: "1.0".to_string(),
        resolved_tool_version: None,
        image_digest: "sha256:img".to_string(),
        runner_kind: "docker".to_string(),
        platform: "local".to_string(),
        parameters_json: serde_json::json!({}),
        parameters_json_normalized: serde_json::json!({}),
        effective_params_json: serde_json::json!({}),
        effective_params_json_normalized: serde_json::json!({}),
        params_provenance: serde_json::json!({}),
        params_provenance_normalized: serde_json::json!({}),
        adapter_bank: None,
        banks: None,
        bank_assets: None,
        resources: bijux_dna_core::contract::ToolConstraints::default(),
        environment: BTreeMap::default(),
        input_hashes: vec!["sha256:input".to_string()],
        output_hashes: vec!["sha256:output".to_string()],
        executed_command: None,
    };
    let manifest = RunManifest {
        schema_version: "bijux.run_manifest.v3".to_string(),
        contract_version: ContractVersion::v1(),
        run_id: "run-1".to_string(),
        started_at: "2024-01-01T00:00:00Z".to_string(),
        finished_at: "2024-01-01T00:00:10Z".to_string(),
        pipeline: "fastq-to-fastq__default__v1".to_string(),
        graph_hash: "sha256:graph".to_string(),
        cache_key: None,
        layout: bijux_dna_core::prelude::input_assessment::FastqLayout::SingleEnd,
        stages: Vec::new(),
        tool_invocations: vec![invocation],
        artifacts: Vec::new(),
    };
    let expected = include_str!("../../fixtures/runtime_schema/default/run_manifest_v1.json");
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(actual, expected);
}

#[test]
fn run_state_schema_snapshot() {
    let state = RunStateV1 {
        schema_version: "bijux.run_state.v1".to_string(),
        run_id: "run-1".to_string(),
        mode: RunExecutionModeV1::Enforced,
        state: RunLifecycleStateV1::Succeeded,
        transitions: Vec::new(),
        manifest_path: Some("run_manifest.json".into()),
        checkpoint_path: Some("checkpoints/checkpoint.json".into()),
        failure_path: None,
    };
    let expected = include_str!("../../fixtures/runtime_schema/default/run_state_v1.json");
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&state)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(actual.trim_end(), expected.trim_end());
}

#[test]
fn run_failure_schema_snapshot() {
    let failure = RunFailureV1 {
        schema_version: "bijux.run_failure.v1".to_string(),
        run_id: "run-1".to_string(),
        mode: RunExecutionModeV1::Enforced,
        state: RunLifecycleStateV1::Failed,
        failure_code: "runner_execution_failed".to_string(),
        message: "step failed after retries: fastq.validate_reads".to_string(),
        stage_id: Some("fastq.validate_reads".to_string()),
        step_id: Some("fastq.validate_reads".to_string()),
        attempt: None,
        observed_at: "2024-01-01T00:00:10Z".to_string(),
        retryable: true,
    };
    let expected = include_str!("../../fixtures/runtime_schema/default/run_failure_v1.json");
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&failure)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(actual.trim_end(), expected.trim_end());
}

#[test]
fn executor_descriptor_schema_snapshot() {
    let descriptor = ExecutorDescriptorV1::Hpc {
        scheduler: "slurm".to_string(),
        submission_mode: "batch".to_string(),
        scratch_layout_policy: "stage_scoped_scratch".to_string(),
        container_runtime: Some("apptainer".to_string()),
    };
    let expected =
        include_str!("../../fixtures/runtime_schema/default/executor_descriptor_v1.json");
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&descriptor)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(actual.trim_end(), expected.trim_end());
}

#[test]
fn runtime_operations_schema_snapshots() {
    let backend = RunBackendRecordV1 {
        schema_version: "bijux.run_backend.v1".to_string(),
        run_id: "run-1".to_string(),
        mode: RunExecutionModeV1::Enforced,
        descriptor: RunBackendDescriptorV1::Container {
            runtime: "apptainer".to_string(),
            image_identity: "docker://example/tool@sha256:deadbeef".to_string(),
            bind_mount_policy: "readonly_inputs_writable_run_dir_hpc_safe".to_string(),
            user_identity_policy: "host_user_passthrough".to_string(),
            working_directory_policy: "run_dir_scoped_workdir".to_string(),
            network_policy: "inherit_site_policy_no_implicit_network".to_string(),
            resource_limit_policy: "graph_constraints_promoted_to_scheduler_hints".to_string(),
            stdout_stderr_policy: "captured_from_runtime_process".to_string(),
        },
    };
    let backend_expected =
        include_str!("../../fixtures/runtime_schema/default/run_backend_v1.json");
    let backend_actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&backend)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(backend_actual.trim_end(), backend_expected.trim_end());

    let scheduling = RunSchedulingDecisionV1 {
        schema_version: "bijux.run_scheduling_decision.v1".to_string(),
        run_id: "run-1".to_string(),
        runner: "apptainer".to_string(),
        queue_class: "slurm_batch_candidate".to_string(),
        placement_reason: "selected slurm_batch_candidate from threads=8, memory_mb=16384, steps=6"
            .to_string(),
        requested_resources: bijux_dna_runtime::run_layout::RunResourceRequestV1 {
            cpu_threads: 8,
            memory_mb: Some(16_384),
            scratch_mb: Some(32_768),
            walltime_s: Some(7_200),
            io_intensity: "high".to_string(),
            container_runtime: Some("apptainer".to_string()),
        },
        warnings: vec![
            "runner should prefer mocked slurm submission semantics for large apptainer-backed work"
                .to_string(),
        ],
    };
    let scheduling_expected =
        include_str!("../../fixtures/runtime_schema/default/run_scheduling_decision_v1.json");
    let scheduling_actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&scheduling)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(scheduling_actual.trim_end(), scheduling_expected.trim_end());

    let queue_state = RunQueueStateV1 {
        schema_version: "bijux.run_queue_state.v1".to_string(),
        run_id: "run-1".to_string(),
        dedup_key: "sha256:graph".to_string(),
        state: RunQueueLifecycleStateV1::Paused,
        transitions: vec![RunQueueTransitionV1 {
            from_state: Some(RunQueueLifecycleStateV1::Running),
            to_state: RunQueueLifecycleStateV1::Paused,
            occurred_at: "2024-01-01T00:00:05Z".to_string(),
            detail: Some("pause requested before fastq.validate_reads".to_string()),
        }],
        active_step_id: Some("fastq.validate_reads".to_string()),
    };
    let queue_expected =
        include_str!("../../fixtures/runtime_schema/default/run_queue_state_v1.json");
    let queue_actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&queue_state)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(queue_actual.trim_end(), queue_expected.trim_end());

    let lease = RunLeaseV1 {
        schema_version: "bijux.run_lease.v1".to_string(),
        run_id: "run-1".to_string(),
        lease_id: "lease-run-1".to_string(),
        holder: "worker-1:pid=123".to_string(),
        lock_path: "run.lock".into(),
        acquired_at: "2024-01-01T00:00:00Z".to_string(),
        expires_at: None,
        released_at: Some("2024-01-01T00:00:10Z".to_string()),
        exclusive: true,
    };
    let lease_expected = include_str!("../../fixtures/runtime_schema/default/run_lease_v1.json");
    let lease_actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&lease)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(lease_actual.trim_end(), lease_expected.trim_end());

    let control = RunControlStateV1 {
        schema_version: "bijux.run_control.v1".to_string(),
        run_id: "run-1".to_string(),
        requested_action: Some(RunControlActionV1::Pause),
        observed_state: RunQueueLifecycleStateV1::Paused,
        updated_at: "2024-01-01T00:00:05Z".to_string(),
        audit_log: vec![RunControlAuditEntryV1 {
            requested_action: RunControlActionV1::Pause,
            observed_state: RunQueueLifecycleStateV1::Paused,
            occurred_at: "2024-01-01T00:00:05Z".to_string(),
            detail: Some("paused before fastq.validate_reads".to_string()),
        }],
    };
    let control_expected =
        include_str!("../../fixtures/runtime_schema/default/run_control_v1.json");
    let control_actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&control)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(control_actual.trim_end(), control_expected.trim_end());

    let health = OperatorHealthReportV1 {
        schema_version: "bijux.operator_health.v1".to_string(),
        run_id: "run-1".to_string(),
        overall_ok: false,
        checks: vec![
            OperatorHealthCheckV1 {
                check_id: "storage".to_string(),
                ok: true,
                detail: "run layout root run-1".to_string(),
                evidence_path: Some("run-1".into()),
            },
            OperatorHealthCheckV1 {
                check_id: "containers".to_string(),
                ok: false,
                detail: "runtime apptainer availability checked on PATH".to_string(),
                evidence_path: None,
            },
        ],
    };
    let health_expected =
        include_str!("../../fixtures/runtime_schema/default/operator_health_v1.json");
    let health_actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&health)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(health_actual.trim_end(), health_expected.trim_end());

    let slurm = SlurmSubmissionRecordV1 {
        schema_version: "bijux.slurm_submission.v1".to_string(),
        run_id: "run-1".to_string(),
        scheduler: "slurm".to_string(),
        submission_script_path: "slurm_submit.sh".into(),
        job_id: "mock-run-1".to_string(),
        state: SlurmJobStateV1::Submitted,
        poll_command: vec!["squeue".to_string(), "--job".to_string(), "mock-run-1".to_string()],
        cancel_command: vec!["scancel".to_string(), "mock-run-1".to_string()],
        stdout_log_path: "logs/slurm.stdout.log".into(),
        stderr_log_path: "logs/slurm.stderr.log".into(),
        retry_count: 0,
        exit_code: None,
        transitions: vec![SlurmJobTransitionV1 {
            from_state: None,
            to_state: SlurmJobStateV1::Submitted,
            occurred_at: "2024-01-01T00:00:00Z".to_string(),
            detail: Some("mocked apptainer submission recorded for governed monitoring tests".to_string()),
        }],
    };
    let slurm_expected =
        include_str!("../../fixtures/runtime_schema/default/slurm_submission_v1.json");
    let slurm_actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&slurm)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(slurm_actual.trim_end(), slurm_expected.trim_end());
}

#[test]
fn schema_fixture_names_include_version() {
    let dir = support::crate_root("bijux-dna-runtime")
        .unwrap_or_else(|err| panic!("resolve runtime crate root: {err}"))
        .join("tests")
        .join("fixtures")
        .join("runtime_schema")
        .join("default");
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&dir)
        .unwrap_or_else(|err| panic!("read runtime_schema fixtures at {}: {err}", dir.display()))
    {
        let entry = entry.unwrap_or_else(|err| panic!("fixture entry: {err}"));
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_else(|| panic!("fixture name is not valid UTF-8: {}", path.display()));
        if !std::path::Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            continue;
        }
        if name == "CASE.json" {
            continue;
        }
        if name == "artifact_inventory_v0.json" {
            continue;
        }
        if !name.ends_with("_v1.json") {
            offenders.push(name.to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "runtime schema fixtures must use *_v1.json names: {offenders:?}"
    );
}
