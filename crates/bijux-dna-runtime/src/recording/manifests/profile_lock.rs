use std::path::Path;

use anyhow::{anyhow, Context, Result};

use super::manifest_identity::{canonical_sha256, declared_json_array};
use super::{hash_file_sha256, write_canonical_json};

/// # Errors
/// Returns an error if profile or lock manifests cannot be generated.
pub(super) fn write_profile_and_lock_manifests(run_manifest_path: &Path) -> Result<()> {
    let raw = std::fs::read_to_string(run_manifest_path)
        .with_context(|| format!("read {}", run_manifest_path.display()))?;
    let run_manifest: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("parse {}", run_manifest_path.display()))?;
    let stages = declared_json_array(&run_manifest, "stages")?
        .iter()
        .map(|stage| {
            serde_json::json!({
                "stage_id": stage.get("stage_id").cloned().unwrap_or(serde_json::Value::Null),
                "tool_id": stage.get("tool_id").cloned().unwrap_or(serde_json::Value::Null),
                "stage_contract_hash": stage.get("stage_contract_hash").cloned().unwrap_or(serde_json::Value::Null),
            })
        })
        .collect::<Vec<_>>();
    let run_provenance = run_manifest
        .get("run_provenance")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let profile_manifest = serde_json::json!({
        "schema_version": "bijux.profile_manifest.v1",
        "pipeline_id": run_manifest.get("pipeline_id").cloned().unwrap_or(serde_json::Value::Null),
        "profile_id": run_manifest.get("profile_id").cloned().unwrap_or(serde_json::Value::Null),
        "invariants_preset": run_manifest.get("invariants_preset").cloned().unwrap_or(serde_json::Value::Null),
        "stage_list": stages,
        "tool_digests": run_manifest.get("tool_invocations").cloned().unwrap_or_else(|| serde_json::json!([])),
        "param_hashes": run_provenance.get("params_by_stage").cloned().unwrap_or_else(|| serde_json::json!({})),
        "schema_versions": {
            "run_manifest": run_manifest.get("schema_version").cloned().unwrap_or(serde_json::Value::Null),
            "tool_invocation": "bijux.tool_invocation.v1",
            "metrics_envelope": "bijux.metrics_envelope.v2",
        }
    });
    let profile_manifest_hash = canonical_sha256(&profile_manifest)?;
    let run_manifest_hash = hash_file_sha256(run_manifest_path)?;
    let mut resolved_tools = declared_json_array(&run_manifest, "tool_invocations")?
        .iter()
        .map(|inv| {
            serde_json::json!({
                "stage_id": inv.get("stage_id").cloned().unwrap_or(serde_json::Value::Null),
                "tool_id": inv.get("tool_id").cloned().unwrap_or(serde_json::Value::Null),
                "image_digest": inv.get("image_digest").cloned().unwrap_or(serde_json::Value::Null),
                "tool_version": inv.get("tool_version").cloned().unwrap_or(serde_json::Value::Null),
                "resolved_tool_version": inv.get("resolved_tool_version").cloned().unwrap_or(serde_json::Value::Null),
            })
        })
        .collect::<Vec<_>>();
    resolved_tools.sort_by_key(std::string::ToString::to_string);
    let lock_manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.lock.v1",
        "run_manifest": {
            "path": "run_manifest.json",
            "sha256": run_manifest_hash,
        },
        "profile_manifest": {
            "path": "profile_manifest.json",
            "sha256": profile_manifest_hash,
        },
        "resolved_tool_digests": resolved_tools,
    });
    let run_dir = run_manifest_path
        .parent()
        .ok_or_else(|| anyhow!("run_manifest missing parent dir"))?;
    write_canonical_json(&run_dir.join("profile_manifest.json"), &profile_manifest)
        .context("write profile_manifest.json")?;
    write_canonical_json(&run_dir.join("run_manifest.lock.json"), &lock_manifest)
        .context("write run_manifest.lock.json")?;
    Ok(())
}
