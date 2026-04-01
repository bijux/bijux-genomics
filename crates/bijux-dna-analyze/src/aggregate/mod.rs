//! Owner: bijux-dna-analyze
//! Metric aggregation and schema validation.
//! Owns metric schemas, validation, and rollups for analysis.
//! Must not perform IO or call into load/report/pipeline layers.
//! Invariants: metrics must validate against registry; rollups are deterministic.

pub mod metrics;
pub mod schema;
pub mod stats;

pub use crate::diagnostics::aggregate::BenchError;

pub type Result<T> = std::result::Result<T, BenchError>;

pub use metrics::*;
pub use schema::*;
