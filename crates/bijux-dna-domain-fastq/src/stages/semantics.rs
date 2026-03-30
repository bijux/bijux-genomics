mod catalog;
mod queries;

pub use catalog::{
    BoundaryInvariant, FastqStageKind, StageDefinition, StageSemantics, STAGES,
    STAGE_BOUNDARY_INVARIANTS,
};
pub use queries::{
    canonical_stage_order, fastq_stage_is_stable, optional_branches, stage_criticality, stage_kind,
    stage_metric_classes, stage_metric_invariants, stage_semantics,
};
