// Reading order:
// 1. domain.rs
// 2. core types
// 3. stage semantics
// 4. metrics spec
// 5. execution adapters
pub mod adapter;
pub mod analyze;
pub mod augment;
pub mod core;
pub mod domain;
pub mod meta;
pub mod metrics;
pub mod pipeline;
pub mod stages;
