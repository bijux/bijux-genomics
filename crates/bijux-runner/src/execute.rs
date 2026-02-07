use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use bijux_core::contract::ExecutionStep;
use bijux_core::metrics::ToolInvocationV1;
use bijux_core::prelude::cache::CacheKey;
use bijux_core::prelude::hashing::{input_fingerprint, parameters_fingerprint, run_id_from_hashes};
use bijux_environment::api::RunnerKind;
use uuid::Uuid;

use crate::backend::docker::executor::{
    docker_logs, docker_wait, docker_wait_timeout, parse_mem_to_mb,
};
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

/// Execute a single step using docker.
///
/// # Errors
/// Returns an error if execution fails or docker is unavailable.
pub fn execute_step(
    step: &ExecutionStep,
    runner: RunnerKind,
    timeout: Option<Duration>,
) -> Result<StageResultV1> {
    if runner != RunnerKind::Docker {
        return Err(anyhow!(
            "runner {runner:?} not supported for step execution"
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
    let output_hashes = hash_inputs(&outputs)?;
    let params_fingerprint =
        parameters_fingerprint(&serde_json::json!({ "command": step.command.template }))?;
    let input_fingerprint = input_fingerprint(&input_hashes);
    let env_digest = step
        .image
        .digest
        .clone()
        .unwrap_or_else(|| step.image.image.clone());
    let _cache_key = CacheKey::new(
        input_fingerprint,
        params_fingerprint.clone(),
        step.image.image.clone(),
        env_digest,
    );
    let run_id = run_id_from_hashes(
        "unknown_pipeline",
        "unknown_sample",
        &params_fingerprint,
        &input_hashes,
        None,
    );
    write_minimum_run_artifacts(step, &input_hashes, &output_hashes, runner, &output.command)?;

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
    mount_dir: &Path,
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

fn write_minimum_run_artifacts(
    step: &ExecutionStep,
    input_hashes: &[String],
    output_hashes: &[String],
    runner: RunnerKind,
    command: &str,
) -> Result<()> {
    let run_artifacts_dir = step.out_dir.join("run_artifacts");
    bijux_infra::ensure_dir(&run_artifacts_dir).context("ensure run_artifacts dir")?;

    let metrics_path = run_artifacts_dir.join("metrics.json");
    if !metrics_path.exists() {
        bijux_infra::atomic_write_json(&metrics_path, &serde_json::json!({}))
            .context("write metrics.json")?;
    }

    let effective_config_path = run_artifacts_dir.join("effective_config.json");
    if !effective_config_path.exists() {
        let payload = serde_json::json!({
            "command": step.command.template,
            "image": step.image,
            "resources": step.resources,
        });
        bijux_infra::atomic_write_json(&effective_config_path, &payload)
            .context("write effective_config.json")?;
    }

    let tool_invocation_path = run_artifacts_dir.join("tool_invocation.json");
    if !tool_invocation_path.exists() {
        let parameters_json = serde_json::json!({ "command": step.command.template });
        let params_provenance = serde_json::json!({
            "tool_params": parameters_json,
            "defaults": serde_json::json!({}),
            "overrides": serde_json::json!({}),
            "effective_params": serde_json::json!({}),
        });
        let params_provenance_normalized =
            bijux_core::contract::canonical::canonicalize_json_value(&params_provenance);
        let invocation = ToolInvocationV1 {
            schema_version: "bijux.tool_invocation.v1".to_string(),
            contract_version: bijux_core::contract::ContractVersion::v1(),
            stage_id: step.stage_id.clone(),
            tool_id: bijux_core::ids::ToolId::new(step.image.image.clone()),
            tool_version: "unknown".to_string(),
            resolved_tool_version: None,
            image_digest: step
                .image
                .digest
                .clone()
                .unwrap_or_else(|| step.image.image.clone()),
            runner_kind: format!("{runner:?}"),
            platform: "unknown".to_string(),
            parameters_json: parameters_json.clone(),
            parameters_json_normalized: parameters_json,
            effective_params_json: serde_json::json!({}),
            effective_params_json_normalized: serde_json::json!({}),
            params_provenance,
            params_provenance_normalized,
            adapter_bank: None,
            banks: None,
            bank_assets: None,
            resources: step.resources.clone(),
            environment: std::collections::BTreeMap::new(),
            input_hashes: input_hashes.to_vec(),
            output_hashes: output_hashes.to_vec(),
            executed_command: Some(command.to_string()),
        };
        bijux_infra::atomic_write_json(&tool_invocation_path, &invocation)
            .context("write tool_invocation.json")?;
    }

    let stage_report_path = run_artifacts_dir.join("stage_report.json");
    if !stage_report_path.exists() {
        let payload = serde_json::json!({
            "schema_version": "bijux.stage_report.v1",
            "stage_id": step.stage_id.to_string(),
            "stage_version": 1,
            "tool_id": step.image.image,
            "tool_version": "unknown",
            "metrics_path": metrics_path.display().to_string(),
            "tool_invocation_path": tool_invocation_path.display().to_string(),
            "effective_config_path": effective_config_path.display().to_string(),
            "effective_config_hash": null,
            "facts_row_id": null,
            "summary": {},
            "warnings": [],
            "errors": [],
            "invariants": [],
            "verdict": null,
            "outputs": step
                .io
                .outputs
                .iter()
                .map(|output| output.path.display().to_string())
                .collect::<Vec<_>>(),
            "subreports": [],
            "log_paths": [],
        });
        bijux_infra::atomic_write_json(&stage_report_path, &payload)
            .context("write stage_report.json")?;
    }

    Ok(())
}
