mod command_line;
mod lifecycle;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_environment::api::ResolvedImage;

pub use crate::backend::docker::image_resolution::resolve_image_for_run;
use command_line::{build_docker_run_command, command_string};
pub use lifecycle::{
    docker_logs, docker_rm, docker_stats_mb, docker_wait, docker_wait_timeout, parse_mem_to_mb,
};

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
