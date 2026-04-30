use super::{Path, Result};
use crate::request_args::RunStatus;

/// # Errors
/// This wrapper preserves the public API shape and does not currently return an error.
#[allow(clippy::unnecessary_wraps)]
pub fn status(run_dir: &Path) -> Result<RunStatus> {
    let mut status = super::lifecycle::status(run_dir);
    let layout = bijux_dna_runtime::run_layout::RunLayout::from_run_dir(run_dir.to_path_buf());
    let run_state = layout
        .run_state_path
        .exists()
        .then(|| std::fs::read_to_string(&layout.run_state_path).ok())
        .flatten()
        .and_then(|raw| serde_json::from_str::<bijux_dna_runtime::run_layout::RunStateV1>(&raw).ok());
    let correlation_id = status
        .manifest_path
        .as_ref()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .and_then(|manifest| {
            manifest.get("correlation_id").and_then(serde_json::Value::as_str).map(str::to_string)
        });

    status.evidence_bundle_path = layout.evidence_bundle_path.exists().then_some(layout.evidence_bundle_path);
    status.run_state_path = layout.run_state_path.exists().then_some(layout.run_state_path);
    status.runtime_policy_path =
        layout.runtime_policy_path.exists().then_some(layout.runtime_policy_path);
    status.executor_descriptor_path = layout
        .executor_descriptor_path
        .exists()
        .then_some(layout.executor_descriptor_path);
    status.checkpoint_path = layout.checkpoint_path.exists().then_some(layout.checkpoint_path);
    status.failure_path = layout.failure_path.exists().then_some(layout.failure_path);
    status.correlation_id = correlation_id;
    status.mode = run_state.as_ref().map(|state| state.mode);
    status.state = run_state.as_ref().map(|state| state.state);
    status.has_failures |= status.failure_path.is_some();
    Ok(status)
}
