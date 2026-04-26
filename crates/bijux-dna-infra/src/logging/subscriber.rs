#[cfg(feature = "tracing")]
use std::path::Path;

#[cfg(feature = "tracing")]
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[cfg(feature = "tracing")]
use crate::{IoError, IoErrorKind};

#[cfg(feature = "tracing")]
pub(super) fn rolling_writer(
    path: &Path,
) -> Result<tracing_appender::rolling::RollingFileAppender, IoError> {
    Ok(tracing_appender::rolling::never(
        path.parent()
            .ok_or_else(|| IoError::new(IoErrorKind::Corruption, "log path missing parent"))?,
        path.file_name()
            .ok_or_else(|| IoError::new(IoErrorKind::Corruption, "log path missing filename"))?,
    ))
}

#[cfg(feature = "tracing")]
pub(super) fn install(writer: tracing_appender::non_blocking::NonBlocking) -> Result<(), IoError> {
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(writer).with_ansi(false).json())
        .with(EnvFilter::from_default_env())
        .try_init()
        .map_err(|err| {
            IoError::new(IoErrorKind::Other, format!("install tracing subscriber: {err}"))
        })
}
