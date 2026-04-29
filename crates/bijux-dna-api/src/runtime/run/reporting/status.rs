use super::{Path, Result};
use crate::request_args::RunStatus;

/// # Errors
/// This wrapper preserves the public API shape and does not currently return an error.
#[allow(clippy::unnecessary_wraps)]
pub fn status(run_dir: &Path) -> Result<RunStatus> {
    let status = super::lifecycle::status(run_dir);
    let evidence_bundle_path = run_dir.join("evidence_bundle.json");
    let correlation_id = status
        .manifest_path
        .as_ref()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .and_then(|manifest| manifest.get("correlation_id").and_then(serde_json::Value::as_str).map(str::to_string));
    Ok(RunStatus {
        evidence_bundle_path: evidence_bundle_path.exists().then_some(evidence_bundle_path),
        correlation_id,
        ..status
    })
}
