// Reading order:
// 1. domain.rs
// 2. core types
// 3. stage semantics
// 4. metrics spec
// 5. execution adapters
// Structural layout of this crate is frozen as of FASTQ v1.
pub mod adapter;
pub mod analyze;
pub mod contracts;
pub mod domain;
pub mod invariants;
pub mod metrics;
pub mod stages;
