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
