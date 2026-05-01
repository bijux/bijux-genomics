use anyhow::Result;
use bijux_dna_core::contract::canonical::to_canonical_json_bytes;
use bijux_dna_core::contract::RunMetadataV1;
use chrono::{DateTime, Utc};

use crate::recording::write_canonical_json;

use super::{
    ArtifactInventoryV1, HashLedgerV1, OperatorHealthReportV1, ReplayManifestV1,
    RunBackendRecordV1, RunCheckpointV1, RunControlStateV1, RunEnvironment,
    RunExecutorDescriptorV1, RunFailureV1, RunLayout, RunLeaseV1, RunManifest, RunQueueStateV1,
    RunSchedulingDecisionV1, RunStateV1, RuntimePolicyV1, SlurmSubmissionRecordV1,
};

/// Write the environment fingerprint.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_environment(layout: &RunLayout, env: &RunEnvironment) -> Result<()> {
    write_canonical_json(&layout.environment_path, env)?;
    Ok(())
}

/// Write run metadata.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_run_metadata(layout: &RunLayout, metadata: &RunMetadataV1) -> Result<()> {
    write_canonical_json(&layout.metadata_path, metadata)?;
    Ok(())
}

/// Write the run manifest.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_manifest(layout: &RunLayout, manifest: &RunManifest) -> Result<()> {
    let payload = to_canonical_json_bytes(manifest)?;
    bijux_dna_infra::atomic_write_bytes(&layout.manifest_path, payload.as_slice())?;
    Ok(())
}

/// Write the governed run-state contract.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_run_state(layout: &RunLayout, run_state: &RunStateV1) -> Result<()> {
    write_canonical_json(&layout.run_state_path, run_state)?;
    Ok(())
}

/// Write the governed executor descriptor.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_executor_descriptor(
    layout: &RunLayout,
    descriptor: &RunExecutorDescriptorV1,
) -> Result<()> {
    write_canonical_json(&layout.executor_descriptor_path, descriptor)?;
    Ok(())
}

/// Write the governed backend descriptor snapshot.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_backend_descriptor(layout: &RunLayout, descriptor: &RunBackendRecordV1) -> Result<()> {
    write_canonical_json(&layout.backend_descriptor_path, descriptor)?;
    Ok(())
}

/// Write the governed scheduling decision.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_scheduling_decision(
    layout: &RunLayout,
    decision: &RunSchedulingDecisionV1,
) -> Result<()> {
    write_canonical_json(&layout.scheduling_decision_path, decision)?;
    Ok(())
}

/// Write the durable queue-state record.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_queue_state(layout: &RunLayout, state: &RunQueueStateV1) -> Result<()> {
    write_canonical_json(&layout.queue_state_path, state)?;
    Ok(())
}

/// Write the exclusive run lease snapshot.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_lease(layout: &RunLayout, lease: &RunLeaseV1) -> Result<()> {
    write_canonical_json(&layout.lease_path, lease)?;
    Ok(())
}

/// Write the auditable control-state record.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_control_state(layout: &RunLayout, control: &RunControlStateV1) -> Result<()> {
    write_canonical_json(&layout.control_state_path, control)?;
    Ok(())
}

/// Write the operator health report.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_health_report(layout: &RunLayout, report: &OperatorHealthReportV1) -> Result<()> {
    write_canonical_json(&layout.health_report_path, report)?;
    Ok(())
}

/// Write the mocked Slurm submission record.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_slurm_submission(
    layout: &RunLayout,
    submission: &SlurmSubmissionRecordV1,
) -> Result<()> {
    write_canonical_json(&layout.slurm_submission_path, submission)?;
    Ok(())
}

/// Write the governed runtime policy snapshot.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_runtime_policy(layout: &RunLayout, policy: &RuntimePolicyV1) -> Result<()> {
    write_canonical_json(&layout.runtime_policy_path, policy)?;
    Ok(())
}

/// Write the governed checkpoint snapshot.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_checkpoint(layout: &RunLayout, checkpoint: &RunCheckpointV1) -> Result<()> {
    write_canonical_json(&layout.checkpoint_path, checkpoint)?;
    Ok(())
}

/// Write the structured run failure record.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_failure_record(layout: &RunLayout, failure: &RunFailureV1) -> Result<()> {
    write_canonical_json(&layout.failure_path, failure)?;
    Ok(())
}

/// Write the governed artifact inventory.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_artifact_inventory(layout: &RunLayout, inventory: &ArtifactInventoryV1) -> Result<()> {
    write_canonical_json(&layout.artifact_inventory_path, inventory)?;
    Ok(())
}

/// Write the governed replay manifest.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_replay_manifest(layout: &RunLayout, manifest: &ReplayManifestV1) -> Result<()> {
    write_canonical_json(&layout.replay_manifest_path, manifest)?;
    Ok(())
}

/// Write the tamper-evident hash ledger.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_hash_ledger(layout: &RunLayout, ledger: &HashLedgerV1) -> Result<()> {
    write_canonical_json(&layout.hash_ledger_path, ledger)?;
    Ok(())
}

#[must_use]
pub fn now_string() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.to_rfc3339()
}
