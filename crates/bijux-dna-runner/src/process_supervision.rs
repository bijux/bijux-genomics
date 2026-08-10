use std::io::Read;
use std::process::{Child, ExitStatus};
use std::thread::JoinHandle;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use wait_timeout::ChildExt as _;

pub(crate) struct SupervisedOutput {
    pub(crate) status: ExitStatus,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
    pub(crate) timed_out: bool,
}

/// Wait for the operating system's child-exit notification while independent
/// readers drain both pipes. This keeps timeout enforcement event-driven and
/// prevents a verbose child from blocking on a full pipe before it can exit.
pub(crate) fn wait_for_output(
    child: &mut Child,
    timeout: Duration,
    label: &str,
) -> Result<SupervisedOutput> {
    let stdout = child.stdout.take().map(|pipe| drain(pipe, "stdout", label));
    let stderr = child.stderr.take().map(|pipe| drain(pipe, "stderr", label));
    let (status, timed_out) = if let Some(status) =
        child.wait_timeout(timeout).with_context(|| format!("wait for {label}"))?
    {
        (status, false)
    } else {
        child.kill().with_context(|| format!("kill timed out {label}"))?;
        let status = child.wait().with_context(|| format!("reap timed out {label}"))?;
        (status, true)
    };
    Ok(SupervisedOutput {
        status,
        stdout: join_pipe(stdout, "stdout", label)?,
        stderr: join_pipe(stderr, "stderr", label)?,
        timed_out,
    })
}

fn drain<R>(mut pipe: R, stream: &'static str, label: &str) -> JoinHandle<Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    let label = label.to_string();
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        pipe.read_to_end(&mut bytes).with_context(|| format!("read {stream} for {label}"))?;
        Ok(bytes)
    })
}

fn join_pipe(
    handle: Option<JoinHandle<Result<Vec<u8>>>>,
    stream: &str,
    label: &str,
) -> Result<Vec<u8>> {
    handle
        .map(|handle| handle.join().map_err(|_| anyhow!("{stream} reader panicked for {label}"))?)
        .transpose()
        .map(Option::unwrap_or_default)
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
        let mut child = Command::new("sh")
            .args(["-c", "exec tail -f /dev/null"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let output = wait_for_output(&mut child, Duration::from_millis(20), "timeout probe")?;

        assert!(output.timed_out);
        assert!(!output.status.success());
        Ok(())
    }
}
