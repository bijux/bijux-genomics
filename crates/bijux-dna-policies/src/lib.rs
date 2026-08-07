//! Shared contract/policy checks across crates (enforces ownership and interfaces).

mod assertions;
mod checks;
mod guardrails;
pub mod policy_diagnostics;
pub mod public_api;
pub mod qualification;
pub mod security_evidence;
mod source_scan;

pub use public_api::{check, GuardrailConfig};
pub use qualification::{
    evaluate_benchmark_evidence, evaluate_execution_evidence, evaluate_publication_freshness,
    evaluate_source_identity, render_qualification_report_json,
    render_qualification_report_markdown, BenchmarkEvidenceObservation, EvidenceDecision,
    EvidenceDimension, EvidenceVerdict, ExecutionEvidenceObservation, ExecutionEvidenceStatus,
    ImmutablePublicationIdentity, PublicationArtifactIdentity, PublicationAuthority,
    PublicationDisposition, PublicationFreshnessEvaluation, PublicationFreshnessVerdict,
    PublicationIdentityMismatch, QualificationAssessment, QualificationClaim,
    QualificationEvidenceSource, QualificationReport, QualificationReportSummary,
    SupportDisposition, BENCHMARK_EVIDENCE_RULE_V1, EXECUTION_EVIDENCE_RULE_V1,
    QUALIFICATION_ASSESSMENT_SCHEMA_V1, QUALIFICATION_REPORT_SCHEMA_V2,
    REQUIRED_EVIDENCE_DIMENSIONS, SOURCE_IDENTITY_RULE_V1,
};
pub use security_evidence::{
    parse_gitleaks_report, parse_grype_vulnerability_report, security_evidence_schema,
    validate_security_evidence_publication, ContainerSecurityEvidencePublication,
    ImageSignatureDisposition, ParsedSecretScanReport, ParsedVulnerabilityReport,
    RuntimeAccessPolicy, RuntimeSecurityEvidence, SbomEvidence, SbomFormat, SecretScanEvidence,
    SecurityArtifactIdentity, SecuritySourceIdentity, StructuralSecurityEvidence,
    ToolSecurityEvidence, VulnerabilityDatabaseIdentity, VulnerabilityDatabaseStatus,
    VulnerabilityEvidence, VulnerabilityFinding, VulnerabilityScannerIdentity,
    VulnerabilitySeverity, VulnerabilitySeverityCounts, VulnerabilityWaiver,
    CONTAINER_SECURITY_EVIDENCE_SCHEMA_V1,
};
