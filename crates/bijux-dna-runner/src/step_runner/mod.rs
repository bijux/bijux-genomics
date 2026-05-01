use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_core::prelude::cache::CacheKey;
use bijux_dna_core::prelude::hashing::{input_fingerprint, parameters_fingerprint};
use bijux_dna_environment::api::RuntimeKind;

mod apptainer_args;
mod apptainer_execution;
mod artifacts;
mod command_template;
mod contracts;
mod docker_execution;
mod effects;
mod execution_dispatch;
mod execution_outcome;
mod identity;
mod inputs;
#[cfg(test)]
mod internal_contracts;
mod observer;
mod records;
mod runtime_policy;

#[allow(unused_imports)]
use apptainer_args::build_apptainer_exec_args;
#[allow(unused_imports)]
use command_template::container_command_template;
pub use contracts::StageResultV1;
use effects::{runner_failure, RunnerEffectKind};
use execution_dispatch::execute_step_outcome;
use identity::hash_inputs;
#[allow(unused_imports)]
use inputs::{common_parent, input_bind_roots, preserve_absolute_input_paths};
pub use observer::execute_observer_command;
use records::materialize_execution_records;

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
    let inputs: Vec<PathBuf> = step.io.inputs.iter().map(|input| input.path.clone()).collect();
    let input_root = common_parent(&inputs).unwrap_or_else(|| out_dir.clone());
    let outcome = execute_step_outcome(step, runner, &inputs, &input_root, out_dir, timeout)?;

    let outputs: Vec<PathBuf> = step.io.outputs.iter().map(|output| output.path.clone()).collect();
    let input_hashes = hash_inputs(&inputs)?;
    let output_hashes = hash_inputs(&outputs)?;
    let params_fingerprint =
        parameters_fingerprint(&serde_json::json!({ "command": step.command.template }))?;
    let input_fingerprint = input_fingerprint(&input_hashes);
    let env_digest = step.image.digest.clone().unwrap_or_else(|| step.image.image.clone());
    let _cache_key = CacheKey::new(
        input_fingerprint,
        params_fingerprint.clone(),
        step.image.image.clone(),
        env_digest,
    );
    let run_id = materialize_execution_records(
        step,
        &input_hashes,
        &output_hashes,
        runner,
        &outcome.command_output.command,
        &outcome.stdout,
        &outcome.stderr,
        &params_fingerprint,
    )?;

    Ok(StageResultV1 {
        run_id,
        exit_code: outcome.exit_code,
        runtime_s: outcome.runtime_s,
        memory_mb: outcome.memory_mb,
        outputs,
        metrics_path: None,
        stdout: outcome.stdout,
        stderr: outcome.stderr,
        command: outcome.command_output.command,
    })
}
