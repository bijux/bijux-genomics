//! Stable public API facade for environment QA consumers.

pub mod api {
    pub use crate::qualification::{
        bind_qualification_source_to_frozen_repository,
        compile_container_qualification_specification, derive_qualification_specification_coverage,
        evaluate_container_qualification, load_qualification_source_selection,
        load_qualification_specification_source, materialize_qualification_specification_source,
        plan_container_qualification, qualification_source_selection_schema,
        qualification_specification_source_schema, write_qualification_source_selection_schema,
        write_qualification_specification_source_schema, ContainerQualificationAssessment,
        ContainerQualificationPlan, QualificationFailure, QualificationFailureClass,
        QualificationInputBind, QualificationMetricEvidence, QualificationMetricPlan,
        QualificationOutputEvidence, QualificationProbeKind, QualificationProbeObservation,
        QualificationProbePlan, QualificationSourceInputSelectionV1,
        QualificationSourceMaterializationRequest, QualificationSourceSelectionV1,
        QualificationSpecificationCompilationRequest, QualificationSpecificationCoverage,
        QualificationSpecificationCoverageState, QualificationSpecificationCoverageSummary,
        QualificationSpecificationCoverageUnit, QualificationSpecificationInputKindV1,
        QualificationSpecificationInputV1, QualificationSpecificationOutputV1,
        QualificationSpecificationSiteResourcesV1, QualificationSpecificationSourceV1,
        QUALIFICATION_SOURCE_SELECTION_SCHEMA_V1, QUALIFICATION_SPECIFICATION_COVERAGE_SCHEMA_V2,
        QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1,
    };
    #[cfg(feature = "verification-tools")]
    pub use crate::runtime_verification::{
        can_reuse_apptainer_smoke, execute_runtime_verification, run_apptainer_smoke,
        ApptainerSmokeManifest, ApptainerSmokeProbeResult, ApptainerSmokeRequest,
        RuntimeVerificationRequest, RuntimeVerificationResult, APPTAINER_SMOKE_MANIFEST_SCHEMA_V5,
        APPTAINER_SMOKE_MANIFEST_SCHEMA_V6,
    };
    #[cfg(feature = "verification-tools")]
    pub use crate::security_verification::{
        execute_security_verification, SecurityVerificationRequest, SecurityVerificationResult,
    };
    pub use bijux_dna_containers::api::*;
}
