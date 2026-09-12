//! Governed QC fixture admission and publication; no external `MultiQC` execution.
mod admission;
mod model;
mod projection;
mod publication;
pub use model::LocalReportQcConfig;
pub use publication::materialize_qc_fixture;
