//! Process lifecycle and output collection share one operation deadline.
use anyhow::Result;
use std::process::{Child, Command, ExitStatus};
use std::time::Duration;

mod lifecycle;
#[cfg(unix)]
mod readiness;
#[cfg(not(unix))]
mod threaded;
pub(crate) use lifecycle::wait_for_status;

pub(crate) struct SupervisedOutput {
    pub(crate) status: ExitStatus,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
    pub(crate) timed_out: bool,
}
pub(crate) struct SupervisedStatus {
    pub(crate) status: ExitStatus,
    pub(crate) timed_out: bool,
}
/// Isolate Unix descendants before spawning so cancellation reaches inherited pipes.
pub(crate) fn isolate_process_group(command: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt as _;
        command.process_group(0);
    }
    #[cfg(not(unix))]
    let _ = command;
}
/// Collect both streams and child exit. Unix readiness uses a shared deadline.
pub(crate) fn wait_for_output(
    child: &mut Child,
    timeout: Duration,
    label: &str,
) -> Result<SupervisedOutput> {
    #[cfg(unix)]
    {
        readiness::collect(child, timeout, label)
    }
    #[cfg(not(unix))]
    {
        threaded::collect(child, timeout, label)
    }
}
#[cfg(test)]
mod tests {
    use std::process::{Command, Stdio};
    use std::time::Duration;

    use super::wait_for_output;

    #[cfg(unix)]
    #[test]
    fn timed_wait_drains_output_without_polling() -> anyhow::Result<()> {
        let mut child = Command::new("sh")
            .args(["-c", "printf stdout; printf stderr >&2"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let output = wait_for_output(&mut child, Duration::from_secs(2), "output probe")?;

        assert!(!output.timed_out);
        assert!(output.status.success());
        assert_eq!(output.stdout, b"stdout");
        assert_eq!(output.stderr, b"stderr");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn timed_wait_kills_and_reaps_at_deadline() -> anyhow::Result<()> {
        let mut command = Command::new("sh");
        command.args(["-c", "tail -f /dev/null"]).stdout(Stdio::piped()).stderr(Stdio::piped());
        super::isolate_process_group(&mut command);
        let mut child = command.spawn()?;
        let output = wait_for_output(&mut child, Duration::from_millis(20), "timeout probe")?;

        assert!(output.timed_out);
        assert!(!output.status.success());
        Ok(())
    }
}
