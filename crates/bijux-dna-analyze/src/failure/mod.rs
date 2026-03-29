//! Owner: bijux-dna-analyze
//! Failure classification and structured remediation hints.
//! Owns stable failure IDs and remediation guidance.
//! Must not perform IO or depend on pipeline/report layers.
//! Invariants: failure kinds are stable and hints are structured.

mod classify;
mod hints;
mod types;

pub use classify::{classify_raw_failure, error_category, failure_class};
pub use types::{BenchmarkFailure, FailureClass, FailureKind};
