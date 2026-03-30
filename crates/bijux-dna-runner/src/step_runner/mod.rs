use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_core::metrics::{ToolInvocationSpecV1, ToolInvocationV1};
use bijux_dna_core::prelude::cache::CacheKey;
use bijux_dna_core::prelude::hashing::{
    input_fingerprint, parameters_fingerprint, run_id_from_hashes,
};
use bijux_dna_environment::api::RuntimeKind;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::backend::docker::executor::{
    docker_logs, docker_wait, docker_wait_timeout, parse_mem_to_mb,
};
use crate::command_runner::{run_command, CommandOutputV1};

#[derive(Debug, Clone, Copy)]
enum RunnerEffectKind {
    Filesystem,
    CommandSpawn,
    ContainerLifecycle,
    TelemetryWrite,
}

impl RunnerEffectKind {
    const fn code(self) -> &'static str {
        match self {
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
            hashes.push(hash_path(path)?);
        }
    }
    Ok(hashes)
}

fn hash_path(path: &Path) -> Result<String> {
    if path.is_file() {
        return Ok(bijux_dna_infra::hash_file_sha256(path)?);
    }
    if path.is_dir() {
        return hash_directory(path);
    }
    Err(anyhow!(
        "unsupported hash input path type: {}",
        path.display()
    ))
}

fn hash_directory(root: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut entries = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("walk directory for hashing")?;
    entries.sort_by(|left, right| left.path().cmp(right.path()));

    for entry in entries {
        let path = entry.path();
        if path == root {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .with_context(|| format!("strip directory prefix for {}", path.display()))?;
        hasher.update(relative.to_string_lossy().as_bytes());
        if entry.file_type().is_dir() {
            hasher.update(b"\0dir\0");
            continue;
        }
        if entry.file_type().is_symlink() {
            hasher.update(b"\0symlink\0");
            let target = std::fs::read_link(path)
                .with_context(|| format!("read directory hash symlink {}", path.display()))?;
            hasher.update(target.to_string_lossy().as_bytes());
            continue;
        }
        hasher.update(b"\0file\0");
        hasher.update(bijux_dna_infra::hash_file_sha256(path)?.as_bytes());
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn container_input_mapping(input_root: &Path) -> (PathBuf, String) {
    if input_root.is_file() {
        let mount_root = input_root
            .parent()
            .map_or_else(|| PathBuf::from("/"), Path::to_path_buf);
        let file_name = input_root.file_name().map_or_else(
            || "input".to_string(),
            |name| name.to_string_lossy().into_owned(),
        );
        return (mount_root, format!("/data/input/{file_name}"));
    }
    (input_root.to_path_buf(), "/data/input".to_string())
}

fn preserve_absolute_input_paths(inputs: &[PathBuf]) -> bool {
    common_parent(inputs).is_some_and(|path| path == Path::new("/"))
}

fn input_bind_root(input: &Path) -> PathBuf {
    if input.exists() {
        if input.is_file() {
            return input
                .parent()
                .map_or_else(|| PathBuf::from("/"), Path::to_path_buf);
        }
        return input.to_path_buf();
    }
    input
        .parent()
        .map_or_else(|| PathBuf::from("/"), Path::to_path_buf)
}

fn collapse_bind_roots(mut roots: Vec<PathBuf>) -> Vec<PathBuf> {
    roots.sort_by(|left, right| {
        left.components()
            .count()
            .cmp(&right.components().count())
            .then_with(|| left.cmp(right))
    });
    let mut collapsed = Vec::<PathBuf>::new();
    for root in roots {
        if collapsed.iter().any(|existing| root.starts_with(existing)) {
            continue;
        }
        collapsed.push(root);
    }
    collapsed
}

fn input_bind_roots(
    inputs: &[PathBuf],
    input_root: &Path,
    preserve_absolute: bool,
) -> Vec<PathBuf> {
    if !preserve_absolute {
        return vec![container_input_mapping(input_root).0];
    }
    collapse_bind_roots(inputs.iter().map(|input| input_bind_root(input)).collect())
}

fn rewrite_container_path(value: &str, host_root: &Path, container_root: &str) -> String {
    let host_root_path = host_root;
    let host_root = host_root_path.display().to_string();
    let rewritten = value.replace(&host_root, container_root);
    if value == host_root {
        return container_root.to_string();
    }
    if host_root_path.is_file() {
        return rewritten;
    }
    let host_prefix = format!("{host_root}/");
    let container_prefix = format!("{container_root}/");
    rewritten.replace(&host_prefix, &container_prefix)
}

fn container_command_template(
    template: &[String],
    input_root: &Path,
    out_dir: &Path,
    preserve_absolute_inputs: bool,
) -> Vec<String> {
    let (_, container_input_root) = container_input_mapping(input_root);
    template
        .iter()
        .map(|part| {
            let rewritten_output = rewrite_container_path(part, out_dir, "/data/output");
            if preserve_absolute_inputs {
                rewritten_output
            } else {
                rewrite_container_path(&rewritten_output, input_root, &container_input_root)
            }
        })
        .collect()
}

fn build_apptainer_exec_args(
    step: &ExecutionStep,
    inputs: &[PathBuf],
    input_root: &Path,
    out_dir: &Path,
    _runner: RuntimeKind,
) -> Result<Vec<String>> {
    let preserve_absolute_inputs = preserve_absolute_input_paths(inputs);
    let bind_roots = input_bind_roots(inputs, input_root, preserve_absolute_inputs);
    let output_mount = format!("{}:/data/output", out_dir.display());
    let mut args: Vec<String> = vec![
        "exec".to_string(),
        "--cleanenv".to_string(),
        "--no-home".to_string(),
        "--containall".to_string(),
    ];
    for bind_root in bind_roots {
        let input_mount = if preserve_absolute_inputs {
            format!("{}:{}:ro", bind_root.display(), bind_root.display())
        } else {
            format!("{}:/data/input:ro", bind_root.display())
        };
        args.push("--bind".to_string());
        args.push(input_mount);
    }
    args.push("--bind".to_string());
    args.push(output_mount);
    let workdir_in_container = if let Ok(workdir) = std::env::var("BIJUX_STAGE_WORKDIR") {
        let out_dir_prefix = format!("{}/", out_dir.display());
        if workdir.starts_with(&out_dir_prefix) {
            format!(
                "/data/output/{}",
                workdir.trim_start_matches(&out_dir_prefix)
            )
        } else {
            "/data/output".to_string()
        }
    } else {
        "/data/output".to_string()
    };
    args.push("--pwd".to_string());
    args.push(workdir_in_container);
    args.push(step.image.image.clone());
    args.extend(container_command_template(
        &step.command.template,
        input_root,
        out_dir,
        preserve_absolute_inputs,
    ));
    if args.is_empty() {
        return Err(runner_failure(
            RunnerEffectKind::CommandSpawn,
            "apptainer/singularity command args are empty",
        ));
    }
    Ok(args)
}

fn runtime_env_exports() -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for key in [
        "LC_ALL",
        "LANG",
        "TZ",
        "TMPDIR",
        "HOME",
        "XDG_CACHE_HOME",
        "BIJUX_CACHE_ROOT",
        "BIJUX_STAGE_THREADS",
        "BIJUX_STAGE_MEMORY_MB",
        "BIJUX_COMPRESSION_THREADS",
        "BIJUX_STAGE_SEED",
        "BIJUX_UMASK",
    ] {
        if let Ok(value) = std::env::var(key) {
            pairs.push((key.to_string(), value));
        }
    }
    pairs
}

fn configured_memory_mb(step: &ExecutionStep) -> f64 {
    if let Ok(value) = std::env::var("BIJUX_STAGE_MEMORY_MB") {
        if let Ok(parsed) = value.parse::<f64>() {
            if parsed.is_finite() && parsed > 0.0 {
                return parsed;
            }
        }
    }
    f64::from(step.resources.mem_gb.max(1)) * 1024.0
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
    let preserve_absolute_inputs = preserve_absolute_input_paths(&inputs);
    let bind_roots = input_bind_roots(&inputs, &input_root, preserve_absolute_inputs);
    let output_mount = format!("{}:/data/output", out_dir.display());
    let command_template = container_command_template(
        &step.command.template,
        &input_root,
        out_dir,
        preserve_absolute_inputs,
    );

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
            if let Ok(workdir) = std::env::var("BIJUX_STAGE_WORKDIR") {
                let out_dir_prefix = format!("{}/", out_dir.display());
                let workdir_in_container = if workdir.starts_with(&out_dir_prefix) {
                    format!(
                        "/data/output/{}",
                        workdir.trim_start_matches(&out_dir_prefix)
                    )
                } else {
                    "/data/output".to_string()
                };
                args.push("-w".to_string());
                args.push(workdir_in_container);
            }
            for (key, value) in runtime_env_exports() {
                args.push("-e".to_string());
                args.push(format!("{key}={value}"));
            }
            if !network_allowed() {
                args.push("--network".to_string());
                args.push("none".to_string());
            }
            for bind_root in &bind_roots {
                let input_mount = if preserve_absolute_inputs {
                    format!("{}:{}:ro", bind_root.display(), bind_root.display())
                } else {
                    format!("{}:/data/input:ro", bind_root.display())
                };
                args.push("-v".to_string());
                args.push(input_mount);
            }
            args.extend(["-v".to_string(), output_mount, step.image.image.clone()]);
            args.extend(command_template.clone());

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
            let args = build_apptainer_exec_args(step, &inputs, &input_root, out_dir, runner)?;
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
            let memory_mb = configured_memory_mb(step);
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
    let pipeline_id = execution_pipeline_identity(step);
    let sample_id = execution_sample_identity(step);
    let run_id = run_id_from_hashes(
        &pipeline_id,
        &sample_id,
        &params_fingerprint,
        &input_hashes,
        None,
    );
    write_minimum_run_artifacts(
        step,
        &input_hashes,
        &output_hashes,
        runner,
        &output.command,
        &run_id,
        &params_fingerprint,
    )?;

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
    let mount_dir = mount_dir
        .canonicalize()
        .map_err(|err| runner_failure(RunnerEffectKind::Filesystem, err.to_string()))?;
    let (bin, command_args) = build_observer_command_args(image, &mount_dir, args, runner);
    let output = run_command(bin, &command_args)
        .map_err(|err| runner_failure(RunnerEffectKind::CommandSpawn, err.to_string()))?;
    Ok(output)
}

fn build_observer_command_args(
    image: &str,
    mount_dir: &Path,
    args: &[String],
    runner: RuntimeKind,
) -> (&'static str, Vec<String>) {
    let mount_arg = format!("{}:/data:ro", mount_dir.display());
    match runner {
        RuntimeKind::Docker => {
            let mut command_args: Vec<String> = vec!["run".to_string(), "--rm".to_string()];
            if !network_allowed() {
                command_args.push("--network".to_string());
                command_args.push("none".to_string());
            }
            command_args.extend(["-v".to_string(), mount_arg, image.to_string()]);
            command_args.extend(args.iter().cloned());
            ("docker", command_args)
        }
        RuntimeKind::Apptainer | RuntimeKind::Singularity => {
            let mut command_args = vec![
                "exec".to_string(),
                "--cleanenv".to_string(),
                "--no-home".to_string(),
                "--containall".to_string(),
                "--bind".to_string(),
                mount_arg,
            ];
            command_args.push(image.to_string());
            command_args.extend(args.iter().cloned());
            let bin = if runner == RuntimeKind::Apptainer {
                "apptainer"
            } else {
                "singularity"
            };
            (bin, command_args)
        }
    }
}

fn network_allowed() -> bool {
    std::env::var("BIJUX_ALLOW_NETWORK")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}

fn execution_pipeline_identity(step: &ExecutionStep) -> String {
    std::env::var("BIJUX_PIPELINE_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| step.stage_id.to_string())
}

fn execution_sample_identity(step: &ExecutionStep) -> String {
    std::env::var("BIJUX_SAMPLE_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| step.step_id.to_string())
}

fn runtime_platform_identity(runner: RuntimeKind) -> String {
    std::env::var("BIJUX_PLATFORM")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| match runner {
            RuntimeKind::Docker => "docker".to_string(),
            RuntimeKind::Apptainer => "apptainer".to_string(),
            RuntimeKind::Singularity => "singularity".to_string(),
        })
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
    run_id: &str,
    params_fingerprint: &str,
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
        write_tool_invocation(
            step,
            runner,
            input_hashes,
            output_hashes,
            command,
            &tool_invocation_path,
        )?;
    }

    let stage_report_path = run_artifacts_dir.join("stage_report.json");
    if !stage_report_path.exists() {
        let inferred_tool_version = infer_tool_version_from_image(&step.image.image);
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
                    "run_id": run_id,
                    "stage_id": step.stage_id.to_string(),
                    "tool_id": step.image.image.clone(),
                    "tool_version": inferred_tool_version,
                    "params_hash": params_fingerprint,
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

fn write_tool_invocation(
    step: &ExecutionStep,
    runner: RuntimeKind,
    input_hashes: &[String],
    output_hashes: &[String],
    command: &str,
    tool_invocation_path: &Path,
) -> Result<()> {
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
    let invocation = ToolInvocationV1::new(ToolInvocationSpecV1 {
        schema_version: "bijux.tool_invocation.v1".to_string(),
        contract_version: bijux_dna_core::contract::ContractVersion::v1(),
        stage_id: step.stage_id.clone(),
        tool_id: bijux_dna_core::ids::ToolId::new(step.image.image.clone()),
        tool_version: inferred_tool_version.clone(),
        resolved_tool_version: None,
        image_digest: step
            .image
            .digest
            .clone()
            .unwrap_or_else(|| step.image.image.clone()),
        runner_kind: format!("{runner:?}"),
        platform: runtime_platform_identity(runner),
        parameters_json: parameters_json.clone(),
        parameters_json_normalized: parameters_json,
        effective_params_json: serde_json::json!({}),
        effective_params_json_normalized: serde_json::json!({}),
        params_provenance,
        params_provenance_normalized,
        resources: step.resources.clone(),
        environment: std::collections::BTreeMap::new(),
        input_hashes: input_hashes.to_vec(),
        output_hashes: output_hashes.to_vec(),
        executed_command: Some(command.to_string()),
    });
    bijux_dna_infra::atomic_write_json(tool_invocation_path, &invocation)
        .context("write tool_invocation.json")
}

#[cfg(test)]
mod tests {
    use super::{
        build_apptainer_exec_args, build_observer_command_args, container_command_template,
        container_input_mapping, execution_pipeline_identity, execution_sample_identity,
        hash_inputs, hash_path, runtime_platform_identity,
    };
    use anyhow::anyhow;
    use bijux_dna_core::contract::{ExecutionStep, StageIO, ToolConstraints};
    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRef, ArtifactRole, CommandSpecV1, ContainerImageRefV1, StageId, StepId,
    };
    use bijux_dna_environment::api::RuntimeKind;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    #[test]
    fn observer_args_use_docker_mounts_for_docker_runner() {
        let (bin, args) = build_observer_command_args(
            "bijuxdna/seqkit:latest-pinned-amd64",
            Path::new("/tmp/input"),
            &["stats".to_string(), "/data/reads.fastq.gz".to_string()],
            RuntimeKind::Docker,
        );

        assert_eq!(bin, "docker");
        assert!(args.starts_with(&["run".to_string(), "--rm".to_string()]));
        assert!(args.contains(&"/tmp/input:/data:ro".to_string()));
    }

    #[test]
    fn execution_identity_defaults_to_stage_and_step_ids() {
        let step = ExecutionStep {
            step_id: StepId::from_static("sample-0001.fastq.trim_reads.fastp"),
            stage_id: StageId::from_static("fastq.trim_reads"),
            command: CommandSpecV1 {
                template: vec!["fastp".to_string()],
            },
            image: ContainerImageRefV1 {
                image: "fastp:0.23.4".to_string(),
                digest: None,
            },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: Vec::new(),
                outputs: Vec::new(),
            },
            out_dir: PathBuf::from("/tmp/out"),
            aux_images: std::collections::BTreeMap::default(),
            expected_artifact_ids: Vec::new(),
            metrics_schema_ids: Vec::new(),
        };

        assert_eq!(execution_pipeline_identity(&step), "fastq.trim_reads");
        assert_eq!(
            execution_sample_identity(&step),
            "sample-0001.fastq.trim_reads.fastp"
        );
    }

    #[test]
    fn runtime_platform_identity_defaults_to_runner_name() {
        assert_eq!(runtime_platform_identity(RuntimeKind::Docker), "docker");
        assert_eq!(
            runtime_platform_identity(RuntimeKind::Apptainer),
            "apptainer"
        );
        assert_eq!(
            runtime_platform_identity(RuntimeKind::Singularity),
            "singularity"
        );
    }

    #[test]
    fn observer_args_use_apptainer_exec_for_apptainer_runner() {
        let (bin, args) = build_observer_command_args(
            "/containers/seqkit.sif",
            Path::new("/tmp/input"),
            &["stats".to_string(), "/data/reads.fastq.gz".to_string()],
            RuntimeKind::Apptainer,
        );

        assert_eq!(bin, "apptainer");
        assert!(args.starts_with(&[
            "exec".to_string(),
            "--cleanenv".to_string(),
            "--no-home".to_string(),
            "--containall".to_string()
        ]));
        assert!(args.contains(&"--bind".to_string()));
        assert!(args.contains(&"/tmp/input:/data:ro".to_string()));
        assert!(args.contains(&"/containers/seqkit.sif".to_string()));
    }

    #[test]
    fn apptainer_exec_defaults_workdir_to_output_mount() -> anyhow::Result<()> {
        let step = ExecutionStep {
            step_id: StepId::from_static("step.trim_reads.tool.seqkit"),
            stage_id: StageId::from_static("fastq.trim_reads"),
            command: CommandSpecV1 {
                template: vec!["seqkit".to_string(), "stats".to_string()],
            },
            image: ContainerImageRefV1 {
                image: "/containers/seqkit.sif".to_string(),
                digest: None,
            },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: vec![ArtifactRef::required(
                    ArtifactId::from_static("reads"),
                    Path::new("/tmp/input/sample.fastq.gz").to_path_buf(),
                    ArtifactRole::Reads,
                )],
                outputs: vec![ArtifactRef::required(
                    ArtifactId::from_static("report"),
                    Path::new("/tmp/out/report.json").to_path_buf(),
                    ArtifactRole::ReportJson,
                )],
            },
            out_dir: Path::new("/tmp/out").to_path_buf(),
            aux_images: std::collections::BTreeMap::default(),
            expected_artifact_ids: Vec::new(),
            metrics_schema_ids: Vec::new(),
        };

        let args = build_apptainer_exec_args(
            &step,
            &[PathBuf::from("/tmp/input/sample.fastq.gz")],
            Path::new("/tmp/input"),
            Path::new("/tmp/out"),
            RuntimeKind::Apptainer,
        )?;

        let pwd_index = args
            .iter()
            .position(|value| value == "--pwd")
            .ok_or_else(|| anyhow!("missing --pwd flag in apptainer args"))?;
        assert_eq!(args[pwd_index + 1], "/data/output");
        Ok(())
    }

    #[test]
    fn container_command_template_rewrites_mounted_input_and_output_paths() {
        let template = vec![
            "sh".to_string(),
            "-lc".to_string(),
            "validator /tmp/corpus/sample_0004_R1.fastq.gz > /tmp/out/validation_r1.log && printf '%s' /tmp/out/validation.json"
                .to_string(),
        ];

        let rewritten = container_command_template(
            &template,
            Path::new("/tmp/corpus"),
            Path::new("/tmp/out"),
            false,
        );

        assert_eq!(rewritten[0], "sh");
        assert!(rewritten[2].contains("/data/input/sample_0004_R1.fastq.gz"));
        assert!(rewritten[2].contains("/data/output/validation_r1.log"));
        assert!(rewritten[2].contains("/data/output/validation.json"));
    }

    #[test]
    fn container_command_template_rewrites_single_file_mounts_inside_shell_scripts(
    ) -> anyhow::Result<()> {
        let temp = tempdir()?;
        let input = temp.path().join("sample_0004_R1.fastq.gz");
        std::fs::write(&input, b"@read\nACGT\n+\n!!!!\n")?;
        let out_dir = temp.path().join("out");
        std::fs::create_dir_all(&out_dir)?;
        let template = vec![
            "sh".to_string(),
            "-lc".to_string(),
            format!(
                "seqkit fx2tab -j 1 -n -s {} > {}",
                input.display(),
                out_dir.join("reads.tsv").display()
            ),
        ];

        let rewritten = container_command_template(&template, &input, &out_dir, false);

        assert_eq!(rewritten[0], "sh");
        assert!(
            rewritten[2].contains("seqkit fx2tab -j 1 -n -s /data/input/sample_0004_R1.fastq.gz")
        );
        assert!(rewritten[2].contains("> /data/output/reads.tsv"));
        Ok(())
    }

    #[test]
    fn container_command_template_rewrites_exact_output_root_inside_shell_scripts() {
        let template = vec![
            "sh".to_string(),
            "-lc".to_string(),
            "flash2 -o flash2 -d /tmp/out -t 1 /tmp/corpus/sample_0004_R1.fastq.gz /tmp/corpus/sample_0004_R2.fastq.gz"
                .to_string(),
        ];

        let rewritten = container_command_template(
            &template,
            Path::new("/tmp/corpus"),
            Path::new("/tmp/out"),
            false,
        );

        assert_eq!(rewritten[0], "sh");
        assert!(rewritten[2].contains("-d /data/output"));
        assert!(rewritten[2].contains("/data/input/sample_0004_R1.fastq.gz"));
        assert!(rewritten[2].contains("/data/input/sample_0004_R2.fastq.gz"));
    }

    #[test]
    fn container_command_template_keeps_output_paths_writable_when_out_dir_is_under_input_root() {
        let template = vec![
            "sh".to_string(),
            "-lc".to_string(),
            "printf '%s' /tmp/results/benchmark_corpus/fastq.report_qc/cluster/bench/report_qc/sample_0001/tools/multiqc/report_qc_report.json > /tmp/results/benchmark_corpus/fastq.report_qc/cluster/bench/report_qc/sample_0001/tools/multiqc/report_qc_report.json".to_string(),
        ];

        let rewritten = container_command_template(
            &template,
            Path::new("/tmp/results/benchmark_corpus"),
            Path::new(
                "/tmp/results/benchmark_corpus/fastq.report_qc/cluster/bench/report_qc/sample_0001/tools/multiqc",
            ),
            false,
        );

        assert_eq!(rewritten[0], "sh");
        assert!(rewritten[2].contains("/data/output/report_qc_report.json"));
        assert!(!rewritten[2].contains("/data/input/fastq.report_qc"));
    }

    #[test]
    fn container_input_mapping_preserves_single_file_basename() -> anyhow::Result<()> {
        let temp = tempdir()?;
        let input = temp.path().join("sample_0004_R1.fastq.gz");
        std::fs::write(&input, b"@read\nACGT\n+\n!!!!\n")?;

        let (mount_root, container_root) = container_input_mapping(&input);

        assert_eq!(mount_root, temp.path());
        assert_eq!(container_root, "/data/input/sample_0004_R1.fastq.gz");
        Ok(())
    }

    #[test]
    fn container_command_template_preserves_absolute_inputs_for_mixed_roots() {
        let template = vec![
            "bowtie2".to_string(),
            "-x".to_string(),
            "/opt/reference/contaminants/phix/bowtie2/reference".to_string(),
            "-S".to_string(),
            "/dev/null".to_string(),
            "-1".to_string(),
            "/data/benchmark_corpus/normalized/sample_0001_R1.fastq.gz".to_string(),
            "--met-file".to_string(),
            "/tmp/out/bowtie2.metrics.txt".to_string(),
        ];

        let rewritten =
            container_command_template(&template, Path::new("/"), Path::new("/tmp/out"), true);

        assert_eq!(rewritten[2], template[2]);
        assert_eq!(rewritten[4], "/dev/null");
        assert_eq!(rewritten[6], template[6]);
        assert_eq!(rewritten[8], "/data/output/bowtie2.metrics.txt");
    }

    #[test]
    fn hash_path_supports_directory_outputs() -> anyhow::Result<()> {
        let temp = tempdir()?;
        let root = temp.path().join("fastqc");
        std::fs::create_dir_all(root.join("nested"))?;
        std::fs::write(root.join("nested").join("summary.txt"), b"adapter-summary")?;

        let digest = hash_path(&root)?;

        assert_eq!(digest.len(), 64);
        Ok(())
    }

    #[test]
    fn hash_path_supports_directory_symlinks() -> anyhow::Result<()> {
        let temp = tempdir()?;
        let root = temp.path().join("taxonomy");
        std::fs::create_dir_all(root.join("kraken2"))?;
        std::fs::write(root.join("kraken2").join("hash.k2d"), b"kraken-hash")?;
        std::os::unix::fs::symlink(root.join("kraken2"), root.join("krakenuniq"))?;

        let digest = hash_path(&root)?;

        assert_eq!(digest.len(), 64);
        Ok(())
    }

    #[test]
    fn hash_inputs_ignores_missing_paths_and_hashes_directories() -> anyhow::Result<()> {
        let temp = tempdir()?;
        let root = temp.path().join("fastqc");
        std::fs::create_dir_all(&root)?;
        std::fs::write(root.join("summary.txt"), b"adapter-summary")?;

        let hashes = hash_inputs(&[root, temp.path().join("missing.txt")])?;

        assert_eq!(hashes.len(), 1);
        assert_eq!(hashes[0].len(), 64);
        Ok(())
    }
}
