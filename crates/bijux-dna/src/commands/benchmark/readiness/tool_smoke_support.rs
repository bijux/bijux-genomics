use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, SystemTime};

use anyhow::{bail, Context, Result};

#[derive(Debug, Clone)]
pub(crate) struct CommandExecution {
    pub(crate) exit_code: i32,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) timed_out: bool,
}

pub(crate) fn run_command_with_timeout(
    repo_root: &Path,
    applied_command: &[String],
    timeout: Duration,
    context_label: &str,
) -> Result<CommandExecution> {
    let mut child = spawn_command(repo_root, applied_command, &[], context_label)?;

    let (status, stdout, stderr, timed_out) = wait_with_timeout(&mut child, timeout)?;
    Ok(CommandExecution { exit_code: status.code().unwrap_or(-1), stdout, stderr, timed_out })
}

pub(crate) fn run_command_with_timeout_and_env(
    repo_root: &Path,
    applied_command: &[String],
    envs: &[(String, String)],
    timeout: Duration,
    context_label: &str,
) -> Result<CommandExecution> {
    let mut child = spawn_command(repo_root, applied_command, envs, context_label)?;

    let (status, stdout, stderr, timed_out) = wait_with_timeout(&mut child, timeout)?;
    Ok(CommandExecution { exit_code: status.code().unwrap_or(-1), stdout, stderr, timed_out })
}

fn spawn_command(
    repo_root: &Path,
    applied_command: &[String],
    envs: &[(String, String)],
    context_label: &str,
) -> Result<std::process::Child> {
    let Some(program) = applied_command.first() else {
        bail!("{context_label} attempted to run an empty command");
    };
    let mut command = Command::new(program);
    command
        .args(applied_command.iter().skip(1))
        .current_dir(repo_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in envs {
        command.env(key, value);
    }
    command.spawn().with_context(|| format!("{context_label} `{}`", applied_command.join(" ")))
}

fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Result<(ExitStatus, String, String, bool)> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait().context("poll local smoke command")? {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            if let Some(mut handle) = child.stdout.take() {
                handle.read_to_end(&mut stdout).context("read local smoke stdout")?;
            }
            if let Some(mut handle) = child.stderr.take() {
                handle.read_to_end(&mut stderr).context("read local smoke stderr")?;
            }
            return Ok((
                status,
                String::from_utf8_lossy(&stdout).to_string(),
                String::from_utf8_lossy(&stderr).to_string(),
                false,
            ));
        }

        if std::time::Instant::now() >= deadline {
            child.kill().context("kill timed out local smoke command")?;
            let status = child.wait().context("wait for killed local smoke command")?;
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            if let Some(mut handle) = child.stdout.take() {
                handle.read_to_end(&mut stdout).context("read timed out local smoke stdout")?;
            }
            if let Some(mut handle) = child.stderr.take() {
                handle.read_to_end(&mut stderr).context("read timed out local smoke stderr")?;
            }
            let timed_out_stderr = format!(
                "{}\nlocal smoke command exceeded timeout of {}s",
                String::from_utf8_lossy(&stderr),
                timeout.as_secs()
            )
            .trim()
            .to_string();
            return Ok((
                status,
                String::from_utf8_lossy(&stdout).to_string(),
                timed_out_stderr,
                true,
            ));
        }

        std::thread::sleep(Duration::from_millis(25));
    }
}

pub(crate) fn now_unix_s() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

pub(crate) fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

pub(crate) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
