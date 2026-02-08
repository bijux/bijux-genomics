//! Shared helpers for stage adapter output auditing.

use std::path::Path;

use anyhow::Result;
use bijux_dna_core::prelude::{ArtifactId, ArtifactRole};
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};

#[must_use]
pub fn audit_outputs(stage: bijux_dna_domain_bam::BamStage, out_dir: &Path) -> Vec<ArtifactRef> {
    bijux_dna_domain_bam::required_audit_artifacts(stage)
        .iter()
        .map(|artifact| {
            let role = role_for_artifact_name(artifact.name);
            ArtifactRef::required(
                ArtifactId::new(artifact.name.to_string()),
                out_dir.join(artifact.filename),
                role,
            )
        })
        .collect()
}

fn role_for_artifact_name(name: &str) -> ArtifactRole {
    let name = name.to_ascii_lowercase();
    if name.contains("bam") {
        return ArtifactRole::Bam;
    }
    if name.contains("bai") || name.contains("index") || name.ends_with("_idx") {
        return ArtifactRole::Index;
    }
    if name.contains("metrics") {
        return ArtifactRole::MetricsJson;
    }
    if name.contains("report") || name.contains("summary") || name.contains("flagstat") {
        return ArtifactRole::ReportJson;
    }
    ArtifactRole::Unknown
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
        if !plan
            .io
            .outputs
            .iter()
            .any(|output| output.name.as_str() == *name)
        {
            missing.push(*name);
        }
    }
    if !missing.is_empty() {
        anyhow::bail!(
            "stage {} missing required outputs: {}",
            plan.stage_id.as_str(),
            missing.join(", ")
        );
    }
    Ok(plan)
}

pub struct ReferenceAssets {
    pub fasta: std::path::PathBuf,
    pub fai: std::path::PathBuf,
    pub dict: std::path::PathBuf,
}

/// # Errors
/// Returns an error if reference assets are missing and auto-build is disabled.
pub fn ensure_reference_assets(
    reference: &Path,
    build_indices: bool,
    require_dict: bool,
) -> Result<ReferenceAssets> {
    let fasta = reference.to_path_buf();
    let fai = reference.with_extension("fai");
    let dict = reference.with_extension("dict");
    if !build_indices {
        if !fai.exists() {
            anyhow::bail!("reference index missing: {}", fai.display());
        }
        if require_dict && !dict.exists() {
            anyhow::bail!("reference dict missing: {}", dict.display());
        }
    }
    Ok(ReferenceAssets { fasta, fai, dict })
}
