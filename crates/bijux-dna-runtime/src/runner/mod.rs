//! Runner execution models and runner-contract policy.

mod contracts;
mod model;

pub use contracts::{
    ensure_stage_supported_by_runner, DomainStageRunnerContract, PrefixDomainStageRunnerContract,
    RunnerContractKind,
};
pub use model::{Artifact, Invocation, Runner, RunnerResult};
