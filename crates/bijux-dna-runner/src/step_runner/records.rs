use anyhow::Result;
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_core::prelude::hashing::run_id_from_hashes;
use bijux_dna_environment::api::RuntimeKind;

use super::artifacts::write_minimum_run_artifacts;
use super::identity::{execution_pipeline_identity, execution_sample_identity};

pub(super) fn materialize_execution_records(
    step: &ExecutionStep,
    input_hashes: &[String],
    output_hashes: &[String],
    runner: RuntimeKind,
    command: &str,
    stdout: &str,
    stderr: &str,
    params_fingerprint: &str,
) -> Result<String> {
    let pipeline_id = execution_pipeline_identity(step);
    let sample_id = execution_sample_identity(step);
    let run_id =
        run_id_from_hashes(&pipeline_id, &sample_id, params_fingerprint, input_hashes, None);
    write_minimum_run_artifacts(
        step,
        input_hashes,
        output_hashes,
        runner,
        command,
        stdout,
        stderr,
        &run_id,
        params_fingerprint,
    )?;
    Ok(run_id)
}
