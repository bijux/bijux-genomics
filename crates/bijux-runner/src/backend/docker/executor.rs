use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use bijux_environment::api::{
    docker_image_exists, resolve_image, PlatformSpec, ResolvedImage, ToolImageSpec,
};
use tracing::warn;

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

pub fn resolve_image_for_run(
    spec: &ToolImageSpec,
    platform: &PlatformSpec,
) -> Result<ResolvedImage> {
    let image = resolve_image(spec, platform)?;
    if std::env::var("BIJUX_SKIP_IMAGE_CHECK").is_ok() {
        return Ok(image);
    }
    if docker_image_exists(&image) {
        return Ok(image);
    }
    if spec.digest.is_some() {
        let fallback = ResolvedImage {
            full_name: format!(
                "{}/{}:{}-{}",
                platform.image_prefix, spec.tool, spec.version, platform.arch
            ),
            arch: platform.arch.clone(),
            runner: platform.runner,
        };
        if docker_image_exists(&fallback) {
            warn!(
                "digest image missing locally; falling back to tag {}",
                fallback.full_name
            );
            return Ok(fallback);
        }
    }
    Err(anyhow!("docker image not found: {}", image.full_name))
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

pub(crate) fn push_arg(cmd: &mut Command, args: &mut Vec<String>, value: impl Into<String>) {
    let value = value.into();
    cmd.arg(&value);
    args.push(value);
}

pub(crate) fn command_string(args: &[String]) -> String {
    format!("docker {}", args.join(" "))
}

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
        let dir = bijux_infra::temp_dir("bijux")?;
        let output = dir.path().join("out.data");
        bijux_infra::atomic_write_bytes(&output, b"ok")?;
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
        let dir = bijux_infra::temp_dir("bijux")?;
        let present = dir.path().join("present.data");
        bijux_infra::atomic_write_bytes(&present, b"ok")?;
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
