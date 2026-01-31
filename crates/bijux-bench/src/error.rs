//! Owner: bijux-bench
//! Error types for bench workflows.
//! Owns stable error codes for bench validation.
//! Must not perform IO.

use std::fmt;

#[derive(Debug, Clone)]
pub enum BenchError {
    MissingConfounder { field: &'static str },
    InvalidObservation { reason: String },
}

impl fmt::Display for BenchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BenchError::MissingConfounder { field } => {
                write!(f, "missing required confounder: {field}")
            }
            BenchError::InvalidObservation { reason } => write!(f, "{reason}"),
        }
    }
}

impl std::error::Error for BenchError {}
