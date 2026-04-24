use std::fs;

use anyhow::Result;
use bijux_dna_core::contract::{ArtifactRole, ExecutionStep};

use super::contract_error;

pub(super) fn verify_outputs(step: &ExecutionStep) -> Result<()> {
    for output in &step.io.outputs {
        if output.optional && !output.path.exists() {
            continue;
        }
        if !output.path.exists() {
            return Err(contract_error(
                step,
                output.name.as_str(),
                &output.path.display().to_string(),
                "missing output",
            ));
        }
        let metadata = fs::metadata(&output.path).map_err(|err| {
            contract_error(
                step,
                output.name.as_str(),
                &output.path.display().to_string(),
                &format!("unable to stat output: {err}"),
            )
        })?;
        if metadata.len() == 0 {
            return Err(contract_error(
                step,
                output.name.as_str(),
                &output.path.display().to_string(),
                "output is empty",
            ));
        }
        tracing::info!(
            target: "exec.contract",
            stage_id = %step.step_id.0,
            path = %output.path.display(),
            "artifact verified"
        );
        if matches!(output.role, ArtifactRole::MetricsJson | ArtifactRole::MetricsEnvelope) {
            let raw = fs::read_to_string(&output.path)?;
            serde_json::from_str::<serde_json::Value>(&raw).map_err(|err| {
                contract_error(
                    step,
                    output.name.as_str(),
                    &output.path.display().to_string(),
                    &format!("metrics output not parseable: {err}"),
                )
            })?;
        }
    }
    Ok(())
}
