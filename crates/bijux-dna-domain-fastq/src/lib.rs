//! FASTQ domain definitions and contracts.
//!
//! Owns: FASTQ stage semantics, invariants, and contracts.
//! Must NOT depend on: bijux-dna-engine or runtime/container execution logic.
// Reading order:
// 1. domain.rs
// 2. core types
// 3. stage semantics
// 4. metrics spec
// 5. domain adapter
// Structural layout of this crate is frozen as of FASTQ v1.
mod artifacts;
pub mod banks;
mod bench;
pub mod bench_repository;
mod comparison_contract;
mod domain_adapter;
pub mod execution_support;
pub mod id_catalog;
mod integration_matrix;
pub mod invariants;
pub mod metrics;
pub mod observer;
pub mod params;
pub mod pipeline_contract;
pub mod prelude;
mod public_api;
mod qc_contract;
pub mod run;
mod stage_tool_governance;
pub mod stages;
pub mod types;
pub use public_api::*;

pub mod stage_contract {
    pub use crate::stages::contract::{stage_contract_hash, stage_contract_json};
}

pub mod stage_semantics {
    pub use crate::stages::semantics::{
        canonical_stage_order, fastq_stage_is_stable, optional_branches, stage_criticality,
        stage_kind, stage_metric_classes, stage_metric_invariants, stage_semantics,
        BoundaryInvariant, FastqStageKind, StageDefinition, StageSemantics,
        STAGE_BOUNDARY_INVARIANTS,
    };
}

pub mod stage_specs {
    pub use crate::stages::specs::{
        canonical_contract_for_stage, infer_input_kind, qc_class_for_stage, FastqStage,
        FastqStageContract, QcClass, StageContract, StageIO,
    };
}
