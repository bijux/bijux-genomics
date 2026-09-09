mod decision;
mod evaluation;
mod model;
mod publication;
mod render;
mod report;
mod schema;

pub use decision::{
    evaluate_benchmark_evidence, evaluate_execution_evidence, evaluate_identity_evidence,
    evaluate_source_identity, BenchmarkEvidenceObservation, ExecutionEvidenceObservation,
    ExecutionEvidenceStatus, BENCHMARK_EVIDENCE_RULE_V1, BUILD_EVIDENCE_RULE_V1,
    CATALOG_EVIDENCE_RULE_V1, EXECUTION_EVIDENCE_RULE_V1, FRESHNESS_EVIDENCE_RULE_V1,
    RELEASE_EVIDENCE_RULE_V1, RUNTIME_EVIDENCE_RULE_V1, SECURITY_EVIDENCE_RULE_V1,
    SOURCE_IDENTITY_RULE_V1, SUPPORT_DIMENSIONS_RULE_V2, SUPPORT_DISPOSITION_RULE_V1,
};
pub use evaluation::REQUIRED_EVIDENCE_DIMENSIONS;
pub use model::{
    ArtifactAvailability, ArtifactDelivery, DecisionAudit, EvidenceDecision, EvidenceDimension,
    EvidenceVerdict, ProductSupport, PublicationDisposition, QualificationAssessment,
    QualificationClaim, SupportDisposition, QUALIFICATION_ASSESSMENT_SCHEMA_V3,
};
pub use publication::{
    evaluate_publication_freshness, ImmutablePublicationIdentity, PublicationArtifactIdentity,
    PublicationAuthority, PublicationFreshnessEvaluation, PublicationFreshnessVerdict,
    PublicationIdentityMismatch,
};
pub use render::{render_qualification_report_json, render_qualification_report_markdown};
pub use report::{
    QualificationEvidenceSource, QualificationReport, QualificationReportSummary,
    QUALIFICATION_REPORT_SCHEMA_V4,
};

pub use schema::{render_qualification_assessment_schema, render_qualification_report_schema};
