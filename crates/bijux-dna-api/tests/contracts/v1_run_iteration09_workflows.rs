use anyhow::Result;
use bijux_dna_api::v1::api::run::{
    execute_local_bam_workflow, execute_local_fastq_workflow, execute_local_vcf_workflow,
    replay_manifest,
};

fn assert_governed_run_bundle(response: &bijux_dna_api::v1::api::run::ExecuteResponse) -> Result<()> {
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
    replay_manifest(&response.manifest_path, true)?;
    Ok(())
}

#[test]
fn local_fastq_workflow_runs_end_to_end_with_governed_artifacts() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let response = execute_local_fastq_workflow(temp.path())?;
    assert_eq!(response.mode, bijux_dna_runtime::run_layout::RunExecutionModeV1::Enforced);
    assert_eq!(response.state, bijux_dna_runtime::run_layout::RunLifecycleStateV1::Succeeded);
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
