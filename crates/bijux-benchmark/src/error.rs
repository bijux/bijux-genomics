//! Owner: bijux-benchmark
//! Error types for bench workflows.
//! Owns stable error codes for bench validation.
//! Must not perform IO.

use std::fmt;

#[derive(Debug, Clone)]
pub enum BenchError {
    InvalidPolicy(String),
    MissingMetrics(String),
    MissingSemantics(String),
    IncompatibleRuns(String),
    RepoError(String),
    ArtifactWriteError(String),
    MissingConfounder { field: &'static str },
    InvalidObservation { reason: String },
}

impl fmt::Display for BenchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BenchError::InvalidPolicy(reason) => write!(f, "invalid policy: {reason}"),
            BenchError::MissingMetrics(reason) => write!(f, "missing metrics: {reason}"),
            BenchError::MissingSemantics(reason) => write!(f, "missing semantics: {reason}"),
            BenchError::IncompatibleRuns(reason) => write!(f, "incompatible runs: {reason}"),
            BenchError::RepoError(reason) => write!(f, "repo error: {reason}"),
            BenchError::ArtifactWriteError(reason) => write!(f, "artifact write error: {reason}"),
            BenchError::MissingConfounder { field } => {
                write!(f, "missing required confounder: {field}")
            }
            BenchError::InvalidObservation { reason } => write!(f, "{reason}"),
        }
    }
}

impl std::error::Error for BenchError {}
