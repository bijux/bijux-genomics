//! QA-owned container qualification planning and verdict evaluation.

mod compilation;
mod evaluation;
mod materialization;
mod model;
mod planning;

pub use compilation::{
    bind_qualification_source_to_frozen_repository, compile_container_qualification_specification,
    derive_qualification_specification_coverage, load_qualification_specification_source,
    qualification_specification_source_schema, write_qualification_specification_source_schema,
    QualificationSpecificationCompilationRequest, QualificationSpecificationCoverage,
    QualificationSpecificationCoverageState, QualificationSpecificationCoverageSummary,
    QualificationSpecificationCoverageUnit, QualificationSpecificationInputKindV1,
    QualificationSpecificationInputV1, QualificationSpecificationOutputV1,
    QualificationSpecificationSiteResourcesV1, QualificationSpecificationSourceV1,
    QUALIFICATION_SPECIFICATION_COVERAGE_SCHEMA_V2, QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1,
};
pub use evaluation::evaluate_container_qualification;
pub use materialization::{
    load_qualification_source_selection, materialize_qualification_specification_source,
    qualification_source_selection_schema, write_qualification_source_selection_schema,
    QualificationSourceInputSelectionV1, QualificationSourceMaterializationRequest,
    QualificationSourceSelectionV1, QUALIFICATION_SOURCE_SELECTION_SCHEMA_V1,
};
pub use model::{
    ContainerQualificationAssessment, ContainerQualificationPlan, QualificationFailure,
    QualificationFailureClass, QualificationInputBind, QualificationMetricEvidence,
    QualificationMetricPlan, QualificationOutputEvidence, QualificationProbeKind,
    QualificationProbeObservation, QualificationProbePlan,
};
pub use planning::plan_container_qualification;
