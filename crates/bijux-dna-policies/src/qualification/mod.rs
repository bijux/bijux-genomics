mod evaluation;
mod model;
mod publication;

pub use evaluation::REQUIRED_EVIDENCE_DIMENSIONS;
pub use model::{
    EvidenceDecision, EvidenceDimension, EvidenceVerdict, PublicationDisposition,
    QualificationAssessment, QualificationClaim, SupportDisposition,
    QUALIFICATION_ASSESSMENT_SCHEMA_V1,
};
pub use publication::{
    evaluate_publication_freshness, ImmutablePublicationIdentity, PublicationArtifactIdentity,
    PublicationAuthority, PublicationFreshnessEvaluation, PublicationFreshnessVerdict,
    PublicationIdentityMismatch,
};
