use std::fs;

use anyhow::Result;
use bijux_dna_core::contract::{ArtifactRole, ExecutionStep};

use crate::{EngineEvent, EngineHooks};

use super::contract_error;

pub(super) fn verify_outputs(step: &ExecutionStep, hooks: Option<&dyn EngineHooks>) -> Result<()> {
    for expected in &step.expected_artifact_ids {
        if !step.io.outputs.iter().any(|output| output.name == *expected) {
            return Err(contract_error(
                step,
                expected.as_str(),
                "",
                "expected artifact is not declared as an output",
            ));
        }
    }
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
        if let Some(hooks) = hooks {
            hooks.on_event(EngineEvent::ArtifactVerified {
                step_id: step.step_id.clone(),
                path: output.path.display().to_string(),
            });
        }
        if is_json_role(output.role) {
            let raw = fs::read_to_string(&output.path)?;
            serde_json::from_str::<serde_json::Value>(&raw).map_err(|err| {
                contract_error(
                    step,
                    output.name.as_str(),
                    &output.path.display().to_string(),
                    &format!("json output not parseable: {err}"),
                )
            })?;
        }
    }
    Ok(())
}

fn is_json_role(role: ArtifactRole) -> bool {
    matches!(
        role,
        ArtifactRole::MetricsJson
            | ArtifactRole::MetricsEnvelope
            | ArtifactRole::ReportJson
            | ArtifactRole::StageReport
            | ArtifactRole::SummaryJson
    )
}
