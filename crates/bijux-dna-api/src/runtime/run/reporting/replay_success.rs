use super::Result;
use crate::request_args::{ReplayExplainRequestV1, ReplayExplainResponseV1};
use anyhow::{anyhow, Context};
use bijux_dna_runtime::run_layout::{ArtifactInventoryV1, ReplayManifestV1};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Typed replay explainability API.
///
/// # Errors
/// Returns an error if replay or inventory contracts are missing or invalid.
pub fn replay_explain(request: &ReplayExplainRequestV1) -> Result<ReplayExplainResponseV1> {
    let replay_layout = bijux_dna_runtime::run_layout::RunLayout::from_run_dir(
        request.replay_run_dir.to_path_buf(),
    );
    let replay_manifest: ReplayManifestV1 = serde_json::from_slice(
        &std::fs::read(&replay_layout.replay_manifest_path)
            .with_context(|| format!("read {}", replay_layout.replay_manifest_path.display()))?,
    )
    .context("parse replay manifest")?;
    let original_layout = bijux_dna_runtime::run_layout::RunLayout::from_run_dir(
        request.original_run_dir.to_path_buf(),
    );
    let original_inventory = load_inventory(&original_layout.artifact_inventory_path)?;
    let replay_inventory = load_inventory(&replay_layout.artifact_inventory_path)?;

    let original_hashes = inventory_hashes(&original_inventory);
    let replay_hashes = inventory_hashes(&replay_inventory);
    let reused_ids = replay_manifest.reused_artifact_ids.into_iter().collect::<BTreeSet<_>>();
    let mut unchanged_outputs = Vec::new();
    let mut changed_outputs = Vec::new();
    let mut unverifiable_outputs = Vec::new();
    for expected in &replay_manifest.expected_outputs {
        match (
            replay_hashes.get(expected).and_then(|value| value.as_deref()),
            original_hashes.get(expected).and_then(|value| value.as_deref()),
        ) {
            (Some(replay_sha), Some(original_sha)) if replay_sha == original_sha => {
                unchanged_outputs.push(expected.clone());
            }
            (Some(_), Some(_)) => {
                changed_outputs.push(expected.clone());
            }
            _ => {
                unverifiable_outputs.push(expected.clone());
            }
        }
    }
    let reused_outputs = replay_manifest
        .expected_outputs
        .iter()
        .filter(|artifact_id| reused_ids.contains(*artifact_id))
        .cloned()
        .collect::<Vec<_>>();

    if replay_manifest.original_run_id.is_empty() {
        return Err(anyhow!("replay manifest original_run_id must not be empty"));
    }

    Ok(ReplayExplainResponseV1 {
        schema_version: "bijux.replay_success_explain.v1".to_string(),
        original_run_dir: request.original_run_dir.display().to_string(),
        replay_run_dir: request.replay_run_dir.display().to_string(),
        original_run_id: replay_manifest.original_run_id,
        replay_run_id: replay_manifest.replay_run_id,
        rerun_stage_ids: replay_manifest.rerun_stage_ids,
        reused_outputs,
        unchanged_outputs,
        changed_outputs,
        unverifiable_outputs,
    })
}

/// Explain artifact reuse and drift for a successful replay.
///
/// # Errors
/// Returns an error if replay or inventory contracts are missing or invalid.
pub fn explain_successful_replay(
    original_run_dir: &Path,
    replay_run_dir: &Path,
) -> Result<serde_json::Value> {
    let response = replay_explain(&ReplayExplainRequestV1 {
        original_run_dir: original_run_dir.to_path_buf(),
        replay_run_dir: replay_run_dir.to_path_buf(),
    })?;
    Ok(serde_json::to_value(response)?)
}

fn load_inventory(path: &Path) -> Result<ArtifactInventoryV1> {
    serde_json::from_slice(
        &std::fs::read(path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))
}

fn inventory_hashes(inventory: &ArtifactInventoryV1) -> BTreeMap<String, Option<String>> {
    inventory
        .artifacts
        .iter()
        .map(|artifact| (artifact.artifact_id.clone(), artifact.sha256.clone()))
        .collect()
}
