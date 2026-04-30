use super::Result;
use anyhow::{Context, Result as AnyhowResult};
use bijux_dna_runtime::run_layout::ArtifactInventoryV1;
use std::path::Path;

/// Explain cache key and cache miss reasons between two run bundles.
///
/// # Errors
/// Returns an error if required contracts cannot be loaded.
pub fn explain_cache_hit_miss(
    original_run_dir: &Path,
    replay_run_dir: &Path,
) -> Result<serde_json::Value> {
    let original = fingerprint(original_run_dir)?;
    let replay = fingerprint(replay_run_dir)?;
    let mut miss_reasons = Vec::new();
    if original.manifest_schema != replay.manifest_schema {
        miss_reasons.push(reason("schema_changed", "manifest schema version changed"));
    }
    if original.graph_hash != replay.graph_hash {
        miss_reasons.push(reason("graph_changed", "execution graph hash changed"));
    }
    if original.reference_hash != replay.reference_hash {
        miss_reasons.push(reason("reference_changed", "reference artifact identity changed"));
    }
    if original.backend_sha256 != replay.backend_sha256 {
        miss_reasons.push(reason("backend_changed", "backend descriptor changed"));
    }
    if original.runtime_policy_sha256 != replay.runtime_policy_sha256 {
        miss_reasons.push(reason("policy_changed", "runtime policy changed"));
    }
    if original.environment_sha256 != replay.environment_sha256 {
        miss_reasons.push(reason("environment_changed", "environment identity changed"));
    }
    if original.artifact_inventory_sha256 != replay.artifact_inventory_sha256 {
        miss_reasons.push(reason("artifact_identity_changed", "artifact inventory changed"));
    }
    let status = if miss_reasons.is_empty() { "hit" } else { "miss" };
    Ok(serde_json::json!({
        "schema_version": "bijux.cache_explain.v1",
        "status": status,
        "original_cache_key": original,
        "replay_cache_key": replay,
        "unsafe_miss_reasons": miss_reasons,
    }))
}

#[derive(Debug, Clone, serde::Serialize)]
struct CacheFingerprint {
    manifest_schema: String,
    graph_hash: String,
    reference_hash: String,
    backend_sha256: String,
    runtime_policy_sha256: String,
    environment_sha256: String,
    artifact_inventory_sha256: String,
}

fn fingerprint(run_dir: &Path) -> AnyhowResult<CacheFingerprint> {
    let layout = bijux_dna_runtime::run_layout::RunLayout::from_run_dir(run_dir.to_path_buf());
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&layout.manifest_path)?).context("parse manifest")?;
    let inventory: ArtifactInventoryV1 = serde_json::from_slice(&std::fs::read(&layout.artifact_inventory_path)?)
        .context("parse artifact inventory")?;
    let reference_hash = inventory
        .artifacts
        .iter()
        .filter(|artifact| artifact.role == "reference")
        .filter_map(|artifact| artifact.sha256.clone())
        .collect::<Vec<_>>()
        .join(",");

    Ok(CacheFingerprint {
        manifest_schema: manifest
            .get("schema_version")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        graph_hash: manifest
            .get("graph_hash")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        reference_hash,
        backend_sha256: bijux_dna_infra::hash_file_sha256(&layout.backend_descriptor_path)?,
        runtime_policy_sha256: bijux_dna_infra::hash_file_sha256(&layout.runtime_policy_path)?,
        environment_sha256: bijux_dna_infra::hash_file_sha256(&layout.environment_path)?,
        artifact_inventory_sha256: bijux_dna_infra::hash_file_sha256(&layout.artifact_inventory_path)?,
    })
}

fn reason(reason_code: &str, message: &str) -> serde_json::Value {
    serde_json::json!({
        "reason_code": reason_code,
        "message": message,
    })
}
