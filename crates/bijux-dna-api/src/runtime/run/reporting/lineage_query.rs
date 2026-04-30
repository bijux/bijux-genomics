use super::Result;
use crate::request_args::{
    RunLineageEdgeV1, RunLineageQueryRequestV1, RunLineageQueryResponseV1,
};
use anyhow::Context;

/// Query artifact lineage edges from a run artifact inventory.
///
/// # Errors
/// Returns an error if the inventory cannot be loaded.
pub fn query_run_lineage(request: &RunLineageQueryRequestV1) -> Result<RunLineageQueryResponseV1> {
    let layout =
        bijux_dna_runtime::run_layout::RunLayout::from_run_dir(request.run_dir.to_path_buf());
    let inventory: bijux_dna_runtime::run_layout::ArtifactInventoryV1 = serde_json::from_slice(
        &std::fs::read(&layout.artifact_inventory_path)
            .with_context(|| format!("read {}", layout.artifact_inventory_path.display()))?,
    )
    .context("parse artifact inventory")?;

    let mut edges = inventory
        .artifacts
        .iter()
        .filter(|artifact| {
            request
                .artifact_id
                .as_ref()
                .is_none_or(|requested| requested == &artifact.artifact_id)
        })
        .flat_map(|artifact| {
            artifact
                .input_lineage
                .iter()
                .map(|lineage_key| RunLineageEdgeV1 {
                    artifact_id: artifact.artifact_id.clone(),
                    producing_stage_id: artifact.producing_stage_id.clone(),
                    lineage_key: lineage_key.clone(),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    edges.sort_by(|left, right| {
        left
            .artifact_id
            .cmp(&right.artifact_id)
            .then_with(|| left.lineage_key.cmp(&right.lineage_key))
    });

    Ok(RunLineageQueryResponseV1 {
        schema_version: "bijux.run_lineage_query.v1".to_string(),
        run_id: inventory.run_id,
        run_dir: request.run_dir.clone(),
        total_artifacts: inventory.artifacts.len(),
        edges,
    })
}
