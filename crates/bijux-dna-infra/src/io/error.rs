use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoErrorKind {
    Permission,
    Missing,
    Transient,
    Corruption,
    LockTimeout,
    Other,
}

#[derive(Debug, Error)]
#[error("{kind:?}: {message}")]
pub struct IoError {
    pub kind: IoErrorKind,
    pub message: String,
    #[source]
    pub source: Option<std::io::Error>,
}

impl IoError {
    #[must_use]
    pub fn from_io(err: std::io::Error) -> Self {
        let kind = classify_io_error(&err);
        Self { kind, message: err.to_string(), source: Some(err) }
    }

    #[must_use]
    pub fn new(kind: IoErrorKind, message: impl Into<String>) -> Self {
        Self { kind, message: message.into(), source: None }
    }
}

#[must_use]
pub fn classify_io_error(err: &std::io::Error) -> IoErrorKind {
    use std::io::ErrorKind;

    match err.kind() {
        ErrorKind::NotFound => IoErrorKind::Missing,
        ErrorKind::PermissionDenied => IoErrorKind::Permission,
        ErrorKind::TimedOut | ErrorKind::WouldBlock | ErrorKind::Interrupted => {
            IoErrorKind::Transient
        }
        ErrorKind::InvalidData | ErrorKind::InvalidInput | ErrorKind::UnexpectedEof => {
            IoErrorKind::Corruption
        }
        _ => IoErrorKind::Other,
    }
}
