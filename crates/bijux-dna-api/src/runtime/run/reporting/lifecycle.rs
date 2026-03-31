use super::{anyhow, DockerRunner, Path, PathBuf, Result};
use crate::request_args::RunStatus;
use bijux_dna_core::contract::ExecutionGraph;
use bijux_dna_engine::Engine;
use chrono::Utc;

/// # Errors
/// Returns an error if run status inspection fails.
pub(super) fn status(run_dir: &Path) -> Result<RunStatus> {
    let manifest_path = run_dir.join("run_manifest.json");
    let run_artifacts = bijux_dna_runtime::recording::run_artifacts_dir_for_out(run_dir);
    let envelope_path = run_artifacts.join("run_artifact_envelope.json");
    let manifest = if envelope_path.exists() {
        std::fs::read_to_string(&envelope_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
            .and_then(|value| {
                value
                    .get("manifest_json")
                    .and_then(serde_json::Value::as_str)
                    .map(PathBuf::from)
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
    Ok(RunStatus {
        run_dir: run_dir.to_path_buf(),
        manifest_path: manifest,
        report_path: report,
        has_failures,
    })
}

/// Replay or verify a run from a run manifest.
///
/// # Errors
/// Returns an error if manifest parsing, graph loading, execution, or verification fails.
pub(super) fn replay_manifest(manifest_path: &Path, verify_only: bool) -> Result<()> {
    let raw = std::fs::read_to_string(manifest_path)
        .map_err(|err| anyhow!("read run_manifest.json: {err}"))?;
    let manifest: serde_json::Value =
        serde_json::from_str(&raw).map_err(|err| anyhow!("parse run_manifest.json: {err}"))?;
    let base_dir = manifest_path
        .parent()
        .ok_or_else(|| anyhow!("run_manifest.json missing parent"))?;
    let artifacts = manifest
        .get("output_artifacts")
        .and_then(|v| v.as_array())
        .cloned()
        .ok_or_else(|| anyhow!("run_manifest.json missing output_artifacts array"))?;
    if verify_only {
        for entry in artifacts {
            let Some(path_value) = entry.get("path") else {
                continue;
            };
            let Some(path_str) = path_value.as_str() else {
                continue;
            };
            let path = base_dir.join(path_str);
            if !path.exists() {
                return Err(anyhow!("missing output artifact {}", path.display()));
            }
            if let Some(expected) = entry.get("sha256").and_then(|v| v.as_str()) {
                let actual = bijux_dna_infra::hash_file_sha256(&path)?;
                if actual != expected {
                    return Err(anyhow!(
                        "artifact hash mismatch for {} (expected {}, got {})",
                        path.display(),
                        expected,
                        actual
                    ));
                }
            }
        }
        return Ok(());
    }
    let graph_path =
        bijux_dna_runtime::recording::run_artifacts_dir_for_out(base_dir).join("graph.json");
    let graph_raw =
        std::fs::read_to_string(&graph_path).map_err(|err| anyhow!("read graph.json: {err}"))?;
    let graph: ExecutionGraph =
        serde_json::from_str(&graph_raw).map_err(|err| anyhow!("parse graph.json: {err}"))?;
    let runner = DockerRunner::new(None);
    let layout = run_layout_from_dir(base_dir);
    Engine::default().execute(&graph, &runner, &layout, None, None)?;
    Ok(())
}

pub(super) fn write_run_summary_artifact(
    path: &Path,
    mode: &str,
    pipeline_id: &str,
    manifest_path: &Path,
) -> Result<()> {
    let payload = serde_json::json!({
        "schema_version": "bijux.run_summary.v1",
        "mode": mode,
        "pipeline_id": pipeline_id,
        "manifest_path": relative_path_string(
            manifest_path.parent().unwrap_or_else(|| Path::new(".")),
            manifest_path,
        ),
        "generated_at": Utc::now().to_rfc3339(),
    });
    let bytes = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&payload)?;
    bijux_dna_infra::atomic_write_bytes(path, bytes.as_slice())?;
    Ok(())
}

pub(super) fn relative_path_string(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn run_layout_from_dir(base_dir: &Path) -> bijux_dna_runtime::run_layout::RunLayout {
    bijux_dna_runtime::run_layout::RunLayout {
        run_dir: base_dir.to_path_buf(),
        stages_dir: base_dir.join("stages"),
        summary_dir: base_dir.join("summary"),
        assessment_path: base_dir.join("input_assessment.json"),
        manifest_path: base_dir.join("execution_manifest.json"),
        environment_path: base_dir.join("environment.json"),
        metadata_path: base_dir.join("run_metadata.json"),
        events_path: base_dir.join("events.jsonl"),
    }
}
