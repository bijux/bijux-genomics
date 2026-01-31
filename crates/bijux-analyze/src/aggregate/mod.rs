//! Owner: bijux-analyze
//! Metric aggregation and schema validation.

pub mod metrics;
pub mod registry;
pub mod stats;

pub type Result<T> = std::result::Result<T, BenchError>;

pub use metrics::*;
pub use registry::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BenchError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("measure error: {0}")]
    Measure(#[from] bijux_core::measure::MeasureError),
    #[error("validation error: {0}")]
    Validation(String),
}
