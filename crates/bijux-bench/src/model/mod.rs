//! Owner: bijux-bench
//! Typed models for bench (suite, observation, summary, decision).
//! Must not perform IO or depend on repo/policy/compare logic.

pub mod decision;
pub mod observation;
pub mod summary;
pub mod suite;

pub use decision::{BenchmarkDecision, DecisionRationale};
pub use observation::BenchmarkObservation;
pub use summary::{BenchmarkSummary, MetricSummary, SummaryRow};
pub use suite::BenchmarkSuiteSpec;
