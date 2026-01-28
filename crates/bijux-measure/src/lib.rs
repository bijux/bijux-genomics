//! Execution-time measurement layer for runtime/resource metrics.
//! This crate is the single authority for run-time measurement schemas.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, MeasureError>;

#[derive(Debug, Error)]
pub enum MeasureError {
    #[error("validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub runtime_s: f64,
    /// Container memory usage in MB sampled via `docker stats --no-stream` after exit.
    pub memory_mb: f64,
    pub exit_code: i32,
}

impl ExecutionMetrics {
    /// Validate execution metrics are sane and non-negative.
    ///
    /// # Errors
    /// Returns an error if any metrics are invalid.
    pub fn validate(&self) -> Result<()> {
        if !(self.runtime_s.is_finite() && self.runtime_s > 0.0) {
            return Err(MeasureError::Validation(
                "runtime_s must be > 0".to_string(),
            ));
        }
        if !(self.memory_mb.is_finite() && self.memory_mb > 0.0) {
            return Err(MeasureError::Validation(
                "memory_mb must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}
