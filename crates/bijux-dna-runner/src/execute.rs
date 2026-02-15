use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_core::metrics::ToolInvocationV1;
use bijux_dna_core::prelude::cache::CacheKey;
use bijux_dna_core::prelude::hashing::{
    input_fingerprint, parameters_fingerprint, params_hash, run_id_from_hashes,
};
use bijux_dna_environment::api::RuntimeKind;
use uuid::Uuid;

use crate::backend::docker::executor::{
    docker_logs, docker_wait, docker_wait_timeout, parse_mem_to_mb,
};
use crate::runner_core::{run_command, CommandOutputV1};

#[derive(Debug, Clone, Copy)]
enum RunnerEffectKind {
    UnsupportedRuntime,
    Filesystem,
    CommandSpawn,
    ContainerLifecycle,
    TelemetryWrite,
}

impl RunnerEffectKind {
    const fn code(self) -> &'static str {
        match self {
            Self::UnsupportedRuntime => "unsupported_runtime",
            Self::Filesystem => "filesystem",
            Self::CommandSpawn => "command_spawn",
            Self::ContainerLifecycle => "container_lifecycle",
            Self::TelemetryWrite => "telemetry_write",
        }
    }
}

fn runner_failure(kind: RunnerEffectKind, message: impl Into<String>) -> anyhow::Error {
    anyhow!("[runner_effect:{}] {}", kind.code(), message.into())
}

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
            hashes.push(bijux_dna_infra::hash_file_sha256(path)?);
        }
    }
    Ok(hashes)
}

fn build_apptainer_exec_args(
    step: &ExecutionStep,
    input_root: &Path,
    out_dir: &Path,
    runner: RuntimeKind,
) -> Result<Vec<String>> {
    let input_mount = format!("{}:/data/input:ro", input_root.display());
    let output_mount = format!("{}:/data/output", out_dir.display());
    let mut args: Vec<String> = vec![
        "exec".to_string(),
        "--cleanenv".to_string(),
        "--no-home".to_string(),
        "--containall".to_string(),
        "--bind".to_string(),
        input_mount,
        "--bind".to_string(),
        output_mount,
    ];
    if !network_allowed() {
        match runner {
            RuntimeKind::Apptainer => {
                args.push("--net".to_string());
            }
            RuntimeKind::Singularity => {}
            RuntimeKind::Docker => {}
        }
    }
    args.push(step.image.image.clone());
    args.extend(step.command.template.clone());
    if args.is_empty() {
        return Err(runner_failure(
            RunnerEffectKind::CommandSpawn,
            "apptainer/singularity command args are empty",
        ));
    }
    Ok(args)
}

/// Execute a single step using docker.
///
/// # Errors
/// Returns an error if execution fails or docker is unavailable.
#[allow(clippy::too_many_lines)]
pub fn execute_step(
    step: &ExecutionStep,
    runner: RuntimeKind,
    timeout: Option<Duration>,
) -> Result<StageResultV1> {
    let out_dir = &step.out_dir;
    bijux_dna_infra::ensure_dir(out_dir)
        .map_err(|err| runner_failure(RunnerEffectKind::Filesystem, err.to_string()))?;
    let inputs: Vec<PathBuf> = step
        .io
        .inputs
        .iter()
        .map(|input| input.path.clone())
        .collect();
    let input_root = common_parent(&inputs).unwrap_or_else(|| out_dir.clone());
    let input_mount = format!("{}:/data/input:ro", input_root.display());
    let output_mount = format!("{}:/data/output", out_dir.display());

    let (output, exit_code, stdout, stderr, runtime_s, memory_mb) = match runner {
        RuntimeKind::Docker => {
            let container_name = format!("bijux-dna-stage-{}", Uuid::new_v4());
            let mut args: Vec<String> = vec![
                "run".to_string(),
                "-d".to_string(),
                "--rm=false".to_string(),
                "--name".to_string(),
                container_name.clone(),
            ];
            if !network_allowed() {
                args.push("--network".to_string());
                args.push("none".to_string());
            }
            args.extend([
                "-v".to_string(),
                input_mount,
                "-v".to_string(),
                output_mount,
                step.image.image.clone(),
            ]);
            args.extend(step.command.template.clone());

            let output = run_command("docker", &args)
                .map_err(|err| runner_failure(RunnerEffectKind::CommandSpawn, err.to_string()))?;
            if output.exit_code != 0 {
                return Err(runner_failure(
                    RunnerEffectKind::ContainerLifecycle,
                    format!("docker run failed for {}", step.step_id.0),
                ));
            }
            let id = output.stdout.trim().to_string();
            if id.is_empty() {
                return Err(runner_failure(
                    RunnerEffectKind::ContainerLifecycle,
                    format!("missing container id for {}", step.step_id.0),
                ));
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
            (output, exit_code, stdout, stderr, runtime_s, memory_mb)
        }
        RuntimeKind::Apptainer | RuntimeKind::Singularity => {
            let args = build_apptainer_exec_args(step, &input_root, out_dir, runner)?;
            let bin = if runner == RuntimeKind::Apptainer {
                "apptainer"
            } else {
                "singularity"
            };
            let output = run_command(bin, &args)
                .map_err(|err| runner_failure(RunnerEffectKind::CommandSpawn, err.to_string()))?;
            let exit_code = output.exit_code;
            let stdout = output.stdout.clone();
            let stderr = output.stderr.clone();
            let runtime_s = output.runtime_s;
            let memory_mb = 0.0;
            (output, exit_code, stdout, stderr, runtime_s, memory_mb)
        }
    };

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
    runner: RuntimeKind,
) -> Result<CommandOutputV1> {
    if runner != RuntimeKind::Docker {
        return Err(runner_failure(
            RunnerEffectKind::UnsupportedRuntime,
            format!("runner {runner:?} not supported for observer execution"),
        ));
    }
    let mount_dir = mount_dir
        .canonicalize()
        .map_err(|err| runner_failure(RunnerEffectKind::Filesystem, err.to_string()))?;
    let mount_arg = format!("{}:/data:ro", mount_dir.display());
    let mut command_args: Vec<String> = vec!["run".to_string(), "--rm".to_string()];
    if !network_allowed() {
        command_args.push("--network".to_string());
        command_args.push("none".to_string());
    }
    command_args.extend(["-v".to_string(), mount_arg, image.to_string()]);
    command_args.extend(args.iter().cloned());
    let output = run_command("docker", &command_args)
        .map_err(|err| runner_failure(RunnerEffectKind::CommandSpawn, err.to_string()))?;
    Ok(output)
}

fn network_allowed() -> bool {
    std::env::var("BIJUX_ALLOW_NETWORK")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}

fn infer_tool_version_from_image(image: &str) -> String {
    let without_digest = image.split('@').next().unwrap_or(image);
    if let Some((_, tag)) = without_digest.rsplit_once(':') {
        if !tag.is_empty() && tag != "latest" {
            return tag.to_string();
        }
    }
    "unknown".to_string()
}

fn write_minimum_run_artifacts(
    step: &ExecutionStep,
    input_hashes: &[String],
    output_hashes: &[String],
    runner: RuntimeKind,
    command: &str,
) -> Result<()> {
    let run_artifacts_dir = step.out_dir.join("run_artifacts");
    bijux_dna_infra::ensure_dir(&run_artifacts_dir)
        .map_err(|err| runner_failure(RunnerEffectKind::Filesystem, err.to_string()))?;

    let metrics_path = run_artifacts_dir.join("metrics.json");
    if !metrics_path.exists() {
        bijux_dna_infra::atomic_write_json(&metrics_path, &serde_json::json!({}))
            .map_err(|err| runner_failure(RunnerEffectKind::TelemetryWrite, err.to_string()))?;
    }

    let effective_config_path = run_artifacts_dir.join("effective_config.json");
    if !effective_config_path.exists() {
        let payload = serde_json::json!({
            "command": step.command.template,
            "image": step.image,
            "resources": step.resources,
        });
        bijux_dna_infra::atomic_write_json(&effective_config_path, &payload)
            .map_err(|err| runner_failure(RunnerEffectKind::TelemetryWrite, err.to_string()))?;
    }

    let tool_invocation_path = run_artifacts_dir.join("tool_invocation.json");
    if !tool_invocation_path.exists() {
        let inferred_tool_version = infer_tool_version_from_image(&step.image.image);
        let parameters_json = serde_json::json!({ "command": step.command.template });
        let params_provenance = serde_json::json!({
            "tool_params": parameters_json,
            "defaults": serde_json::json!({}),
            "overrides": serde_json::json!({}),
            "effective_params": serde_json::json!({}),
        });
        let params_provenance_normalized =
            bijux_dna_core::contract::canonical::canonicalize_json_value(&params_provenance);
        let invocation = ToolInvocationV1::new(
            "bijux.tool_invocation.v1".to_string(),
            bijux_dna_core::contract::ContractVersion::v1(),
            step.stage_id.clone(),
            bijux_dna_core::ids::ToolId::new(step.image.image.clone()),
            inferred_tool_version.clone(),
            None,
            step.image
                .digest
                .clone()
                .unwrap_or_else(|| step.image.image.clone()),
            format!("{runner:?}"),
            inferred_tool_version.clone(),
            parameters_json.clone(),
            parameters_json,
            serde_json::json!({}),
            serde_json::json!({}),
            params_provenance,
            params_provenance_normalized,
            step.resources.clone(),
            std::collections::BTreeMap::new(),
            input_hashes.to_vec(),
            output_hashes.to_vec(),
            Some(command.to_string()),
        );
        bijux_dna_infra::atomic_write_json(&tool_invocation_path, &invocation)
            .context("write tool_invocation.json")?;
    }

    let stage_report_path = run_artifacts_dir.join("stage_report.json");
    if !stage_report_path.exists() {
        let inferred_tool_version = infer_tool_version_from_image(&step.image.image);
        let summary_params_hash =
            params_hash(&serde_json::json!({ "command": step.command.template.clone() }))
                .unwrap_or_else(|_| "unknown".to_string());
        let payload = serde_json::json!({
            "schema_version": "bijux.stage_report.v1",
            "stage_id": step.stage_id.to_string(),
            "stage_version": 1,
            "tool_id": step.image.image.clone(),
            "tool_version": inferred_tool_version,
            "metrics_path": metrics_path.display().to_string(),
            "tool_invocation_path": tool_invocation_path.display().to_string(),
            "effective_config_path": effective_config_path.display().to_string(),
            "effective_config_hash": null,
            "facts_row_id": null,
            "summary": {
                "metric_provenance": {
                    "run_id": std::env::var("BIJUX_RUN_ID").unwrap_or_else(|_| "unknown".to_string()),
                    "stage_id": step.stage_id.to_string(),
                    "tool_id": step.image.image.clone(),
                    "tool_version": inferred_tool_version,
                    "params_hash": summary_params_hash,
                    "input_artifact_hashes": input_hashes,
                }
            },
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
        bijux_dna_infra::atomic_write_json(&stage_report_path, &payload)
            .context("write stage_report.json")?;
    }

    Ok(())
}
