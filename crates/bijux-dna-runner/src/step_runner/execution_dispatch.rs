use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_environment::api::RuntimeKind;

use super::apptainer_execution::execute_apptainer_step;
use super::docker_execution::execute_docker_step;
use super::execution_outcome::StepExecutionOutcome;
use crate::runner_driver::run_local_command;

pub(super) fn execute_step_outcome(
    step: &ExecutionStep,
    runner: RuntimeKind,
    inputs: &[PathBuf],
    input_root: &Path,
    out_dir: &Path,
    timeout: Option<Duration>,
) -> Result<StepExecutionOutcome> {
    match runner {
        RuntimeKind::Local => execute_local_step(step, out_dir, timeout),
        RuntimeKind::Docker => execute_docker_step(step, inputs, input_root, out_dir, timeout),
        RuntimeKind::Apptainer | RuntimeKind::Singularity => {
            execute_apptainer_step(step, runner, inputs, input_root, out_dir)
        }
    }
}

fn execute_local_step(
    step: &ExecutionStep,
    out_dir: &Path,
    timeout: Option<Duration>,
) -> Result<StepExecutionOutcome> {
    let output = run_local_command(&step.command.template, out_dir, timeout)?;
    Ok(StepExecutionOutcome {
        exit_code: output.exit_code,
        runtime_s: output.runtime_s,
        memory_mb: 0.0,
        stdout: output.stdout.clone(),
        stderr: output.stderr.clone(),
        command_output: output,
    })
}
