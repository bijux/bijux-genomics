use anyhow::Result;
use bijux_dna_api::v1::api::status;

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
