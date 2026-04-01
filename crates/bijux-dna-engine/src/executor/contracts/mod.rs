use anyhow::Result;
use bijux_dna_core::contract::ExecutionStep;

mod metrics;
mod outputs;
mod run_artifacts;

pub(super) fn enforce_contract(step: &ExecutionStep) -> Result<()> {
    outputs::verify_outputs(step)?;
    metrics::verify_metrics_envelope(step)?;
    run_artifacts::verify_required_run_artifacts(step)?;
    Ok(())
}

fn contract_error(
    step: &ExecutionStep,
    artifact_id: &str,
    path: &str,
    message: &str,
) -> anyhow::Error {
    crate::errors::EngineError::Contract {
        step_id: step.step_id.as_str().to_string(),
        artifact_id: artifact_id.to_string(),
        path: path.to_string(),
        message: message.to_string(),
    }
    .into()
}
