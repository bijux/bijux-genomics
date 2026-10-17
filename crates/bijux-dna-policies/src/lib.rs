//! Shared contract/policy checks across crates (enforces ownership and interfaces).

mod assertions;
mod checks;
mod guardrails;
pub mod policy_diagnostics;
pub mod public_api;
pub mod qualification;
pub mod release_evidence;
pub mod security_evidence;
mod source_scan;
pub mod tool_eligibility;

pub use checks::build_authority::{
    validate_ci_build_authority, CiBuildAuthority, CiCacheAuthority, CiCompilationAuthority,
    CiLockfileAuthority, CiTargetAuthority, CiTestExecutorAuthority, CiToolchainAuthority,
    CI_BUILD_AUTHORITY_SCHEMA_V3,
};
pub use checks::dependency_graph::{
    plan_change_impact, validate_dependency_graph, ChangeImpactPlan, CiDependencyEdge,
    CiDependencyGraph, CiDependencyKind, CiDependencyNode, CiGateFamily, CiNodeKind,
    CiPeriodicFullValidation, CiPlannerPolicy, CI_DEPENDENCY_GRAPH_SCHEMA_V1,
};
pub use checks::no_wait::{
    evaluate_no_wait_gate, is_governed_no_wait_source, no_wait_gate_schema, no_wait_policy_sha256,
    AllowlistedNoWaitFinding, NoWaitAllowlist, NoWaitException, NoWaitFinding, NoWaitGateReceipt,
    NoWaitGateSummary, NoWaitRule, NoWaitSource, NoWaitVerdict, NO_WAIT_ALLOWLIST_SCHEMA_V1,
    NO_WAIT_GATE_SCHEMA_V1,
};
pub use public_api::{check, GuardrailConfig};
pub use qualification::{
    evaluate_benchmark_evidence, evaluate_execution_evidence, evaluate_identity_evidence,
    evaluate_publication_freshness, evaluate_source_identity,
    render_qualification_assessment_schema, render_qualification_report_json,
    render_qualification_report_markdown, render_qualification_report_schema, ArtifactAvailability,
    ArtifactDelivery, BenchmarkEvidenceObservation, DecisionAudit, EvidenceDecision,
    EvidenceDimension, EvidenceVerdict, ExecutionEvidenceObservation, ExecutionEvidenceStatus,
    ImmutablePublicationIdentity, ProductSupport, PublicationArtifactIdentity,
    PublicationAuthority, PublicationDisposition, PublicationFreshnessEvaluation,
    PublicationFreshnessVerdict, PublicationIdentityMismatch, QualificationAssessment,
    QualificationClaim, QualificationEvidenceSource, QualificationReport,
    QualificationReportSummary, SupportDisposition, BENCHMARK_EVIDENCE_RULE_V1,
    BUILD_EVIDENCE_RULE_V1, CATALOG_EVIDENCE_RULE_V1, EXECUTION_EVIDENCE_RULE_V1,
    FRESHNESS_EVIDENCE_RULE_V1, QUALIFICATION_ASSESSMENT_SCHEMA_V3, QUALIFICATION_REPORT_SCHEMA_V4,
    RELEASE_EVIDENCE_RULE_V1, REQUIRED_EVIDENCE_DIMENSIONS, RUNTIME_EVIDENCE_RULE_V1,
    SECURITY_EVIDENCE_RULE_V1, SOURCE_IDENTITY_RULE_V1, SUPPORT_DIMENSIONS_RULE_V2,
    SUPPORT_DISPOSITION_RULE_V1,
};
pub use release_evidence::{
    admit_release_evidence, derive_release_evidence, qualification_evidence_set_sha256,
    qualification_sources_sha256, release_evidence_policy_sha256, release_evidence_schema,
    validate_release_evidence_publication, AdmittedReleaseEvidence,
    ContainerReleaseEvidencePublication, ReleaseEvidenceDerivationRequest, ToolReleaseEvidence,
    CONTAINER_RELEASE_EVIDENCE_SCHEMA_V1,
};
pub use security_evidence::{
    admit_security_evidence, classify_vulnerability_database, derive_security_evidence,
    parse_gitleaks_report, parse_grype_vulnerability_report, security_evidence_policy_sha256,
    security_evidence_schema, validate_security_evidence_publication, AdmittedSecurityEvidence,
    CollectedSecurityToolObservation, ContainerSecurityEvidencePublication,
    ImageSignatureDisposition, NoSecurityArtifactObservation, ParsedSecretScanReport,
    ParsedVulnerabilityReport, RuntimeAccessPolicy, RuntimeProbeObservation,
    RuntimeSecurityEvidence, SbomEvidence, SbomFormat, ScannerReportObservation,
    SecretScanEvidence, SecurityArtifactIdentity, SecurityEvidenceDerivationRequest,
    SecuritySourceIdentity, SecurityToolObservation, StructuralProbeObservation,
    StructuralSecurityEvidence, ToolSecurityEvidence, VulnerabilityDatabaseIdentity,
    VulnerabilityDatabaseStatus, VulnerabilityEvidence, VulnerabilityFinding,
    VulnerabilityScannerIdentity, VulnerabilitySeverity, VulnerabilitySeverityCounts,
    VulnerabilityWaiver, CONTAINER_SECURITY_EVIDENCE_SCHEMA_V1,
    CONTAINER_SECURITY_EVIDENCE_SCHEMA_V2,
};
pub use tool_eligibility::{
    select_eligible_tools, ToolEligibilityCandidate, ToolEligibilityPolicy, ToolEligibilityTier,
};

pub use checks::workspace_dependencies::{
    analyze_workspace_dependencies, WorkspaceDependencyAnalysis,
};

pub use checks::workspace_source_audits::{
    evaluate_workspace_source_audit, forbidden_workspace_execution_dependencies,
    workspace_execution_owner_dependencies, workspace_source_matches, WorkspaceExecutionDependency,
    WorkspaceForbiddenDependency, WorkspaceSourceAuditInput, WorkspaceSourceMatch,
    WorkspaceSourceRule,
};
