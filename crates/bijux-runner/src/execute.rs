use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use bijux_core::hashing::{params_hash, run_id_from_hashes};
use bijux_core::StagePlanV1;
use bijux_environment::api::RunnerKind;
use sha2::Digest;
use uuid::Uuid;

use crate::docker::executor::{docker_logs, docker_wait, docker_wait_timeout, parse_mem_to_mb};

#[derive(Debug, Clone)]
pub struct StageResultV1 {
    pub run_id: String,
    pub exit_code: i32,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub outputs: Vec<PathBuf>,
    pub metrics_path: Option<PathBuf>,
    pub stdout: String,
    pub stderr: String,
    pub command: String,
}

fn common_parent(paths: &[PathBuf]) -> Option<PathBuf> {
    let mut iter = paths.iter();
    let first = iter.next()?.clone();
    let mut prefix = first;
    while let Some(path) = iter.next() {
        while !path.starts_with(&prefix) {
            if !prefix.pop() {
                return None;
            }
        }
    }
    Some(prefix)
}

fn hash_inputs(inputs: &[PathBuf]) -> Result<String> {
    if inputs.is_empty() {
        return Ok("no-inputs".to_string());
    }
    let mut hashes = Vec::with_capacity(inputs.len());
    for path in inputs {
        if path.exists() {
            hashes.push(bijux_infra::hash_file_sha256(path)?);
        }
    }
    Ok(format!("{:x}", sha2::Sha256::digest(hashes.join("|").as_bytes())))
}

fn build_command_string(args: &[String]) -> String {
    if args.is_empty() {
        return String::new();
    }
    args.join(" ")
}

/// Execute a single stage plan using docker.
///
/// # Errors
/// Returns an error if execution fails or docker is unavailable.
pub fn execute_stage_plan(
    plan: &StagePlanV1,
    runner: RunnerKind,
    timeout: Option<Duration>,
) -> Result<StageResultV1> {
    if runner != RunnerKind::Docker {
        return Err(anyhow!("runner {:?} not supported for stage execution", runner));
    }
    let out_dir = &plan.out_dir;
    bijux_infra::ensure_dir(out_dir).context("ensure out dir")?;
    let inputs: Vec<PathBuf> = plan.io.inputs.iter().map(|input| input.path.clone()).collect();
    let input_root = common_parent(&inputs).unwrap_or_else(|| out_dir.clone());
    let input_mount = format!("{}:/data/input:ro", input_root.display());
    let output_mount = format!("{}:/data/output", out_dir.display());

    let container_name = format!("bijux-stage-{}", Uuid::new_v4());
    let mut cmd = Command::new("docker");
    let mut args: Vec<String> = Vec::new();
    args.push("run".to_string());
    args.push("-d".to_string());
    args.push("--rm=false".to_string());
    args.push("--name".to_string());
    args.push(container_name.clone());
    args.push("-v".to_string());
    args.push(input_mount);
    args.push("-v".to_string());
    args.push(output_mount);
    args.push(plan.image.image.clone());
    args.extend(plan.command.template.clone());

    let start = Instant::now();
    let output = cmd.args(&args).output().context("docker run")?;
    if !output.status.success() {
        return Err(anyhow!(
            "docker run failed for {}",
            plan.tool_id.0
        ));
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() {
        return Err(anyhow!("missing container id for {}", plan.tool_id.0));
    }
    let exit_code = if let Some(timeout) = timeout {
        docker_wait_timeout(&id, timeout)?
    } else {
        docker_wait(&id)?
    };
    let stdout = docker_logs(&id)?;
    let stderr = String::new();
    let runtime_s = start.elapsed().as_secs_f64();
    let memory_mb = parse_mem_to_mb("0MiB / 0MiB").unwrap_or(0.0);

    let outputs: Vec<PathBuf> = plan.io.outputs.iter().map(|output| output.path.clone()).collect();
    let input_hash = hash_inputs(&inputs)?;
    let params_hash = params_hash(&plan.params);
    let run_id = run_id_from_hashes(&input_hash, &params_hash);

    Ok(StageResultV1 {
        run_id,
        exit_code,
        runtime_s,
        memory_mb,
        outputs,
        metrics_path: None,
        stdout,
        stderr,
        command: build_command_string(&args),
    })
}
