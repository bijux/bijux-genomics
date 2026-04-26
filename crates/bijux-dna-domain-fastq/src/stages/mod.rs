//! Owner: bijux-dna-domain-fastq
//! Declarative stage definitions. See docs/DOMAIN_MODEL.md for stage authority.

pub mod analyze;
pub mod contract;
pub mod ids;
pub mod ports;
pub mod semantics;
pub mod specs;

pub use contract::*;
pub use ids::*;
pub use ports::{
    stage_compatible_tool_ids, stage_input_ids, stage_output_ids, stage_parameter_ids,
};
pub use semantics::{
    fastq_stage_is_stable, stage_criticality, stage_kind, stage_metric_classes,
    stage_metric_invariants, stage_semantics, BoundaryInvariant, FastqStageKind, StageDefinition,
    StageSemantics, STAGE_BOUNDARY_INVARIANTS,
};
pub use specs::*;
