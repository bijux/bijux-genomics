//! Owner: bijux-dna-analyze
//! Decision core for ranking, comparison, and explainability.
//! Owns compare/score/explain logic and decision traces.
//! Must not depend on load/report or perform IO.
//! Invariants: missing semantics produce errors with remediation hints.

pub mod compare;
pub mod effect;
pub mod score;
mod trace;

pub use trace::{DecisionMetricTrace, DecisionTrace};
