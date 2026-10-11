//! QA-owned container qualification planning and verdict evaluation.

mod compilation;
mod evaluation;
mod model;
mod planning;

pub use compilation::{
    compile_container_qualification_specification, derive_qualification_specification_coverage,
    qualification_specification_source_schema, write_qualification_specification_source_schema,
    QualificationSpecificationCompilationRequest, QualificationSpecificationCoverage,
    QualificationSpecificationCoverageState, QualificationSpecificationCoverageSummary,
    QualificationSpecificationCoverageUnit, QualificationSpecificationInputKindV1,
    QualificationSpecificationInputV1, QualificationSpecificationOutputV1,
    QualificationSpecificationSiteResourcesV1, QualificationSpecificationSourceV1,
    QUALIFICATION_SPECIFICATION_COVERAGE_SCHEMA_V1, QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1,
};
pub use evaluation::evaluate_container_qualification;
pub use model::{
    ContainerQualificationAssessment, ContainerQualificationPlan, QualificationFailure,
    QualificationFailureClass, QualificationInputBind, QualificationMetricEvidence,
    QualificationMetricPlan, QualificationOutputEvidence, QualificationProbeKind,
    QualificationProbeObservation, QualificationProbePlan,
};
pub use planning::plan_container_qualification;
