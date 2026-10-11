use super::{wait_for_status, SupervisedOutput, SupervisedStatus};
use anyhow::{anyhow, Context, Result};
use std::io::Read;
use std::process::Child;
use std::thread::JoinHandle;
use std::time::Duration;
pub(super) fn collect(
    child: &mut Child,
    timeout: Duration,
    label: &str,
) -> Result<SupervisedOutput> {
    let stdout = child.stdout.take().map(|pipe| drain(pipe, "stdout", label));
    let stderr = child.stderr.take().map(|pipe| drain(pipe, "stderr", label));
    let SupervisedStatus { status, timed_out } = wait_for_status(child, timeout, label)?;
    Ok(SupervisedOutput {
        status,
        stdout: join_pipe(stdout, "stdout", label)?,
        stderr: join_pipe(stderr, "stderr", label)?,
        timed_out,
        output_limit_exceeded: false,
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
