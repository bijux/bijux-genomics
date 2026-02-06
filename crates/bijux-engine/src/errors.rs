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
}
