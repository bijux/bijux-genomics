use anyhow::Result;
use bijux_dna_api::v1::api::{
    browse_runs, evidence_gap, operator_diagnosis, query_run_lineage, status,
    EvidenceGapRequestV1, OperatorDiagnosisRequestV1, RunBrowserFilterV1, RunBrowserRequestV1,
    RunLineageQueryRequestV1,
};

#[test]
fn status_discovers_evidence_bundle_and_correlation() -> Result<()> {
    let temp = tempfile::tempdir()?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("run_manifest.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_manifest.v3",
            "run_id": "run-1",
            "correlation_id": "corr-run-1",
            "failures": []
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("evidence_bundle.json"),
        &serde_json::json!({
            "schema_version": "bijux.evidence_bundle.v1"
        }),
    )?;

    let snapshot = status(temp.path())?;
    assert_eq!(snapshot.correlation_id.as_deref(), Some("corr-run-1"));
    assert_eq!(
        snapshot
            .evidence_bundle_path
            .as_deref()
            .map(std::path::Path::file_name)
            .and_then(|value| value.and_then(|value| value.to_str())),
        Some("evidence_bundle.json")
    );
    assert!(snapshot.evidence_verification_path.is_none());
    Ok(())
}

#[test]
fn status_reads_governed_run_state_and_failure_paths() -> Result<()> {
    let temp = tempfile::tempdir()?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("run_manifest.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_manifest.v3",
            "run_id": "run-2",
            "correlation_id": "corr-run-2",
            "failures": [{"path": "run_failure.json"}]
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("run_state.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_state.v1",
            "run_id": "run-2",
            "mode": "simulation",
            "state": "succeeded",
            "transitions": [],
            "manifest_path": "run_manifest.json",
            "checkpoint_path": "checkpoints/checkpoint.json",
            "failure_path": null
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("runtime_policy.json"),
        &serde_json::json!({
            "schema_version": "bijux.runtime_policy.v1"
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("executor_descriptor.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_executor_descriptor.v1"
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("backend_descriptor.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_backend.v1"
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("scheduling_decision.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_scheduling_decision.v1"
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("queue_state.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_queue_state.v1"
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("run_lease.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_lease.v1"
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("run_control.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_control.v1"
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("operator_health.json"),
        &serde_json::json!({
            "schema_version": "bijux.operator_health.v1"
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("slurm_submission.json"),
        &serde_json::json!({
            "schema_version": "bijux.slurm_submission.v1"
        }),
    )?;
    std::fs::create_dir_all(temp.path().join("checkpoints"))?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("checkpoints/checkpoint.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_checkpoint.v1"
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("run_failure.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_failure.v1"
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("artifact_inventory.json"),
        &serde_json::json!({
            "schema_version": "bijux.artifact_inventory.v1",
            "run_id": "run-2",
            "artifacts": []
        }),
    )?;
    std::fs::write(temp.path().join("artifact_inventory.txt"), b"artifact inventory\n")?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("replay_manifest.json"),
        &serde_json::json!({
            "schema_version": "bijux.replay_manifest.v1",
            "replay_run_id": "run-2",
            "original_run_id": "run-2"
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("hash_ledger.json"),
        &serde_json::json!({
            "schema_version": "bijux.hash_ledger.v1",
            "run_id": "run-2",
            "root_sha256": "abc",
            "entries": []
        }),
    )?;
    std::fs::create_dir_all(temp.path().join("summary"))?;
    std::fs::write(temp.path().join("summary").join("run_summary.txt"), b"summary\n")?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("evidence_verification.json"),
        &serde_json::json!({
            "schema_version": "bijux.evidence_verification.v1",
            "verified": true,
            "checks": [],
            "missing_paths": [],
            "gap_count": 0
        }),
    )?;

    let snapshot = status(temp.path())?;
    assert_eq!(
        snapshot.mode,
        Some(bijux_dna_runtime::run_layout::RunExecutionModeV1::Simulation)
    );
    assert_eq!(
        snapshot.state,
        Some(bijux_dna_runtime::run_layout::RunLifecycleStateV1::Succeeded)
    );
    assert!(snapshot.runtime_policy_path.is_some());
    assert!(snapshot.executor_descriptor_path.is_some());
    assert!(snapshot.backend_descriptor_path.is_some());
    assert!(snapshot.scheduling_decision_path.is_some());
    assert!(snapshot.queue_state_path.is_some());
    assert!(snapshot.lease_path.is_some());
    assert!(snapshot.control_state_path.is_some());
    assert!(snapshot.health_report_path.is_some());
    assert!(snapshot.slurm_submission_path.is_some());
    assert!(snapshot.checkpoint_path.is_some());
    assert!(snapshot.failure_path.is_some());
    assert!(snapshot.artifact_inventory_path.is_some());
    assert!(snapshot.artifact_inventory_text_path.is_some());
    assert!(snapshot.replay_manifest_path.is_some());
    assert!(snapshot.hash_ledger_path.is_some());
    assert!(snapshot.run_summary_text_path.is_some());
    assert!(snapshot.evidence_verification_path.is_some());
    assert!(snapshot.has_failures);
    Ok(())
}

#[test]
fn run_browser_lists_run_rows_with_runtime_state() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let run_dir = temp.path().join("run-iteration16");
    std::fs::create_dir_all(&run_dir)?;
    bijux_dna_infra::atomic_write_json(
        &run_dir.join("run_manifest.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_manifest.v3",
            "run_id": "run-iteration16",
            "profile_id": "fastq.default",
            "pipeline_id": "fastq-to-fastq__default__v1",
            "correlation_id": "corr-16",
            "failures": [],
            "output_artifacts": [{"kind": "report", "path": "reports/report.json"}]
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &run_dir.join("run_state.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_state.v1",
            "run_id": "run-iteration16",
            "mode": "enforced",
            "state": "succeeded",
            "transitions": []
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &run_dir.join("artifact_inventory.json"),
        &serde_json::json!({
            "schema_version": "bijux.artifact_inventory.v1",
            "run_id": "run-iteration16",
            "artifacts": [{"artifact_id": "report", "name": "report", "role": "report", "path": "reports/report.json", "input_lineage": []}]
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &run_dir.join("evidence_bundle.json"),
        &serde_json::json!({"schema_version": "bijux.evidence_bundle.v1"}),
    )?;

    let response = browse_runs(&RunBrowserRequestV1 {
        runs_root: temp.path().to_path_buf(),
        page_size: 0,
        page_token: None,
        filter: RunBrowserFilterV1::default(),
    })?;

    assert_eq!(response.schema_version, "bijux.run_browser.v1");
    assert_eq!(response.total_rows, 1);
    let row = response.rows.first().unwrap_or_else(|| panic!("missing row"));
    assert_eq!(row.run_id, "run-iteration16");
    assert_eq!(row.profile_id.as_deref(), Some("fastq.default"));
    assert_eq!(row.pipeline_id.as_deref(), Some("fastq-to-fastq__default__v1"));
    assert_eq!(
        row.state,
        Some(bijux_dna_runtime::run_layout::RunLifecycleStateV1::Succeeded)
    );
    assert!(row.has_evidence_bundle);
    assert_eq!(row.artifact_count, 1);
    Ok(())
}

#[test]
fn run_lineage_query_extracts_artifact_lineage_edges() -> Result<()> {
    let temp = tempfile::tempdir()?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("artifact_inventory.json"),
        &serde_json::json!({
            "schema_version": "bijux.artifact_inventory.v1",
            "run_id": "run-lineage-16",
            "artifacts": [
                {
                    "artifact_id": "filtered_fastq",
                    "name": "filtered_fastq",
                    "role": "reads",
                    "path": "stages/trim/filtered.fastq.gz",
                    "producing_stage_id": "fastq.trim_reads",
                    "input_lineage": [
                        "raw:R1=abc",
                        "raw:R2=def"
                    ]
                }
            ]
        }),
    )?;

    let response = query_run_lineage(&RunLineageQueryRequestV1 {
        run_dir: temp.path().to_path_buf(),
        artifact_id: Some("filtered_fastq".to_string()),
    })?;

    assert_eq!(response.schema_version, "bijux.run_lineage_query.v1");
    assert_eq!(response.run_id, "run-lineage-16");
    assert_eq!(response.total_artifacts, 1);
    assert_eq!(response.edges.len(), 2);
    assert!(response.edges.iter().any(|edge| edge.lineage_key == "raw:R1=abc"));
    assert!(response.edges.iter().any(|edge| edge.lineage_key == "raw:R2=def"));
    Ok(())
}

#[test]
fn evidence_gap_reports_missing_paths_failed_checks_and_unsafe_artifacts() -> Result<()> {
    let temp = tempfile::tempdir()?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("evidence_verification.json"),
        &serde_json::json!({
            "schema_version": "bijux.evidence_verification.v1",
            "verified": false,
            "checks": [
                {"check_id": "artifact_integrity", "ok": false, "message": "artifact mismatch"}
            ],
            "missing_paths": ["reports/report.json"],
            "gap_count": 1
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("artifact_inventory.json"),
        &serde_json::json!({
            "schema_version": "bijux.artifact_inventory.v1",
            "run_id": "gap-run",
            "artifacts": [
                {
                    "artifact_id": "runtime_policy",
                    "name": "runtime_policy",
                    "role": "runtime_policy",
                    "path": "runtime_policy.json",
                    "input_lineage": [],
                    "scientific_context": {"domain": "runtime", "meaning": "policy", "safe_to_use": true, "advisory_only": true}
                },
                {
                    "artifact_id": "run_failure",
                    "name": "run_failure",
                    "role": "run_failure",
                    "path": "run_failure.json",
                    "input_lineage": [],
                    "scientific_context": {"domain": "runtime", "meaning": "failure", "safe_to_use": false, "advisory_only": false}
                }
            ]
        }),
    )?;

    let response = evidence_gap(&EvidenceGapRequestV1 {
        run_dir: temp.path().to_path_buf(),
    })?;

    assert_eq!(response.schema_version, "bijux.evidence_gap.v1");
    assert!(!response.verified);
    assert!(response.gap_count >= 3);
    assert!(response
        .missing_paths
        .iter()
        .any(|path| path == "reports/report.json"));
    assert!(response
        .failed_checks
        .iter()
        .any(|check| check.check_id == "artifact_integrity"));
    assert!(response
        .advisory_only_artifacts
        .iter()
        .any(|id| id == "runtime_policy"));
    assert!(response.unsafe_artifacts.iter().any(|id| id == "run_failure"));
    Ok(())
}

#[test]
fn operator_diagnosis_reports_commands_and_runtime_signals() -> Result<()> {
    let temp = tempfile::tempdir()?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("run_state.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_state.v1",
            "run_id": "diag-run",
            "mode": "enforced",
            "state": "running",
            "transitions": []
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("queue_state.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_queue_state.v1",
            "run_id": "diag-run",
            "dedup_key": "dedup-1",
            "state": "running",
            "transitions": []
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("run_control.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_control.v1",
            "run_id": "diag-run",
            "requested_action": "pause",
            "observed_state": "running",
            "updated_at": "2026-04-30T00:00:00Z",
            "audit_log": []
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("operator_health.json"),
        &serde_json::json!({
            "schema_version": "bijux.operator_health.v1",
            "run_id": "diag-run",
            "overall_ok": true,
            "checks": []
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("run_failure.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_failure.v1",
            "run_id": "diag-run",
            "mode": "enforced",
            "state": "failed",
            "failure_code": "timeout",
            "message": "timeout",
            "observed_at": "2026-04-30T00:00:00Z",
            "retryable": true
        }),
    )?;

    let response = operator_diagnosis(&OperatorDiagnosisRequestV1 {
        run_dir: temp.path().to_path_buf(),
    })?;

    assert_eq!(response.schema_version, "bijux.operator_diagnosis.v1");
    assert_eq!(response.run_id, "diag-run");
    assert_eq!(
        response.queue_state,
        Some(bijux_dna_runtime::run_layout::RunQueueLifecycleStateV1::Running)
    );
    assert_eq!(
        response.requested_action,
        Some(bijux_dna_runtime::run_layout::RunControlActionV1::Pause)
    );
    assert!(response.health_ok);
    assert!(response.has_failure_record);
    assert!(response
        .commands
        .iter()
        .any(|command| command.command_id == "inspect_failure_record"));
    Ok(())
}
