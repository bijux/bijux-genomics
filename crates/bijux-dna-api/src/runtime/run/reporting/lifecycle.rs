use crate::request_args::RunStatus;
use std::path::{Path, PathBuf};

pub(super) fn status(run_dir: &Path) -> RunStatus {
    let manifest_path = run_dir.join("run_manifest.json");
    let run_artifacts = bijux_dna_runtime::recording::run_artifacts_dir_for_out(run_dir);
    let envelope_path = run_artifacts.join("run_artifact_envelope.json");
    let manifest = if envelope_path.exists() {
        std::fs::read_to_string(&envelope_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
            .and_then(|value| {
                value.get("manifest_json").and_then(serde_json::Value::as_str).map(PathBuf::from)
            })
            .or_else(|| manifest_path.exists().then_some(manifest_path.clone()))
    } else if manifest_path.exists() {
        Some(manifest_path.clone())
    } else {
        None
    };
    let report_path = run_artifacts.join("report.html");
    let report = report_path.exists().then_some(report_path);
    let has_failures = manifest
        .as_ref()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .and_then(|value| value.get("failures").cloned())
        .and_then(|value| value.as_array().cloned())
        .is_some_and(|failures| !failures.is_empty());
    RunStatus {
        run_dir: run_dir.to_path_buf(),
        manifest_path: manifest,
        report_path: report,
        evidence_bundle_path: None,
        run_state_path: None,
        runtime_policy_path: None,
        executor_descriptor_path: None,
        checkpoint_path: None,
        failure_path: None,
        correlation_id: None,
        mode: None,
        state: None,
        has_failures,
    }
}
