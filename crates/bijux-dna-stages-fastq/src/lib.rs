//! Stage specs, metrics, and observers for FASTQ.

pub mod metrics;
pub mod observer;
mod plugin;
mod runtime;
pub mod stage_specs;
mod surface;

pub use bijux_dna_stage_contract::StagePlanJsonV1 as StagePlanJson;
pub use plugin::FastqStagePlugin;
pub use runtime::{
    runtime_interpretation_for_stage, runtime_interpretation_for_stage_tool,
    runtime_interpretation_stage_ids, RuntimeInterpretationLevel,
};
pub use surface::contracts;
pub use surface::{
    closed_execution_stage_ids, contract_stage_ids, implemented_stages,
    observer_specialized_stage_ids, observer_stage_ids, observer_stage_tool_bindings,
};
