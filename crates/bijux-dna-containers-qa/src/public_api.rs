//! Stable public API facade for environment QA consumers.

pub mod api {
    pub use crate::qualification::{
        compile_container_qualification_specification, derive_qualification_specification_coverage,
        evaluate_container_qualification, plan_container_qualification,
        qualification_specification_source_schema, write_qualification_specification_source_schema,
        ContainerQualificationAssessment, ContainerQualificationPlan, QualificationFailure,
        QualificationFailureClass, QualificationInputBind, QualificationMetricEvidence,
        QualificationMetricPlan, QualificationOutputEvidence, QualificationProbeKind,
        QualificationProbeObservation, QualificationProbePlan,
        QualificationSpecificationCompilationRequest, QualificationSpecificationCoverage,
        QualificationSpecificationCoverageState, QualificationSpecificationCoverageSummary,
        QualificationSpecificationCoverageUnit, QualificationSpecificationInputKindV1,
        QualificationSpecificationInputV1, QualificationSpecificationOutputV1,
        QualificationSpecificationSiteResourcesV1, QualificationSpecificationSourceV1,
        QUALIFICATION_SPECIFICATION_COVERAGE_SCHEMA_V2,
        QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1,
    };
    pub use crate::runtime_verification::{
        execute_runtime_verification, RuntimeVerificationRequest, RuntimeVerificationResult,
    };
    pub use crate::security_verification::{
        execute_security_verification, SecurityVerificationRequest, SecurityVerificationResult,
    };
    pub use bijux_dna_containers::api::*;
}
