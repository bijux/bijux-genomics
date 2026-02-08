//! Owner: bijux-engine

use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum EngineError {
    #[error("planning error: {0}")]
    Planning(String),
    #[error("execution error: {0}")]
    Execution(String),
    #[error("observation error: {0}")]
    Observation(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("contract error on step {step_id}: {message} (artifact: {artifact_id}, path: {path})")]
    Contract {
        step_id: String,
        artifact_id: String,
        path: String,
        message: String,
    },
}
