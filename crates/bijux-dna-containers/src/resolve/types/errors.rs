use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnvError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("platform error: {0}")]
    Platform(String),
    #[error("runner unavailable")]
    RuntimeUnavailable,
    #[error("dockerfile error: {0}")]
    Dockerfile(String),
    #[error("image error: {0}")]
    Image(String),
}

impl From<bijux_dna_infra::IoError> for EnvError {
    fn from(err: bijux_dna_infra::IoError) -> Self {
        Self::Io(std::io::Error::other(err))
    }
}
