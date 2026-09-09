//! Evidence qualification contracts for v1.

use std::path::Path;

use anyhow::{Context, Result};

pub use bijux_dna_policies::{
    evaluate_publication_freshness, render_qualification_assessment_schema,
    render_qualification_report_json, render_qualification_report_markdown,
    render_qualification_report_schema, ArtifactAvailability, ArtifactDelivery, EvidenceDecision,
    EvidenceDimension, EvidenceVerdict, ImmutablePublicationIdentity, ProductSupport,
    PublicationArtifactIdentity, PublicationAuthority, PublicationDisposition,
    PublicationFreshnessEvaluation, PublicationFreshnessVerdict, PublicationIdentityMismatch,
    QualificationAssessment, QualificationClaim, QualificationEvidenceSource, QualificationReport,
    QualificationReportSummary, SupportDisposition, QUALIFICATION_ASSESSMENT_SCHEMA_V3,
    QUALIFICATION_REPORT_SCHEMA_V4, REQUIRED_EVIDENCE_DIMENSIONS,
};
pub use bijux_dna_science::evidence::genotypes_equivalent_unphased;

pub use bijux_dna_bench::{
    derive_container_qualification_report, derive_container_qualification_report_with_release,
};

/// Write the governed qualification-assessment schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_qualification_assessment_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &render_qualification_assessment_schema())
        .with_context(|| format!("write {}", path.display()))
}

/// Write the governed qualification-report schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_qualification_report_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &render_qualification_report_schema())
        .with_context(|| format!("write {}", path.display()))
}
