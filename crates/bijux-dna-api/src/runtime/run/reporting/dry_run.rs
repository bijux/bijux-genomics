use super::{summary_artifact, Result};
use crate::request_args::{DryRunRequest, DryRunResponse};
use anyhow::anyhow;

/// # Errors
/// Returns an error if dry-run output cannot be written.
pub fn dry_run(request: &DryRunRequest) -> Result<DryRunResponse> {
    bijux_dna_infra::ensure_dir(&request.run_dir)?;
    let graph_hash = request.graph.hash()?;
    let graph_path = request.run_dir.join("graph.json");
    let graph_payload =
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&request.graph)?;
    bijux_dna_infra::atomic_write_bytes(&graph_path, graph_payload.as_slice())?;
    let mut manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v3",
        "contract_version": bijux_dna_core::contract::ContractVersion::v1(),
        "run_id": "dry-run",
        "pipeline_id": request.graph.pipeline_id().to_string(),
        "profile_id": request.profile_id,
        "graph_hash": graph_hash,
        "cache_key": serde_json::Value::Null,
        "toolchain_versions": [],
        "dataset_fingerprints": [],
        "tool_invocations": [],
        "output_artifacts": [],
        "stages": [],
        "failures": [],
    });
    let manifest_path = request.run_dir.join("run_manifest.json");
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)?;
    bijux_dna_infra::atomic_write_bytes(&manifest_path, payload.as_slice())?;
    let summary_path = request.run_dir.join("run_summary.json");
    summary_artifact::write_run_summary_artifact(
        &summary_path,
        "dry-run",
        request.graph.pipeline_id().as_str(),
        &manifest_path,
    )?;
    let graph_sha = bijux_dna_infra::hash_file_sha256(&graph_path)?;
    let summary_sha = bijux_dna_infra::hash_file_sha256(&summary_path)?;
    manifest["output_artifacts"] = serde_json::json!([
        {
            "kind": "graph",
            "schema": "bijux.execution_graph.v1",
            "path": summary_artifact::relative_path_string(&request.run_dir, &graph_path),
            "sha256": graph_sha
        },
        {
            "kind": "run_summary",
            "schema": "bijux.run_summary.v1",
            "path": summary_artifact::relative_path_string(&request.run_dir, &summary_path),
            "sha256": summary_sha
        }
    ]);
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)?;
    bijux_dna_infra::atomic_write_bytes(&manifest_path, payload.as_slice())?;
    if let Some(artifacts) = manifest["output_artifacts"].as_array_mut() {
        artifacts.push(serde_json::json!({
            "kind": "run_manifest",
            "schema": "bijux.run_manifest.v3",
            "path": summary_artifact::relative_path_string(&request.run_dir, &manifest_path),
            "sha256": serde_json::Value::Null
        }));
    } else {
        return Err(anyhow!("dry-run manifest output_artifacts is not an array"));
    }
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)?;
    bijux_dna_infra::atomic_write_bytes(&manifest_path, payload.as_slice())?;
    bijux_dna_runtime::recording::write_profile_and_lock_manifests(&manifest_path)?;
    Ok(DryRunResponse { graph_path, manifest_path })
}
