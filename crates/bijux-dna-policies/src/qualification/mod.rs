mod decision;
mod evaluation;
mod model;
mod publication;
mod render;
mod report;

pub use decision::{
    evaluate_benchmark_evidence, evaluate_execution_evidence, evaluate_source_identity,
    BenchmarkEvidenceObservation, ExecutionEvidenceObservation, ExecutionEvidenceStatus,
    BENCHMARK_EVIDENCE_RULE_V1, EXECUTION_EVIDENCE_RULE_V1, SOURCE_IDENTITY_RULE_V1,
};
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
pub use render::{render_qualification_report_json, render_qualification_report_markdown};
pub use report::{
    QualificationEvidenceSource, QualificationReport, QualificationReportSummary,
    QUALIFICATION_REPORT_SCHEMA_V2,
};
