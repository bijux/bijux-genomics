use super::Result;
use anyhow::anyhow;
use bijux_dna_core::contract::ExecutionGraph;
use bijux_dna_engine::Engine;
use bijux_dna_runner::DockerRunner;
use std::path::Path;

/// Replay or verify a run from a run manifest.
///
/// # Errors
/// Returns an error if manifest parsing, graph loading, execution, or verification fails.
pub fn replay_manifest(manifest_path: &Path, verify_only: bool) -> Result<()> {
    let raw = std::fs::read_to_string(manifest_path)
        .map_err(|err| anyhow!("read run_manifest.json: {err}"))?;
    let manifest: serde_json::Value =
        serde_json::from_str(&raw).map_err(|err| anyhow!("parse run_manifest.json: {err}"))?;
    let base_dir =
        manifest_path.parent().ok_or_else(|| anyhow!("run_manifest.json missing parent"))?;
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
    let layout = bijux_dna_runtime::run_layout::RunLayout {
        run_dir: base_dir.to_path_buf(),
        stages_dir: base_dir.join("stages"),
        summary_dir: base_dir.join("summary"),
        assessment_path: base_dir.join("input_assessment.json"),
        manifest_path: base_dir.join("execution_manifest.json"),
        environment_path: base_dir.join("environment.json"),
        metadata_path: base_dir.join("run_metadata.json"),
        events_path: base_dir.join("events.jsonl"),
    };
    Engine::default().execute(&graph, &runner, &layout, None, None)?;
    Ok(())
}
