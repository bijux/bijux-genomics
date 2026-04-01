use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::prelude::CacheKey;

use super::artifact_catalog::{collect_all_run_artifacts, run_artifacts_dir};
use super::bootstrap::initialize_runtime_support_files;
use super::manifest_identity::input_hash_from_many;
use super::reproducibility::{prepare_reproducibility_context, write_reproducibility_report};
use super::{write_profile_and_lock_manifests, RunArtifactInput, RunDirs};
use crate::recording::write_atomic_bytes;

/// # Errors
/// Returns an error if the run manifest or auxiliary files cannot be written.
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value)]
pub fn write_run_manifest(
    run_dirs: &RunDirs,
    stage: &str,
    _tool: &str,
    run_provenance: &crate::RunProvenanceV1,
    stage_contract_hash: Option<String>,
    extra_artifacts: &[RunArtifactInput],
) -> Result<()> {
    let run_id = run_dirs
        .run_manifest_path
        .parent()
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow!("run id missing from run manifest path"))?
        .to_string();
    let support_files = initialize_runtime_support_files(run_dirs)?;
    let pipeline_id = std::env::var("BIJUX_PIPELINE_ID")
        .ok()
        .unwrap_or_else(|| run_provenance.pipeline_id.clone());
    let profile_id = std::env::var("BIJUX_PROFILE_ID").ok();
    let graph_hash = run_provenance
        .plan_hash
        .clone()
        .or_else(|| std::env::var("BIJUX_PLAN_HASH").ok());
    let declared_tool_image_digest = run_provenance
        .tool_image_digest
        .clone()
        .ok_or_else(|| anyhow!("run manifest requires declared tool image digest"))?;
    let cache_key = CacheKey::new(
        input_hash_from_many(&run_provenance.input_hashes),
        run_provenance.params_hash.clone(),
        run_provenance.tool_version.clone(),
        declared_tool_image_digest.clone(),
    );
    let repro_context = prepare_reproducibility_context(run_dirs, run_provenance)?;
    let image_upstream = std::env::var("BIJUX_IMAGE_UPSTREAM").ok();
    let image_build_timestamp_unix_s = std::env::var("BIJUX_IMAGE_BUILD_TIMESTAMP_UNIX_S")
        .ok()
        .and_then(|v| v.parse::<u64>().ok());
    write_reproducibility_report(
        run_dirs,
        &pipeline_id,
        graph_hash.as_ref(),
        run_provenance,
        &declared_tool_image_digest,
        &repro_context,
    )?;
    let payload = serde_json::json!({
        "schema_version": "bijux.run_manifest.v3",
        "contract_version": bijux_dna_core::contract::ContractVersion::v1(),
        "run_id": run_id,
        "pipeline_id": pipeline_id,
        "profile_id": profile_id,
        "graph_hash": graph_hash,
        "cache_key": cache_key,
        "stage_contract_hash": stage_contract_hash,
        "toolchain_versions": {
            "planner": std::env::var("BIJUX_PLANNER_VERSION").ok(),
            "engine": std::env::var("BIJUX_ENGINE_VERSION").ok(),
        },
        "dataset_fingerprints": run_provenance.input_hashes.clone(),
        "tool_invocations": repro_context.tool_invocations,
        "output_artifacts": [],
        "stages": [],
        "failures": [],
        "run_provenance": run_provenance,
        "execution_replay_identity": {
            "tool_image_ref": repro_context.replay_tool_image_ref,
            "tool_image_digest": repro_context.replay_tool_image_digest,
            "tool_version_output": repro_context.replay_tool_version_output,
        },
        "image_identity_provenance": {
            "tool_id": repro_context.replay_tool_id,
            "version": run_provenance.tool_version.clone(),
            "digest": run_provenance.tool_image_digest.clone(),
            "upstream": image_upstream,
            "build_timestamp_unix_s": image_build_timestamp_unix_s,
        },
        "telemetry": {
            "events_jsonl": support_files.telemetry_events_path,
        },
        "dashboard": {
            "facts_jsonl": support_files.dashboard_facts_path,
        },
        "run_context": repro_context.run_context,
        "reproducibility_tuple": repro_context.reproducibility_tuple,
    });
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&payload)?;
    write_atomic_bytes(&run_dirs.run_manifest_path, payload.as_slice())
        .context("write run_manifest.json")?;
    let artifacts = collect_all_run_artifacts(run_dirs, extra_artifacts)?;
    let output_artifacts: Vec<serde_json::Value> = artifacts
        .iter()
        .map(|artifact| {
            serde_json::json!({
                "stage_id": stage,
                "name": artifact.get("name").cloned().unwrap_or(serde_json::Value::Null),
                "role": serde_json::Value::Null,
                "optional": false,
                "path": artifact.get("path").cloned().unwrap_or(serde_json::Value::Null),
                "sha256": artifact.get("sha256").cloned().unwrap_or(serde_json::Value::Null),
            })
        })
        .collect();
    let mut run_manifest: serde_json::Value =
        serde_json::from_slice(std::fs::read(&run_dirs.run_manifest_path)?.as_slice())
            .context("parse run_manifest for artifact enrichment")?;
    run_manifest["output_artifacts"] = serde_json::Value::Array(output_artifacts);
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&run_manifest)?;
    write_atomic_bytes(&run_dirs.run_manifest_path, payload.as_slice())
        .context("rewrite run_manifest with complete artifact hashes")?;
    write_profile_and_lock_manifests(&run_dirs.run_manifest_path)?;
    Ok(())
}
