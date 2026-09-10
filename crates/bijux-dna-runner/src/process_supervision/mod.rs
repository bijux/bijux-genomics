//! Runner compatibility over the shared process owner; payload capture remains uncapped.
use anyhow::Result;
use std::process::Child;
use std::time::Duration;
#[cfg(not(unix))]
mod threaded;
#[cfg(not(unix))]
pub(crate) use bijux_dna_process::SupervisedStatus;
pub(crate) use bijux_dna_process::{isolate_process_group, wait_for_status, SupervisedOutput};

pub(crate) fn wait_for_output(
    child: &mut Child,
    timeout: Duration,
    label: &str,
) -> Result<SupervisedOutput> {
    #[cfg(unix)]
    {
        bijux_dna_process::wait_for_output(child, timeout, None, label)
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
