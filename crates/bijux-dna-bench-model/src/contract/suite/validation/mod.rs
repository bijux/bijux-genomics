//! Owner: bijux-dna-bench-model
//! Benchmark suite contract validation entrypoint.

mod declared_stage_nodes;
mod edge_contracts;
mod stage_contracts;
mod suite_validation;

pub(super) use declared_stage_nodes::DeclaredStageNodes;

#[cfg(feature = "canonical-fastq-catalog")]
pub use suite_validation::validate_suite;
pub use suite_validation::validate_suite_with_catalog;
