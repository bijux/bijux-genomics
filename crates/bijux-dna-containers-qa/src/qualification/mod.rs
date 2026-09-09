//! QA-owned container qualification planning and verdict evaluation.

mod compilation;
mod evaluation;
mod materialization;
mod model;
mod planning;
mod specification;

pub use compilation::{
    bind_qualification_source_to_frozen_repository, compile_container_qualification_specification,
    derive_qualification_specification_coverage,
};
pub use evaluation::evaluate_container_qualification;
pub use materialization::materialize_qualification_specification_source;
pub use model::{
    ContainerQualificationAssessment, ContainerQualificationPlan, QualificationFailure,
    QualificationFailureClass, QualificationInputBind, QualificationMetricEvidence,
    QualificationMetricPlan, QualificationOutputEvidence, QualificationProbeKind,
    QualificationProbeObservation, QualificationProbePlan,
};
pub use planning::plan_container_qualification;
pub use specification::{
    load_qualification_source_selection, load_qualification_specification_source,
    qualification_source_selection_schema, qualification_specification_source_schema,
    write_qualification_source_selection_schema, write_qualification_specification_source_schema,
    QualificationSourceInputSelectionV1, QualificationSourceMaterializationRequest,
    QualificationSourceSelectionV1, QualificationSpecificationCompilationRequest,
    QualificationSpecificationCoverage, QualificationSpecificationCoverageState,
    QualificationSpecificationCoverageSummary, QualificationSpecificationCoverageUnit,
    QualificationSpecificationInputKindV1, QualificationSpecificationInputV1,
    QualificationSpecificationOutputV1, QualificationSpecificationSiteResourcesV1,
    QualificationSpecificationSourceV1, QUALIFICATION_SOURCE_SELECTION_SCHEMA_V1,
    QUALIFICATION_SPECIFICATION_COVERAGE_SCHEMA_V2, QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1,
};
