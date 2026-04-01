//! Runtime telemetry adapter selection and run-journal events.

mod adapter;
pub mod events;

pub use adapter::{build_telemetry_adapter, TelemetryAdapter, TelemetrySpan};
