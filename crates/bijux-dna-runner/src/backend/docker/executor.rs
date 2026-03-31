mod command_line;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use bijux_dna_environment::api::ResolvedImage;

pub use crate::backend::docker::image_resolution::resolve_image_for_run;
use command_line::{build_docker_run_command, command_string};

#[derive(Debug, Clone)]
pub struct StageExecutionPlan {
    pub tool: String,
    pub container_args: Vec<String>,
    pub expected_outputs: Vec<PathBuf>,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ExecutionOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct ExecutionAssessment {
    pub success: bool,
    pub missing_outputs: Vec<PathBuf>,
    pub reason: Option<String>,
}

/// Execute a container plan and collect command output.
///
/// # Errors
/// Returns an error if Docker invocation fails or the container cannot be observed.
pub fn execute_plan(
    plan: &StageExecutionPlan,
    image: &ResolvedImage,
    input_mount: &Path,
    output_mount: &Path,
    container_name: &str,
) -> Result<ExecutionOutput> {
    let (mut cmd, args) =
        build_docker_run_command(plan, image, input_mount, output_mount, container_name);

    let output = cmd.output().context("run docker")?;
    if !output.status.success() {
        return Err(anyhow!("docker run failed for {}", plan.tool));
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() {
        return Err(anyhow!("missing container id for {}", plan.tool));
    }
    let exit_code = docker_wait(&id)?;
    let stdout = docker_logs(&id)?;
    let stderr = String::new();
    let command = command_string(&args);
    Ok(ExecutionOutput {
        exit_code,
        stdout,
        stderr,
        command,
    })
}

/// Execute a container plan with timeout enforcement.
///
/// # Errors
/// Returns an error if execution fails or timeout is reached.
pub fn execute_plan_with_timeout(
    plan: &StageExecutionPlan,
    image: &ResolvedImage,
    input_mount: &Path,
    output_mount: &Path,
    container_name: &str,
    timeout: std::time::Duration,
) -> Result<ExecutionOutput> {
    let (mut cmd, args) =
        build_docker_run_command(plan, image, input_mount, output_mount, container_name);

    let output = cmd.output().context("run docker")?;
    if !output.status.success() {
        return Err(anyhow!("docker run failed for {}", plan.tool));
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() {
        return Err(anyhow!("missing container id for {}", plan.tool));
    }
    let exit_code = docker_wait_timeout(&id, timeout)?;
    let stdout = docker_logs(&id)?;
    let stderr = String::new();
    let command = command_string(&args);
    Ok(ExecutionOutput {
        exit_code,
        stdout,
        stderr,
        command,
    })
}

#[must_use]
pub fn assess_execution(exit_code: i32, expected_outputs: &[PathBuf]) -> ExecutionAssessment {
    if exit_code != 0 {
        return ExecutionAssessment {
            success: false,
            missing_outputs: Vec::new(),
            reason: Some(format!("exit_code={exit_code}")),
        };
    }
    let missing: Vec<PathBuf> = expected_outputs
        .iter()
        .filter(|path| !path.exists())
        .cloned()
        .collect();
    if !missing.is_empty() {
        return ExecutionAssessment {
            success: false,
            missing_outputs: missing,
            reason: Some("missing_outputs".to_string()),
        };
    }
    ExecutionAssessment {
        success: true,
        missing_outputs: Vec::new(),
        reason: None,
    }
}

/// Wait for container completion and parse its exit code.
///
/// # Errors
/// Returns an error if docker wait fails or output cannot be parsed.
pub fn docker_wait(container_id: &str) -> Result<i32> {
    let output = Command::new("docker")
        .arg("wait")
        .arg(container_id)
        .output()
        .context("docker wait")?;
    if !output.status.success() {
        return Err(anyhow!("docker wait failed for {container_id}"));
    }
    let code = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<i32>()
        .context("parse docker wait output")?;
    Ok(code)
}

/// Wait for completion up to a timeout and return the container exit code.
///
/// # Errors
/// Returns an error if timeout is reached or docker inspection/wait fails.
pub fn docker_wait_timeout(container_id: &str, timeout: std::time::Duration) -> Result<i32> {
    let start = std::time::Instant::now();
    loop {
        let output = Command::new("docker")
            .arg("inspect")
            .arg(container_id)
            .arg("--format")
            .arg("{{.State.Status}}")
            .output()
            .context("docker inspect")?;
        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if status == "exited" {
                return docker_wait(container_id);
            }
        }
        if start.elapsed() >= timeout {
            return Err(anyhow!("timeout"));
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

/// Fetch container logs from Docker.
///
/// # Errors
/// Returns an error if docker logs command fails.
pub fn docker_logs(container_id: &str) -> Result<String> {
    let output = Command::new("docker")
        .arg("logs")
        .arg(container_id)
        .output()
        .context("docker logs")?;
    if !output.status.success() {
        return Err(anyhow!("docker logs failed for {container_id}"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Read current memory usage (MB) from docker stats.
///
/// # Errors
/// Returns an error if docker stats command fails or parsing is invalid.
pub fn docker_stats_mb(container_id: &str) -> Result<f64> {
    let output = Command::new("docker")
        .arg("stats")
        .arg("--no-stream")
        .arg("--format")
        .arg("{{.MemUsage}}")
        .arg(container_id)
        .output()
        .context("docker stats")?;
    if !output.status.success() {
        return Err(anyhow!("docker stats failed for {container_id}"));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mem = stdout
        .lines()
        .next()
        .ok_or_else(|| anyhow!("missing docker stats output"))?;
    parse_mem_to_mb(mem)
}

/// Parse docker memory usage string (e.g. `123.4MiB / 4GiB`) into MB.
///
/// # Errors
/// Returns an error if the input format or unit is unsupported.
pub fn parse_mem_to_mb(value: &str) -> Result<f64> {
    let parts: Vec<&str> = value.split('/').collect();
    let value = parts
        .first()
        .ok_or_else(|| anyhow!("invalid memory format"))?
        .trim();
    let mut number = String::new();
    let mut unit = String::new();
    for ch in value.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            number.push(ch);
        } else {
            unit.push(ch);
        }
    }
    let num: f64 = number.parse().context("parse memory value")?;
    let mb = match unit.as_str() {
        "B" => num / 1024.0 / 1024.0,
        "KiB" => num / 1024.0,
        "MiB" => num,
        "GiB" => num * 1024.0,
        _ => return Err(anyhow!("unknown memory unit: {unit}")),
    };
    Ok(mb)
}

/// Remove a container forcefully.
///
/// # Errors
/// Returns an error if docker rm fails.
pub fn docker_rm(container_id: &str) -> Result<()> {
    let output = Command::new("docker")
        .arg("rm")
        .arg("-f")
        .arg(container_id)
        .output()
        .context("docker rm")?;
    if !output.status.success() {
        return Err(anyhow!("docker rm failed for {container_id}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::assess_execution;
    use std::path::PathBuf;

    #[test]
    fn assess_execution_success() -> anyhow::Result<()> {
        let dir = bijux_dna_infra::temp_dir("bijux")?;
        let output = dir.path().join("out.data");
        bijux_dna_infra::atomic_write_bytes(&output, b"ok")?;
        let assessment = assess_execution(0, &[output]);
        assert!(assessment.success);
        Ok(())
    }

    #[test]
    fn assess_execution_missing_outputs() {
        let missing = PathBuf::from("/tmp/missing.data");
        let assessment = assess_execution(0, &[missing]);
        assert!(!assessment.success);
        assert_eq!(assessment.reason.as_deref(), Some("missing_outputs"));
    }

    #[test]
    fn assess_execution_partial_outputs() -> anyhow::Result<()> {
        let dir = bijux_dna_infra::temp_dir("bijux")?;
        let present = dir.path().join("present.data");
        bijux_dna_infra::atomic_write_bytes(&present, b"ok")?;
        let missing = dir.path().join("missing.data");
        let assessment = assess_execution(0, &[present, missing]);
        assert!(!assessment.success);
        assert_eq!(assessment.reason.as_deref(), Some("missing_outputs"));
        Ok(())
    }

    #[test]
    fn assess_execution_bad_exit_code() {
        let assessment = assess_execution(1, &[]);
        assert!(!assessment.success);
        assert_eq!(assessment.reason.as_deref(), Some("exit_code=1"));
    }
}
