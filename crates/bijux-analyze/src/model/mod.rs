//! Owner: bijux-analyze
//! Canonical internal representation (IR) for analysis.

pub mod facts;
pub mod json;
pub mod run;

pub use facts::{FactRow, FactTable};
pub use json::JsonBlob;
pub use run::{MetricEnvelope, RunSummary, StageRecord, ToolRecord};
