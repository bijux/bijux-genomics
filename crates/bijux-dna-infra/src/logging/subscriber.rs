#[cfg(feature = "tracing")]
use std::path::Path;

#[cfg(feature = "tracing")]
use anyhow::{anyhow, Result};
#[cfg(feature = "tracing")]
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[cfg(feature = "tracing")]
pub(super) fn rolling_writer(
    path: &Path,
) -> Result<tracing_appender::rolling::RollingFileAppender> {
    Ok(tracing_appender::rolling::never(
        path.parent().ok_or_else(|| anyhow!("log path missing parent"))?,
        path.file_name().ok_or_else(|| anyhow!("log path missing filename"))?,
    ))
}

#[cfg(feature = "tracing")]
pub(super) fn install(writer: tracing_appender::non_blocking::NonBlocking) {
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(writer).with_ansi(false).compact())
        .with(EnvFilter::from_default_env())
        .init();
}
