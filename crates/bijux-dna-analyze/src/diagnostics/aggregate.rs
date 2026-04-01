use thiserror::Error;

#[derive(Debug, Error)]
pub enum BenchError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("sqlite error: {0}")]
    #[cfg(feature = "sqlite")]
    Sqlite(#[from] rusqlite::Error),
    #[error("measure error: {0}")]
    Measure(#[from] bijux_dna_core::prelude::measure::MeasureError),
    #[error("validation error: {0}")]
    Validation(String),
}
