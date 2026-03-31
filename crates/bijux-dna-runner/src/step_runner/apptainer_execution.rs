use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_environment::api::RuntimeKind;

use crate::command_runner::run_command;

use super::apptainer_args::build_apptainer_exec_args;
use super::execution_outcome::StepExecutionOutcome;
use super::runtime_policy::configured_memory_mb;
use super::{runner_failure, RunnerEffectKind};

pub(super) fn execute_apptainer_step(
    step: &ExecutionStep,
    runner: RuntimeKind,
    inputs: &[PathBuf],
    input_root: &Path,
    out_dir: &Path,
) -> Result<StepExecutionOutcome> {
    let args = build_apptainer_exec_args(step, inputs, input_root, out_dir, runner)?;
    let bin = if runner == RuntimeKind::Apptainer {
        "apptainer"
    } else {
        "singularity"
    };
    let command_output = run_command(bin, &args)
        .map_err(|err| runner_failure(RunnerEffectKind::CommandSpawn, err.to_string()))?;
    let exit_code = command_output.exit_code;
    let stdout = command_output.stdout.clone();
    let stderr = command_output.stderr.clone();
    let runtime_s = command_output.runtime_s;
    let memory_mb = configured_memory_mb(step);

    Ok(StepExecutionOutcome {
        command_output,
        exit_code,
        stdout,
        stderr,
        runtime_s,
        memory_mb,
    })
}
