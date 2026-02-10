use std::path::Path;

use anyhow::{anyhow, Result};
#[cfg(feature = "tracing")]
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// # Errors
/// Returns an error if logging setup fails.
pub fn init_logging(log_path: &Path) -> Result<tracing_appender::non_blocking::WorkerGuard> {
    #[cfg(not(feature = "tracing"))]
    {
        let _ = log_path;
        Err(anyhow!(
            "logging requires bijux-dna-infra tracing feature; enable it in the caller"
        ))
    }
    #[cfg(feature = "tracing")]
    {
        let file_appender = tracing_appender::rolling::never(
            log_path
                .parent()
                .ok_or_else(|| anyhow!("log path missing parent"))?,
            log_path
                .file_name()
                .ok_or_else(|| anyhow!("log path missing filename"))?,
        );
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .with_writer(non_blocking)
                    .with_ansi(false)
                    .compact(),
            )
            .with(EnvFilter::from_default_env())
            .init();
        Ok(guard)
    }
}
