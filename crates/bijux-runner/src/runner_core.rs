use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct CommandOutputV1 {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub runtime_s: f64,
    pub command: String,
}

fn build_command_string(command: &str, args: &[String]) -> String {
    if args.is_empty() {
        return command.to_string();
    }
    format!("{command} {}", args.join(" "))
}

/// # Errors
/// Returns an error if the command cannot be executed.
pub fn run_command(command: &str, args: &[String]) -> Result<CommandOutputV1> {
    let start = Instant::now();
    let output = Command::new(command)
        .args(args)
        .output()
        .with_context(|| format!("run command {command}"))?;
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
