use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_core::contract::ExecutionStep;

use crate::backend::docker::executor::{
    docker_logs_with_context, docker_wait_timeout_with_context, docker_wait_with_context,
    parse_mem_to_mb,
};
use crate::command_runner::run_command_with_environment;

use super::command_template::container_command_template;
use super::execution_outcome::StepExecutionOutcome;
use super::inputs::{input_bind_roots, preserve_absolute_input_paths};
use super::runtime_policy::{
    environment_value, network_allowed_with_context, runtime_env_exports,
    stage_workdir_in_container, StepRuntimeContext,
};
use super::{runner_failure, RunnerEffectKind};

pub(super) fn execute_docker_step(
    step: &ExecutionStep,
    inputs: &[PathBuf],
    input_root: &Path,
    out_dir: &Path,
    context: StepRuntimeContext<'_>,
) -> Result<StepExecutionOutcome> {
    let preserve_absolute_inputs = preserve_absolute_input_paths(inputs);
    let bind_roots = input_bind_roots(inputs, input_root, preserve_absolute_inputs);
    let output_mount = format!("{}:/data/output:rw", out_dir.display());
    let command_template = container_command_template(
        &step.command.template,
        input_root,
        out_dir,
        preserve_absolute_inputs,
    );

    let container_name = format!("bijux-dna-stage-{}", uuid::Uuid::new_v4());
    let mut args: Vec<String> = vec![
        "run".to_string(),
        "-d".to_string(),
        "--rm=false".to_string(),
        "--name".to_string(),
        container_name.clone(),
    ];
    if environment_value(context.environment, "BIJUX_STAGE_WORKDIR").is_some() {
        args.push("-w".to_string());
        args.push(stage_workdir_in_container(
            out_dir,
            bijux_dna_containers::api::RuntimeKind::Docker,
            context.environment,
        ));
    }
    for (key, value) in runtime_env_exports(context.environment) {
        args.push("-e".to_string());
        args.push(format!("{key}={value}"));
    }
    if !network_allowed_with_context(context.environment) {
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
    args.extend(command_template);

    let command_output =
        run_command_with_environment("docker", &args, None, context.environment)
            .map_err(|err| runner_failure(RunnerEffectKind::CommandSpawn, err.to_string()))?;
    if command_output.exit_code != 0 {
        return Err(runner_failure(
            RunnerEffectKind::ContainerLifecycle,
            format!("docker run failed for {}", step.step_id.0),
        ));
    }
    let container_id = command_output.stdout.trim().to_string();
    if container_id.is_empty() {
        return Err(runner_failure(
            RunnerEffectKind::ContainerLifecycle,
            format!("missing container id for {}", step.step_id.0),
        ));
    }
    let exit_code = if let Some(timeout) = context.timeout {
        docker_wait_timeout_with_context(&container_id, timeout, context.environment)?
    } else {
        docker_wait_with_context(&container_id, context.environment)?
    };
    let stdout = docker_logs_with_context(&container_id, context.environment)?;
    let stderr = String::new();
    let runtime_s = command_output.runtime_s;
    let memory_mb = parse_mem_to_mb("0MiB / 0MiB").unwrap_or(0.0);

    Ok(StepExecutionOutcome { command_output, exit_code, stdout, stderr, runtime_s, memory_mb })
}
