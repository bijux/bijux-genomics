use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnaClientError {
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid ENA query: {0}")]
    InvalidQuery(#[from] crate::model::QueryValidationError),
    #[error("invalid ENA response: {0}")]
    InvalidResponse(String),
}
