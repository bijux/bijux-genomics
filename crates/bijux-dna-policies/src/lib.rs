//! Shared contract/policy checks across crates (enforces ownership and interfaces).

mod assertions;
mod checks;
mod guardrails;
pub mod policy_diagnostics;
pub mod public_api;
pub mod qualification;
mod source_scan;

pub use public_api::{check, GuardrailConfig};
pub use qualification::{
    EvidenceDecision, EvidenceDimension, EvidenceVerdict, PublicationDisposition,
    QualificationAssessment, QualificationClaim, SupportDisposition,
    QUALIFICATION_ASSESSMENT_SCHEMA_V1,
};
