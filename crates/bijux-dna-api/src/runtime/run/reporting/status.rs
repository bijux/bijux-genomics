use super::{Path, Result};
use crate::request_args::RunStatus;

/// # Errors
/// This wrapper preserves the public API shape and does not currently return an error.
#[allow(clippy::unnecessary_wraps)]
pub fn status(run_dir: &Path) -> Result<RunStatus> {
    Ok(super::lifecycle::status(run_dir))
}
