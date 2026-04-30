use anyhow::Result;
use bijux_dna_api::v1::api::run::{
    environment_identity, execute_local_bam_workflow, execute_local_fastq_workflow,
    execute_local_vcf_workflow, replay_manifest,
};

fn assert_governed_run_bundle(response: &bijux_dna_api::v1::api::run::ExecuteResponse) -> Result<()> {
    let run_dir = response
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("manifest path missing parent"))?;
    assert!(response.manifest_path.exists());
    assert!(response.run_state_path.exists());
    assert!(response.runtime_policy_path.exists());
    assert!(response.executor_descriptor_path.exists());
    assert!(response.backend_descriptor_path.exists());
    assert!(response.scheduling_decision_path.exists());
    assert!(response.queue_state_path.exists());
    assert!(response.lease_path.exists());
    assert!(response.control_state_path.exists());
    assert!(response.checkpoint_path.exists());
    assert!(response.health_report_path.exists());
    assert!(response.evidence_bundle_path.exists());
    assert!(response.evidence_verification_path.exists());
    assert!(response.artifact_inventory_path.exists());
    assert!(response.replay_manifest_path.exists());
    assert!(response.hash_ledger_path.exists());
    assert!(response.run_summary_text_path.exists());
    assert!(run_dir.join("environment.json").exists());
    replay_manifest(&response.manifest_path, true)?;
    Ok(())
}

#[test]
fn local_fastq_workflow_runs_end_to_end_with_governed_artifacts() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let response = execute_local_fastq_workflow(temp.path())?;
    assert_eq!(response.mode, bijux_dna_runtime::run_layout::RunExecutionModeV1::Enforced);
    assert_eq!(response.state, bijux_dna_runtime::run_layout::RunLifecycleStateV1::Succeeded);
    assert!(temp.path().join("out/validate_reads/run_artifacts/command.txt").exists());
    assert!(temp.path().join("out/validate_reads/run_artifacts/stdout.log").exists());
    assert!(temp.path().join("out/validate_reads/run_artifacts/stderr.log").exists());
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&response.manifest_path)?)?;
    let invocations = manifest
        .get("tool_invocations")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(!invocations.is_empty(), "run manifest must include tool invocations");
    let first = &invocations[0];
    assert!(first.get("executed_command").is_some());
    assert!(first.get("stage_id").is_some());
    assert!(
        first
            .get("environment")
            .and_then(serde_json::Value::as_object)
            .is_some_and(|environment| environment.contains_key("working_directory"))
    );
    let run_dir = response
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("manifest path missing parent"))?;
    let environment: serde_json::Value =
        serde_json::from_slice(&std::fs::read(run_dir.join("environment.json"))?)?;
    assert!(environment.get("os").is_some());
    assert!(environment.get("arch").is_some());
    assert!(environment.get("runner").is_some());
    assert!(environment.get("tool_images").is_some());
    let environment_identity = environment_identity(run_dir)?;
    let stage_environments = environment_identity
        .get("stage_environments")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        environment_identity
            .get("schema_version")
            .and_then(serde_json::Value::as_str),
        Some("bijux.run_environment_identity.v1")
    );
    if !stage_environments.is_empty() {
        assert!(stage_environments.iter().any(|entry| {
            entry
                .get("environment")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|environment| environment.contains_key("working_directory"))
        }));
    }
    assert_governed_run_bundle(&response)?;
    Ok(())
}

#[test]
fn local_bam_workflow_runs_end_to_end_with_governed_artifacts() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let response = execute_local_bam_workflow(temp.path())?;
    assert_eq!(response.mode, bijux_dna_runtime::run_layout::RunExecutionModeV1::Enforced);
    assert_eq!(response.state, bijux_dna_runtime::run_layout::RunLifecycleStateV1::Succeeded);
    assert_governed_run_bundle(&response)?;
    Ok(())
}

#[test]
fn local_vcf_workflow_runs_end_to_end_with_governed_artifacts() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let response = execute_local_vcf_workflow(temp.path())?;
    assert_eq!(response.mode, bijux_dna_runtime::run_layout::RunExecutionModeV1::Enforced);
    assert_eq!(response.state, bijux_dna_runtime::run_layout::RunLifecycleStateV1::Succeeded);
    assert!(temp.path().join("out").join("report.json").exists());
    assert_governed_run_bundle(&response)?;
    Ok(())
}
