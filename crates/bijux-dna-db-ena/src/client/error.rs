use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnaClientError {
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid ENA response: {0}")]
    InvalidResponse(String),
}
