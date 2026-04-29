use super::{Path, Result};
use bijux_dna_core::contract::ExecutionGraph;
use chrono::Utc;

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
    path.strip_prefix(base).unwrap_or(path).to_string_lossy().to_string()
}

/// # Errors
/// Returns an error if the manifest cannot be updated deterministically.
pub(super) fn attach_output_artifact(
    manifest_path: &Path,
    base_dir: &Path,
    correlation_id: &str,
    name: &str,
    schema: &str,
    artifact_path: &Path,
) -> Result<()> {
    let raw = std::fs::read_to_string(manifest_path)?;
    let mut manifest: serde_json::Value = serde_json::from_str(&raw)?;
    manifest["correlation_id"] = serde_json::Value::String(correlation_id.to_string());
    if !manifest["output_artifacts"].is_array() {
        manifest["output_artifacts"] = serde_json::Value::Array(Vec::new());
    }
    let artifact = serde_json::json!({
        "name": name,
        "kind": name,
        "schema": schema,
        "path": relative_path_string(base_dir, artifact_path),
        "sha256": bijux_dna_infra::hash_file_sha256(artifact_path)?,
    });
    if let Some(entries) = manifest["output_artifacts"].as_array_mut() {
        let artifact_path = artifact["path"].as_str().unwrap_or_default();
        entries.retain(|entry| entry.get("path").and_then(serde_json::Value::as_str) != Some(artifact_path));
        entries.push(artifact);
    }
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)?;
    bijux_dna_infra::atomic_write_bytes(manifest_path, payload.as_slice())?;
    Ok(())
}

pub(super) fn planned_stage_manifest(graph: &ExecutionGraph) -> serde_json::Value {
    serde_json::Value::Array(
        graph
            .steps()
            .iter()
            .map(|step| {
                serde_json::json!({
                    "step_id": step.step_id.to_string(),
                    "stage_id": step.stage_id.to_string(),
                    "image": step.image.image,
                    "image_digest": step.image.digest,
                    "out_dir": step.out_dir,
                    "inputs": step
                        .io
                        .inputs
                        .iter()
                        .map(|artifact| {
                            serde_json::json!({
                                "name": artifact.name.to_string(),
                                "path": artifact.path,
                                "role": format!("{:?}", artifact.role),
                                "optional": artifact.optional,
                            })
                        })
                        .collect::<Vec<_>>(),
                    "outputs": step
                        .io
                        .outputs
                        .iter()
                        .map(|artifact| {
                            serde_json::json!({
                                "name": artifact.name.to_string(),
                                "path": artifact.path,
                                "role": format!("{:?}", artifact.role),
                                "optional": artifact.optional,
                            })
                        })
                        .collect::<Vec<_>>(),
                })
            })
            .collect(),
    )
}
