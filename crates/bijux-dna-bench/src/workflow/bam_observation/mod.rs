//! Owner: bijux-dna-bench
//! Shared immutable authorities for BAM observational benchmark attempts.

mod inputs;
mod parameters;

pub(super) use inputs::execution_inputs;
pub(super) use parameters::{ensure_parameter_evidence, BamParameterPolicy};
