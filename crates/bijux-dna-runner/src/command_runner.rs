use std::path::Path;
use std::process::Command;
use std::time::Instant;

use anyhow::{bail, Context, Result};

mod command_line;
mod command_output;
mod invocation_identity;

use command_line::build_command_string;
pub use command_output::CommandOutputV1;
pub use invocation_identity::invocation_hash;

use std::collections::BTreeMap;

/// # Errors
/// Returns an error if the command cannot be executed.
pub fn run_command(command: &str, args: &[String]) -> Result<CommandOutputV1> {
    run_command_with_context(command, args, None, None)
}

/// # Errors
/// Returns an error if the command cannot be executed.
pub fn run_command_with_context(
    command: &str,
    args: &[String],
    current_dir: Option<&Path>,
    envs: Option<&BTreeMap<String, String>>,
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
    let output = child.output().with_context(|| format!("run command {command}"))?;
    let runtime_s = start.elapsed().as_secs_f64();
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok(CommandOutputV1 {
        stdout,
        stderr,
        exit_code,
        runtime_s,
        command: build_command_string(command, args),
    })
}

#[cfg(test)]
mod tests {
    use super::run_command_with_context;

    #[test]
    fn run_command_rejects_empty_executable() {
        let err = run_command_with_context("  ", &[], None, None)
            .expect_err("empty executable should be rejected before spawn");

        assert_eq!(err.to_string(), "command executable must not be empty");
    }
}
