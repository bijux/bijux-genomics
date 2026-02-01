//! Owner: bijux-bench
//! Typed models for bench (suite, observation, summary, decision).
//! Must not perform IO or depend on repo/policy/compare logic.

pub mod decision;
pub mod observation;
pub mod suite;
pub mod summary;

pub use decision::BenchmarkDecision;
pub use observation::BenchmarkObservation;
pub use suite::BenchmarkSuiteSpec;
pub use summary::{BenchmarkSummary, MetricSummary, SummaryRow, SummaryStratum};
