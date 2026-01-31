//! Owner: bijux-analyze
//! Canonical internal representation (IR) for analysis.
//! Owns typed facts/run/metric records and invariants.
//! Must not perform IO or depend on load/report/decision.
//! Invariants: no raw JSON values beyond typed wrappers.

pub mod facts;
pub mod json;
pub mod run;

pub use facts::{FactRow, FactTable};
pub use json::JsonBlob;
pub use run::{MetricEnvelope, RunSummary, StageRecord, ToolRecord};
