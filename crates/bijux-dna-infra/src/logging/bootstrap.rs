use std::path::Path;

use crate::IoError;

/// # Errors
/// Returns an error if logging setup fails.
pub fn init_logging(
    log_path: &Path,
) -> Result<tracing_appender::non_blocking::WorkerGuard, IoError> {
    #[cfg(not(feature = "tracing"))]
    {
        let _ = log_path;
        Err(IoError::new(
            crate::IoErrorKind::Other,
            "logging requires bijux-dna-infra tracing feature; enable it in the caller",
        ))
    }
    #[cfg(feature = "tracing")]
    {
        let writer = subscriber::rolling_writer(log_path)?;
        let (non_blocking, guard) = tracing_appender::non_blocking(writer);
        subscriber::install(non_blocking)?;
        Ok(guard)
    }
}

#[cfg(feature = "tracing")]
use super::subscriber;
