use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result};
use sha2::Digest;

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
    let start = Instant::now();
    let mut child = Command::new(command);
    child.args(args);
    if let Some(current_dir) = current_dir {
        child.current_dir(current_dir);
    }
    if let Some(envs) = envs {
        child.envs(envs);
    }
    let output = child
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

/// Compute a stable invocation hash for a runner invocation.
///
/// Inputs include argv, env, image digest, and input hashes.
/// # Errors
/// Returns an error if canonical serialization fails.
pub fn invocation_hash(
    argv: &[String],
    env: &BTreeMap<String, String>,
    image_digest: &str,
    input_hashes: &[String],
) -> Result<String> {
    let mut inputs = input_hashes.to_vec();
    inputs.sort();
    let payload = serde_json::json!({
        "argv": argv,
        "env": env,
        "image_digest": image_digest,
        "inputs": inputs,
    });
    let bytes = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&payload)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}
