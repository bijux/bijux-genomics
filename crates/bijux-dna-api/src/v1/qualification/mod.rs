//! Evidence qualification contracts for v1.

use std::path::Path;

use anyhow::{Context, Result};

pub use bijux_dna_policies::{
    evaluate_publication_freshness, EvidenceDecision, EvidenceDimension, EvidenceVerdict,
    ImmutablePublicationIdentity, PublicationArtifactIdentity, PublicationAuthority,
    PublicationDisposition, PublicationFreshnessEvaluation, PublicationFreshnessVerdict,
    PublicationIdentityMismatch, QualificationAssessment, QualificationClaim, SupportDisposition,
    QUALIFICATION_ASSESSMENT_SCHEMA_V1,
};

/// Render the governed qualification-assessment schema.
///
/// # Panics
/// Panics if the Rust-derived schema cannot be represented as JSON.
#[must_use]
pub fn render_qualification_assessment_schema() -> serde_json::Value {
    match serde_json::to_value(schemars::schema_for!(QualificationAssessment)) {
        Ok(value) => value,
        Err(error) => panic!("serialize qualification-assessment schema: {error}"),
    }
}

/// Write the governed qualification-assessment schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_qualification_assessment_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &render_qualification_assessment_schema())
        .with_context(|| format!("write {}", path.display()))
}
