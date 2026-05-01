use super::Result;
use crate::request_args::{EvidenceCheckFailureV1, EvidenceGapRequestV1, EvidenceGapResponseV1};
use anyhow::Context;

/// Typed API for evidence-gap diagnostics.
///
/// # Errors
/// Returns an error if evidence contracts are unreadable.
pub fn evidence_gap(request: &EvidenceGapRequestV1) -> Result<EvidenceGapResponseV1> {
    let layout =
        bijux_dna_runtime::run_layout::RunLayout::from_run_dir(request.run_dir.to_path_buf());
    let verification = layout
        .evidence_verification_path
        .exists()
        .then(|| std::fs::read(&layout.evidence_verification_path).ok())
        .flatten()
        .map(|raw| {
            serde_json::from_slice::<bijux_dna_analyze::exports::EvidenceVerificationV1>(&raw)
                .context("parse evidence verification")
        })
        .transpose()?;
    let inventory = layout
        .artifact_inventory_path
        .exists()
        .then(|| std::fs::read(&layout.artifact_inventory_path).ok())
        .flatten()
        .map(|raw| {
            serde_json::from_slice::<bijux_dna_runtime::run_layout::ArtifactInventoryV1>(&raw)
                .context("parse artifact inventory")
        })
        .transpose()?;

    let missing_paths =
        verification.as_ref().map_or_else(Vec::new, |value| value.missing_paths.clone());
    let failed_checks = verification.as_ref().map_or_else(Vec::new, |value| {
        value
            .checks
            .iter()
            .filter(|check| !check.ok)
            .map(|check| EvidenceCheckFailureV1 {
                check_id: check.check_id.clone(),
                message: check.message.clone(),
            })
            .collect::<Vec<_>>()
    });

    let (advisory_only_artifacts, unsafe_artifacts) = inventory.as_ref().map_or_else(
        || (Vec::new(), Vec::new()),
        |inventory| {
            let advisory = inventory
                .artifacts
                .iter()
                .filter_map(|artifact| {
                    artifact.scientific_context.as_ref().and_then(|context| {
                        (context.safe_to_use && context.advisory_only)
                            .then(|| artifact.artifact_id.clone())
                    })
                })
                .collect::<Vec<_>>();
            let unsafe_ids = inventory
                .artifacts
                .iter()
                .filter_map(|artifact| {
                    artifact.scientific_context.as_ref().and_then(|context| {
                        (!context.safe_to_use).then(|| artifact.artifact_id.clone())
                    })
                })
                .collect::<Vec<_>>();
            (advisory, unsafe_ids)
        },
    );

    let verified = verification.as_ref().is_some_and(|value| value.verified);
    let base_gap_count = verification.as_ref().map_or(0, |value| value.gap_count);

    Ok(EvidenceGapResponseV1 {
        schema_version: "bijux.evidence_gap.v1".to_string(),
        run_dir: request.run_dir.clone(),
        verified,
        gap_count: base_gap_count
            + failed_checks.len()
            + missing_paths.len()
            + unsafe_artifacts.len(),
        missing_paths,
        failed_checks,
        advisory_only_artifacts,
        unsafe_artifacts,
    })
}
