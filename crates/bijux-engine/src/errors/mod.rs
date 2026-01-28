use thiserror::Error;

#[derive(Debug, Error)]
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
