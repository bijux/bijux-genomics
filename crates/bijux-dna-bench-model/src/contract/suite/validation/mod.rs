//! Owner: bijux-dna-bench
//! Benchmark suite contract validation entrypoint.

mod edge_contracts;
mod stage_contracts;
mod suite_validation;

type DeclaredStageNodes = std::collections::BTreeSet<String>;

pub use suite_validation::validate_suite;
