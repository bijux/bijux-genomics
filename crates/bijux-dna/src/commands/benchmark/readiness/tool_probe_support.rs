use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{bail, Context, Result};
use bijux_dna_api::v1::api::run::run_command_with_context_and_timeout;

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
    spawn_command(repo_root, applied_command, &[], timeout, context_label)
}

pub(crate) fn run_command_with_timeout_and_env(
    repo_root: &Path,
    applied_command: &[String],
    envs: &[(String, String)],
    timeout: Duration,
    context_label: &str,
) -> Result<CommandExecution> {
    spawn_command(repo_root, applied_command, envs, timeout, context_label)
}

fn spawn_command(
    repo_root: &Path,
    applied_command: &[String],
    envs: &[(String, String)],
    timeout: Duration,
    context_label: &str,
) -> Result<CommandExecution> {
    let Some(program) = applied_command.first() else {
        bail!("{context_label} attempted to run an empty command");
    };
    let env_map = envs.iter().cloned().collect::<BTreeMap<_, _>>();
    let timed = run_command_with_context_and_timeout(
        program,
        &applied_command[1..],
        Some(repo_root),
        Some(&env_map),
        timeout,
    )
    .with_context(|| format!("{context_label} `{}`", applied_command.join(" ")))?;
    Ok(CommandExecution {
        exit_code: timed.output.exit_code,
        stdout: timed.output.stdout,
        stderr: timed.output.stderr,
        timed_out: timed.timed_out,
    })
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
