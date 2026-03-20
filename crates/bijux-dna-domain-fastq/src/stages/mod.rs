//! Owner: bijux-dna-domain-fastq
//! Declarative stage definitions. See docs/STAGES.md for authoritative docs.

pub mod analyze;
pub mod contract;
pub mod ids;
pub mod ports;
pub mod semantics;
pub mod specs;

pub use contract::*;
pub use ids::*;
pub use ports::{stage_input_ids, stage_output_ids};
pub use semantics::{
    fastq_stage_is_stable, stage_criticality, stage_kind, stage_metric_classes,
    stage_metric_invariants, stage_semantics, BoundaryInvariant, FastqStageKind, StageDefinition,
    StageSemantics, STAGE_BOUNDARY_INVARIANTS,
};
pub use specs::*;
