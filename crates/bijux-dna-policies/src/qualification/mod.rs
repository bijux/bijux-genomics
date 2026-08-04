mod evaluation;
mod model;

pub use evaluation::REQUIRED_EVIDENCE_DIMENSIONS;
pub use model::{
    EvidenceDecision, EvidenceDimension, EvidenceVerdict, PublicationDisposition,
    QualificationAssessment, QualificationClaim, SupportDisposition,
    QUALIFICATION_ASSESSMENT_SCHEMA_V1,
};
