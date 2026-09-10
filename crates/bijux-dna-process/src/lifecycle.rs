use super::SupervisedStatus;
use anyhow::{Context, Result};
use std::process::{Child, ExitStatus};
use std::time::Duration;
use wait_timeout::ChildExt as _;

/// Reap direct children even when output collection fails midway through an operation.
#[cfg(unix)]
pub(super) struct ChildGuard<'a> {
    pub(super) child: &'a mut Child,
    pub(super) armed: bool,
}

#[cfg(unix)]
impl Drop for ChildGuard<'_> {
    fn drop(&mut self) {
        if self.armed {
            let _ = terminate(self.child, "abandoned output collection");
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

/// Wait for direct child exit and cancel its owned process group at the deadline.
/// Unix callers must isolate the group before spawning; file output remains caller-owned.
///
/// # Errors
/// Returns errors when waiting, signalling or reaping fails.
pub fn wait_for_status(
    child: &mut Child,
    timeout: Duration,
    label: &str,
) -> Result<SupervisedStatus> {
    if let Some(status) =
        child.wait_timeout(timeout).with_context(|| format!("wait for {label}"))?
    {
        return Ok(SupervisedStatus { status, timed_out: false });
    }
    Ok(SupervisedStatus { status: terminate(child, label)?, timed_out: true })
}

pub(super) fn terminate(child: &mut Child, label: &str) -> Result<ExitStatus> {
    #[cfg(unix)]
    {
        use rustix::io::Errno;
        use rustix::process::{kill_process_group, Pid, Signal};
        let pid =
            Pid::from_raw(i32::try_from(child.id())?).context("invalid child process group")?;
        let _ = child.try_wait().with_context(|| format!("inspect child for {label}"))?;
        match kill_process_group(pid, Signal::KILL) {
            Ok(()) | Err(Errno::SRCH) => {}
            Err(Errno::PERM) => {
                // A departing leader can race group signalling on Darwin. Reap it
                // and retry; a persistent denial must never become successful cancellation.
                anyhow::ensure!(
                    child.wait_timeout(Duration::from_millis(20))?.is_some(),
                    "cannot cancel live process group for {label}"
                );
                match kill_process_group(pid, Signal::KILL) {
                    Ok(()) | Err(Errno::SRCH) => {}
                    Err(error) => {
                        return Err(error).with_context(|| {
                            format!("cancel process group after leader exit for {label}")
                        })
                    }
                }
            }
            Err(error) => {
                return Err(error).with_context(|| format!("signal process group for {label}"))
            }
        }
    }
    #[cfg(not(unix))]
    child.kill().with_context(|| format!("kill timed out {label}"))?;
    child.wait().with_context(|| format!("reap cancelled {label}"))
}
