use std::path::Path;

use anyhow::Result;
use bijux_core::{ArtifactRef, StagePlanV1};

#[must_use]
pub fn audit_outputs(stage: bijux_domain_bam::BamStage, out_dir: &Path) -> Vec<ArtifactRef> {
    bijux_domain_bam::required_audit_artifacts(stage)
        .iter()
        .map(|artifact| ArtifactRef {
            name: artifact.name.to_string(),
            path: out_dir.join(artifact.filename),
        })
        .collect()
}

/// # Errors
/// Returns an error if the effective params payload is null.
pub fn ensure_effective_params(value: serde_json::Value) -> Result<serde_json::Value> {
    if value.is_null() {
        anyhow::bail!("effective_params must not be null");
    }
    Ok(value)
}

/// # Errors
/// Returns an error if required outputs are missing from the plan.
pub fn ensure_required_outputs(plan: StagePlanV1, required: &[&str]) -> Result<StagePlanV1> {
    let mut missing = Vec::new();
    for name in required {
        if !plan.io.outputs.iter().any(|output| output.name == *name) {
            missing.push(*name);
        }
    }
    if !missing.is_empty() {
        anyhow::bail!(
            "stage {} missing required outputs: {}",
            plan.stage_id.0,
            missing.join(", ")
        );
    }
    Ok(plan)
}
