use std::path::Path;
use std::process::Command;
use std::time::Duration;
use std::time::Instant;

use anyhow::{bail, Context, Result};

mod command_line;
mod command_output;
mod invocation_identity;

use command_line::build_command_string;
pub use command_output::{CommandOutputV1, TimedCommandOutputV1};
pub use invocation_identity::invocation_hash;

use std::collections::BTreeMap;

/// # Errors
/// Returns an error if the command cannot be executed.
pub fn run_command(command: &str, args: &[String]) -> Result<CommandOutputV1> {
    run_command_with_context_and_stdin(command, args, None, None, None)
}

/// # Errors
/// Returns an error if the command cannot be executed.
pub fn run_command_with_context(
    command: &str,
    args: &[String],
    current_dir: Option<&Path>,
    envs: Option<&BTreeMap<String, String>>,
) -> Result<CommandOutputV1> {
    run_command_with_context_and_stdin(command, args, current_dir, envs, None)
}

/// # Errors
/// Returns an error if the command cannot be executed.
pub fn run_command_with_context_and_timeout(
    command: &str,
    args: &[String],
    current_dir: Option<&Path>,
    envs: Option<&BTreeMap<String, String>>,
    timeout: Duration,
) -> Result<TimedCommandOutputV1> {
    if command.trim().is_empty() {
        bail!("command executable must not be empty");
    }

    let start = Instant::now();
    let mut child = Command::new(command);
    child.args(args);
    if let Some(current_dir) = current_dir {
        child.current_dir(current_dir);
    }
    if let Some(envs) = envs {
        child.envs(envs);
    }
    child.stdin(std::process::Stdio::null());
    child.stdout(std::process::Stdio::piped());
    child.stderr(std::process::Stdio::piped());

    let mut process = child.spawn().with_context(|| format!("run command {command}"))?;
    let deadline = Instant::now() + timeout;

    loop {
        if let Some(status) =
            process.try_wait().with_context(|| format!("poll command {command}"))?
        {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            if let Some(mut handle) = process.stdout.take() {
                use std::io::Read;
                handle
                    .read_to_end(&mut stdout)
                    .with_context(|| format!("read stdout for command {command}"))?;
            }
            if let Some(mut handle) = process.stderr.take() {
                use std::io::Read;
                handle
                    .read_to_end(&mut stderr)
                    .with_context(|| format!("read stderr for command {command}"))?;
            }

            return Ok(TimedCommandOutputV1 {
                output: command_output_from_parts(
                    command,
                    args,
                    start,
                    status.code().unwrap_or(-1),
                    stdout,
                    stderr,
                ),
                timed_out: false,
            });
        }

        if Instant::now() >= deadline {
            process.kill().with_context(|| format!("kill timed out command {command}"))?;
            let status =
                process.wait().with_context(|| format!("wait for timed out command {command}"))?;
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            if let Some(mut handle) = process.stdout.take() {
                use std::io::Read;
                handle
                    .read_to_end(&mut stdout)
                    .with_context(|| format!("read timed out stdout for command {command}"))?;
            }
            if let Some(mut handle) = process.stderr.take() {
                use std::io::Read;
                handle
                    .read_to_end(&mut stderr)
                    .with_context(|| format!("read timed out stderr for command {command}"))?;
            }
            let timeout_note =
                format!("local command exceeded timeout of {}s", timeout.as_secs()).into_bytes();
            let mut timed_out_stderr = stderr;
            if !timed_out_stderr.is_empty() {
                timed_out_stderr.push(b'\n');
            }
            timed_out_stderr.extend(timeout_note);

            return Ok(TimedCommandOutputV1 {
                output: command_output_from_parts(
                    command,
                    args,
                    start,
                    status.code().unwrap_or(-1),
                    stdout,
                    timed_out_stderr,
                ),
                timed_out: true,
            });
        }

        std::thread::sleep(Duration::from_millis(25));
    }
}

/// # Errors
/// Returns an error if the command cannot be executed.
pub fn run_command_with_context_and_stdin(
    command: &str,
    args: &[String],
    current_dir: Option<&Path>,
    envs: Option<&BTreeMap<String, String>>,
    stdin_bytes: Option<&[u8]>,
) -> Result<CommandOutputV1> {
    if command.trim().is_empty() {
        bail!("command executable must not be empty");
    }
    let start = Instant::now();
    let mut child = Command::new(command);
    child.args(args);
    if let Some(current_dir) = current_dir {
        child.current_dir(current_dir);
    }
    if let Some(envs) = envs {
        child.envs(envs);
    }
    if stdin_bytes.is_some() {
        use std::io::Write;
        use std::process::Stdio;

        child.stdin(Stdio::piped());
        child.stdout(Stdio::piped());
        child.stderr(Stdio::piped());

        let mut process = child.spawn().with_context(|| format!("run command {command}"))?;
        if let Some(input) = stdin_bytes {
            let stdin = process
                .stdin
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("open stdin for command {command}"))?;
            stdin.write_all(input).with_context(|| format!("write stdin for command {command}"))?;
        }
        let output = process
            .wait_with_output()
            .with_context(|| format!("collect output for command {command}"))?;
        return Ok(command_output(command, args, start, &output));
    }
    let output = child.output().with_context(|| format!("run command {command}"))?;
    Ok(command_output(command, args, start, &output))
}

fn command_output(
    command: &str,
    args: &[String],
    start: Instant,
    output: &std::process::Output,
) -> CommandOutputV1 {
    command_output_from_parts(
        command,
        args,
        start,
        output.status.code().unwrap_or(-1),
        output.stdout.clone(),
        output.stderr.clone(),
    )
}

fn command_output_from_parts(
    command: &str,
    args: &[String],
    start: Instant,
    exit_code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
) -> CommandOutputV1 {
    let runtime_s = start.elapsed().as_secs_f64();
    let stdout = String::from_utf8_lossy(&stdout).to_string();
    let stderr = String::from_utf8_lossy(&stderr).to_string();
    CommandOutputV1 {
        stdout,
        stderr,
        exit_code,
        runtime_s,
        command: build_command_string(command, args),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{run_command_with_context, run_command_with_context_and_stdin};

    #[test]
    fn run_command_rejects_empty_executable() {
        let err = match run_command_with_context("  ", &[], None, None) {
            Ok(_) => panic!("empty executable should be rejected before spawn"),
            Err(err) => err,
        };

        assert_eq!(err.to_string(), "command executable must not be empty");
    }

    #[cfg(unix)]
    #[test]
    fn run_command_supports_stdin_payload() {
        let args = Vec::new();
        let output = run_command_with_context_and_stdin("cat", &args, None, None, Some(b"bijux"))
            .expect("run cat");
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout, "bijux");
    }
}
