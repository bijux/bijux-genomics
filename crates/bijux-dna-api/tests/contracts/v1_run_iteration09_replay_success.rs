use anyhow::{anyhow, Result};
use bijux_dna_api::v1::api::run::{
    execute_local_fastq_workflow, explain_successful_replay, replay_explain,
    ReplayExplainRequestV1,
};

#[test]
fn replay_success_explain_reports_reuse_and_drift_sets() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let original = execute_local_fastq_workflow(&temp.path().join("original"))?;
    let replay = execute_local_fastq_workflow(&temp.path().join("replay"))?;
    let original_run_dir = original
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow!("original manifest missing parent"))?;
    let replay_run_dir = replay
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow!("replay manifest missing parent"))?;

    let replay_manifest_path = replay_run_dir.join("replay_manifest.json");
    let mut replay_manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&replay_manifest_path)?)?;
    replay_manifest["original_run_id"] = serde_json::Value::String(original.run_id.clone());
    replay_manifest["reused_artifact_ids"] = serde_json::json!(["report"]);
    bijux_dna_infra::atomic_write_json(&replay_manifest_path, &replay_manifest)?;

    let explain = explain_successful_replay(original_run_dir, replay_run_dir)?;
    assert_eq!(
        explain
            .get("schema_version")
            .and_then(serde_json::Value::as_str),
        Some("bijux.replay_success_explain.v1")
    );
    assert!(
        explain
            .get("rerun_stage_ids")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|entries| !entries.is_empty())
    );
    assert!(
        explain
            .get("reused_outputs")
            .and_then(serde_json::Value::as_array)
            .is_some()
    );
    assert!(
        explain
            .get("unchanged_outputs")
            .and_then(serde_json::Value::as_array)
            .is_some()
    );
    Ok(())
}

#[test]
fn replay_explain_typed_api_surfaces_stable_contract() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let original = execute_local_fastq_workflow(&temp.path().join("original"))?;
    let replay = execute_local_fastq_workflow(&temp.path().join("replay"))?;
    let response = replay_explain(&ReplayExplainRequestV1 {
        original_run_dir: original
            .manifest_path
            .parent()
            .ok_or_else(|| anyhow!("original manifest missing parent"))?
            .to_path_buf(),
        replay_run_dir: replay
            .manifest_path
            .parent()
            .ok_or_else(|| anyhow!("replay manifest missing parent"))?
            .to_path_buf(),
    })?;

    assert_eq!(response.schema_version, "bijux.replay_success_explain.v1");
    assert!(!response.replay_run_id.is_empty());
    assert!(!response.rerun_stage_ids.is_empty());
    Ok(())
}
