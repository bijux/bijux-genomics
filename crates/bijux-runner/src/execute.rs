use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use bijux_core::plan::execution_graph::ExecutionStep;
use bijux_core::primitives::hashing::{params_hash, run_id_from_hashes};
use bijux_environment::api::RunnerKind;
use uuid::Uuid;

use crate::docker::executor::{docker_logs, docker_wait, docker_wait_timeout, parse_mem_to_mb};
use crate::runner_core::{run_command, CommandOutputV1};

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
    for path in iter {
        while !path.starts_with(&prefix) {
            if !prefix.pop() {
                return None;
            }
        }
    }
    Some(prefix)
}

fn hash_inputs(inputs: &[PathBuf]) -> Result<Vec<String>> {
    if inputs.is_empty() {
        return Ok(Vec::new());
    }
    let mut hashes = Vec::with_capacity(inputs.len());
    for path in inputs {
        if path.exists() {
            hashes.push(bijux_infra::hash_file_sha256(path)?);
        }
    }
    Ok(hashes)
}

/// Execute a single stage plan using docker.
///
/// # Errors
/// Returns an error if execution fails or docker is unavailable.
pub fn execute_stage_plan(
    step: &ExecutionStep,
    runner: RunnerKind,
    timeout: Option<Duration>,
) -> Result<StageResultV1> {
    if runner != RunnerKind::Docker {
        return Err(anyhow!(
            "runner {runner:?} not supported for stage execution"
        ));
    }
    let out_dir = &step.out_dir;
    bijux_infra::ensure_dir(out_dir).context("ensure out dir")?;
    let inputs: Vec<PathBuf> = step
        .io
        .inputs
        .iter()
        .map(|input| input.path.clone())
        .collect();
    let input_root = common_parent(&inputs).unwrap_or_else(|| out_dir.clone());
    let input_mount = format!("{}:/data/input:ro", input_root.display());
    let output_mount = format!("{}:/data/output", out_dir.display());

    let container_name = format!("bijux-stage-{}", Uuid::new_v4());
    let mut args: Vec<String> = vec![
        "run".to_string(),
        "-d".to_string(),
        "--rm=false".to_string(),
        "--name".to_string(),
        container_name.clone(),
        "-v".to_string(),
        input_mount,
        "-v".to_string(),
        output_mount,
        step.image.image.clone(),
    ];
    args.extend(step.command.template.clone());

    let output = run_command("docker", &args).context("docker run")?;
    if output.exit_code != 0 {
        return Err(anyhow!("docker run failed for {}", step.step_id.0));
    }
    let id = output.stdout.trim().to_string();
    if id.is_empty() {
        return Err(anyhow!("missing container id for {}", step.step_id.0));
    }
    let exit_code = if let Some(timeout) = timeout {
        docker_wait_timeout(&id, timeout)?
    } else {
        docker_wait(&id)?
    };
    let stdout = docker_logs(&id)?;
    let stderr = String::new();
    let runtime_s = output.runtime_s;
    let memory_mb = parse_mem_to_mb("0MiB / 0MiB").unwrap_or(0.0);

    let outputs: Vec<PathBuf> = step
        .io
        .outputs
        .iter()
        .map(|output| output.path.clone())
        .collect();
    let input_hashes = hash_inputs(&inputs)?;
    let params_hash = params_hash(&serde_json::json!({ "command": step.command.template }))?;
    let run_id = run_id_from_hashes(
        "unknown_pipeline",
        "unknown_sample",
        &params_hash,
        &input_hashes,
        None,
    );

    Ok(StageResultV1 {
        run_id,
        exit_code,
        runtime_s,
        memory_mb,
        outputs,
        metrics_path: None,
        stdout,
        stderr,
        command: output.command,
    })
}

/// Execute a lightweight observer command using docker.
///
/// # Errors
/// Returns an error if execution fails or docker is unavailable.
pub fn execute_observer_command(
    image: &str,
    mount_dir: &PathBuf,
    args: &[String],
    runner: RunnerKind,
) -> Result<CommandOutputV1> {
    if runner != RunnerKind::Docker {
        return Err(anyhow!(
            "runner {runner:?} not supported for observer execution"
        ));
    }
    let mount_dir = mount_dir.canonicalize().context("resolve mount dir")?;
    let mount_arg = format!("{}:/data:ro", mount_dir.display());
    let mut command_args: Vec<String> = vec![
        "run".to_string(),
        "--rm".to_string(),
        "-v".to_string(),
        mount_arg,
        image.to_string(),
    ];
    command_args.extend(args.iter().cloned());
    let output = run_command("docker", &command_args).context("docker run")?;
    Ok(output)
}
