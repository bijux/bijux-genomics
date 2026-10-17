use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_containers::api::RuntimeKind;
use bijux_dna_core::contract::ExecutionStep;

use crate::command_runner::{
    run_command_with_environment, run_command_with_environment_and_timeout,
};

use super::apptainer_args::build_apptainer_exec_args;
use super::execution_outcome::StepExecutionOutcome;
use super::runtime_policy::{configured_memory_mb, StepRuntimeContext};
use super::{runner_failure, RunnerEffectKind};

pub(super) fn execute_apptainer_step(
    step: &ExecutionStep,
    runner: RuntimeKind,
    inputs: &[PathBuf],
    input_root: &Path,
    out_dir: &Path,
    context: StepRuntimeContext<'_>,
) -> Result<StepExecutionOutcome> {
    let args =
        build_apptainer_exec_args(step, inputs, input_root, out_dir, runner, context.environment)?;
    let default_bin = if runner == RuntimeKind::Apptainer { "apptainer" } else { "singularity" };
    let bin = context
        .executable
        .map_or_else(|| default_bin.to_string(), |path| path.display().to_string());
    let command_output = if let Some(timeout) = context.timeout {
        let timed = run_command_with_environment_and_timeout(
            &bin,
            &args,
            None,
            context.environment,
            timeout,
        )
        .map_err(|err| runner_failure(RunnerEffectKind::CommandSpawn, err.to_string()))?;
        // A leader can exit successfully while descendants retain captured pipes.
        // Cancellation must refuse stage publication regardless of the leader status.
        if timed.timed_out {
            return Err(runner_failure(RunnerEffectKind::ContainerLifecycle, timed.output.stderr));
        }
        timed.output
    } else {
        run_command_with_environment(&bin, &args, None, context.environment)
            .map_err(|err| runner_failure(RunnerEffectKind::CommandSpawn, err.to_string()))?
    };
    let exit_code = command_output.exit_code;
    let stdout = command_output.stdout.clone();
    let stderr = command_output.stderr.clone();
    let runtime_s = command_output.runtime_s;
    let memory_mb = configured_memory_mb(step, context.environment);

    Ok(StepExecutionOutcome { command_output, exit_code, stdout, stderr, runtime_s, memory_mb })
}
