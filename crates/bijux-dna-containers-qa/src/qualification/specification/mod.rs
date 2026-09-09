//! Shared qualification specification contracts; compilation and materialization are consumers.
pub(crate) mod admission;
mod model;
mod storage;
pub use model::{
    qualification_source_selection_schema, qualification_specification_source_schema,
    QualificationSourceInputSelectionV1, QualificationSourceMaterializationRequest,
    QualificationSourceSelectionV1, QualificationSpecificationCompilationRequest,
    QualificationSpecificationCoverage, QualificationSpecificationCoverageState,
    QualificationSpecificationCoverageSummary, QualificationSpecificationCoverageUnit,
    QualificationSpecificationInputKindV1, QualificationSpecificationInputV1,
    QualificationSpecificationOutputV1, QualificationSpecificationSiteResourcesV1,
    QualificationSpecificationSourceV1, QUALIFICATION_SOURCE_SELECTION_SCHEMA_V1,
    QUALIFICATION_SPECIFICATION_COVERAGE_SCHEMA_V2, QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1,
};
pub use storage::{
    load_qualification_source_selection, load_qualification_specification_source,
    write_qualification_source_selection_schema, write_qualification_specification_source_schema,
};
