use anyhow::Result;
use sha2::Digest;
use std::fmt::Write as _;

use crate::{ExecutionPlan, PlanEdge, StagePlanV1};

use super::validation::stage_node_id;

impl ExecutionPlan {
    /// # Errors
    /// Returns an error if canonical JSON serialization fails.
    pub fn canonical_json(&self) -> Result<serde_json::Value> {
        let value = serde_json::to_value(self)?;
        Ok(bijux_dna_core::contract::canonical::canonicalize_json_value(&value))
    }

    /// # Errors
    /// Returns an error if canonical JSON serialization fails.
    pub fn plan_hash(&self) -> Result<String> {
        let canonical = self.canonical_json()?;
        let bytes = serde_json::to_vec(&canonical)?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(bytes);
        Ok(sha256_hex(hasher.finalize()))
    }
}

#[must_use]
pub fn default_edges_for_stages(stages: &[StagePlanV1]) -> Vec<PlanEdge> {
    let mut edges = Vec::new();
    for (to_idx, to_stage) in stages.iter().enumerate() {
        let mut artifact_edges = Vec::new();
        for input in &to_stage.io.inputs {
            let Some((from_stage, output)) = stages[..to_idx].iter().rev().find_map(|candidate| {
                candidate
                    .io
                    .outputs
                    .iter()
                    .find(|output| output.name == input.name)
                    .map(|output| (candidate, output))
            }) else {
                continue;
            };
            artifact_edges.push(PlanEdge::with_artifact_binding(
                stage_node_id(from_stage),
                stage_node_id(to_stage),
                output.name.as_str(),
                input.name.as_str(),
            ));
        }
        if artifact_edges.is_empty() {
            if let Some(from_stage) = to_idx.checked_sub(1).and_then(|idx| stages.get(idx)) {
                edges.push(PlanEdge::new(stage_node_id(from_stage), stage_node_id(to_stage)));
            }
        } else {
            edges.extend(artifact_edges);
        }
    }
    edges.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then_with(|| left.to.cmp(&right.to))
            .then_with(|| left.from_output_id.cmp(&right.from_output_id))
            .then_with(|| left.to_input_id.cmp(&right.to_input_id))
    });
    edges.dedup_by(|left, right| {
        left.from == right.from
            && left.to == right.to
            && left.from_output_id == right.from_output_id
            && left.to_input_id == right.to_input_id
    });
    edges
}

fn sha256_hex(digest: impl AsRef<[u8]>) -> String {
    let bytes = digest.as_ref();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}
