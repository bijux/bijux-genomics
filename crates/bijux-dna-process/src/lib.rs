//! Process transport primitives independent of domain, scheduler and container policy.
mod capture;
mod lifecycle;
#[cfg(unix)]
mod readiness;

pub use capture::{capture, CaptureLimits};
pub use lifecycle::wait_for_status;

#[cfg(unix)]
use std::process::Child;
use std::process::{Command, ExitStatus};
#[cfg(unix)]
use std::time::Duration;

/// Raw process evidence; limit flags distinguish cancellation even after a successful leader exit.
#[derive(Debug)]
pub struct SupervisedOutput {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub timed_out: bool,
    pub output_limit_exceeded: bool,
}

/// Direct child exit and whether cancellation was required at its deadline.
#[derive(Debug)]
pub struct SupervisedStatus {
    pub status: ExitStatus,
    pub timed_out: bool,
}

/// Arrange an isolated Unix process group before spawning an owned command.
pub fn isolate_process_group(command: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt as _;
        command.process_group(0);
    }
    #[cfg(not(unix))]
    let _ = command;
}

/// Collect an already spawned, isolated Unix child under one deadline and shared byte budget.
/// The child must have its own process group, with stdout/stderr configured as piped or files.
/// `None` preserves uncapped binary payload capture for callers that require complete bytes.
/// Prefer `capture` when the caller does not need to configure pipe ownership before spawning.
///
/// # Errors
/// Returns errors from deadline admission, pipe readiness, cancellation or child reaping.
#[cfg(unix)]
pub fn wait_for_output(
    child: &mut Child,
    timeout: Duration,
    max_output_bytes: Option<usize>,
    label: &str,
) -> anyhow::Result<SupervisedOutput> {
    readiness::collect(child, timeout, max_output_bytes, label)
}
