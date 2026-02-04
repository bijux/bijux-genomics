use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use bijux_environment::api::ResolvedImage;

use super::docker::{command_string, docker_logs, docker_wait, docker_wait_timeout, push_arg};
use super::plan::StageExecutionPlan;

#[derive(Debug, Clone)]
pub struct ExecutionOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub command: String,
}

pub fn execute_plan(
    plan: &StageExecutionPlan,
    image: &ResolvedImage,
    input_mount: &Path,
    output_mount: &Path,
    container_name: &str,
) -> Result<ExecutionOutput> {
    let input_mount = format!("{}:/data/input:ro", input_mount.display());
    let output_mount = format!("{}:/data/output", output_mount.display());

    let mut cmd = Command::new("docker");
    let mut args: Vec<String> = Vec::new();
    push_arg(&mut cmd, &mut args, "run");
    push_arg(&mut cmd, &mut args, "-d");
    push_arg(&mut cmd, &mut args, "--rm=false");
    push_arg(&mut cmd, &mut args, "--name");
    push_arg(&mut cmd, &mut args, container_name);
    push_arg(&mut cmd, &mut args, "-v");
    push_arg(&mut cmd, &mut args, input_mount);
    push_arg(&mut cmd, &mut args, "-v");
    push_arg(&mut cmd, &mut args, output_mount);
    for (key, value) in &plan.env {
        push_arg(&mut cmd, &mut args, "-e");
        push_arg(&mut cmd, &mut args, format!("{key}={value}"));
    }
    push_arg(&mut cmd, &mut args, image.full_name.clone());
    for arg in &plan.container_args {
        push_arg(&mut cmd, &mut args, arg.clone());
    }

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

pub fn execute_plan_with_timeout(
    plan: &StageExecutionPlan,
    image: &ResolvedImage,
    input_mount: &Path,
    output_mount: &Path,
    container_name: &str,
    timeout: std::time::Duration,
) -> Result<ExecutionOutput> {
    let input_mount = format!("{}:/data/input:ro", input_mount.display());
    let output_mount = format!("{}:/data/output", output_mount.display());

    let mut cmd = Command::new("docker");
    let mut args: Vec<String> = Vec::new();
    push_arg(&mut cmd, &mut args, "run");
    push_arg(&mut cmd, &mut args, "-d");
    push_arg(&mut cmd, &mut args, "--rm=false");
    push_arg(&mut cmd, &mut args, "--name");
    push_arg(&mut cmd, &mut args, container_name);
    push_arg(&mut cmd, &mut args, "-v");
    push_arg(&mut cmd, &mut args, input_mount);
    push_arg(&mut cmd, &mut args, "-v");
    push_arg(&mut cmd, &mut args, output_mount);
    for (key, value) in &plan.env {
        push_arg(&mut cmd, &mut args, "-e");
        push_arg(&mut cmd, &mut args, format!("{key}={value}"));
    }
    push_arg(&mut cmd, &mut args, image.full_name.clone());
    for arg in &plan.container_args {
        push_arg(&mut cmd, &mut args, arg.clone());
    }

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
