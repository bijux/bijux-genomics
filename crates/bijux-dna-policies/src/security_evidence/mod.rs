mod gitleaks;
mod grype;
mod model;
mod schema;
mod validation;

pub use gitleaks::{parse_gitleaks_report, ParsedSecretScanReport};
pub use grype::{parse_grype_vulnerability_report, ParsedVulnerabilityReport};
pub use model::{
    ContainerSecurityEvidencePublication, ImageSignatureDisposition, RuntimeAccessPolicy,
    RuntimeSecurityEvidence, SbomEvidence, SbomFormat, SecretScanEvidence,
    SecurityArtifactIdentity, SecuritySourceIdentity, StructuralSecurityEvidence,
    ToolSecurityEvidence, VulnerabilityDatabaseIdentity, VulnerabilityDatabaseStatus,
    VulnerabilityEvidence, VulnerabilityFinding, VulnerabilityScannerIdentity,
    VulnerabilitySeverity, VulnerabilitySeverityCounts, VulnerabilityWaiver,
    CONTAINER_SECURITY_EVIDENCE_SCHEMA_V1,
};
pub use schema::security_evidence_schema;
pub use validation::validate_security_evidence_publication;
