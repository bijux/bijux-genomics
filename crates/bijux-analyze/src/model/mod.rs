//! Owner: bijux-analyze
//! Canonical internal representation (IR) for analysis.
//! Owns typed facts/run/metric records and invariants.
//! Must not perform IO or depend on load/report/decision.
//! Invariants: no raw JSON values beyond typed wrappers.

pub mod dashboard;
pub mod facts;
pub mod json;
pub mod run;

pub use dashboard::DashboardFactRow;
pub use facts::{FactRow, FactTable};
pub use json::JsonBlob;
pub(crate) use run::stable_sort_records;
pub use run::{
    FactsSummary, MetricEnvelope, RunSummary, RunSummaryDeltas, RunSummaryStageRow, RunSummaryV1,
    StageRecord, ToolRecord,
};
