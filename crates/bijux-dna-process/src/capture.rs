//! Capture owns spawning, byte budgets and cancellation; callers own command meaning.
use crate::SupervisedOutput;
#[cfg(unix)]
use anyhow::Context;
use anyhow::Result;
use std::process::Command;
#[cfg(unix)]
use std::process::Stdio;
use std::time::Duration;

/// Limits apply to both streams together and to their EOF plus direct child exit.
#[derive(Debug, Clone, Copy)]
pub struct CaptureLimits {
    pub timeout: Duration,
    pub max_output_bytes: usize,
}

/// Run an isolated command with closed stdin and bounded raw output.
///
/// Unix descendants remaining in the owned process group are cancelled on either limit.
/// Non-Unix platforms reject before spawning until equivalent pipe cancellation is available.
/// The caller retains ownership of executable resolution, argv, environment and working directory.
///
/// # Errors
/// Returns an error for unsupported platforms, invalid deadlines, launch or collection failures.
pub fn capture(
    command: &mut Command,
    limits: CaptureLimits,
    label: &str,
) -> Result<SupervisedOutput> {
    #[cfg(unix)]
    {
        anyhow::ensure!(!limits.timeout.is_zero(), "capture deadline must be positive");
        std::time::Instant::now()
            .checked_add(limits.timeout)
            .context("capture deadline overflow")?;
        crate::isolate_process_group(command);
        command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = command.spawn().with_context(|| format!("launch {label}"))?;
        crate::wait_for_output(&mut child, limits.timeout, Some(limits.max_output_bytes), label)
    }
    #[cfg(not(unix))]
    {
        let _ = (command, limits, label);
        anyhow::bail!("bounded process capture is unsupported on this platform")
    }
}
