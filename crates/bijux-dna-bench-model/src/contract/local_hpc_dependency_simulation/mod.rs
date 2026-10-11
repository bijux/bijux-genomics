//! Owner: bijux-dna-bench-model
//! Construction and semantic admission of HPC dependency-simulation reports.

mod simulation;
mod validation;

pub use simulation::build_benchmark_local_hpc_dependency_simulation;
pub use validation::validate_benchmark_local_hpc_dependency_simulation;
