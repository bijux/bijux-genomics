use std::fs;

use anyhow::Result;
use bijux_dna_core::contract::{ArtifactRole, ExecutionStep};

pub(crate) fn enforce_contract(step: &ExecutionStep) -> Result<()> {
    ContractEnforcer::new(step).enforce()
}

struct ContractEnforcer<'a> {
    step: &'a ExecutionStep,
}

impl<'a> ContractEnforcer<'a> {
    fn new(step: &'a ExecutionStep) -> Self {
        Self { step }
    }

    fn enforce(&self) -> Result<()> {
        self.verify_outputs()?;
        self.verify_metrics_envelope()?;
        self.verify_required_run_artifacts()?;
        Ok(())
    }

    fn contract_error(&self, artifact_id: &str, path: &str, message: &str) -> anyhow::Error {
        crate::errors::EngineError::Contract {
            step_id: self.step.step_id.as_str().to_string(),
            artifact_id: artifact_id.to_string(),
            path: path.to_string(),
            message: message.to_string(),
        }
        .into()
    }

    fn verify_outputs(&self) -> Result<()> {
        for output in &self.step.io.outputs {
            if output.optional && !output.path.exists() {
                continue;
            }
            if !output.path.exists() {
                return Err(self.contract_error(
                    output.name.as_str(),
                    &output.path.display().to_string(),
                    "missing output",
                ));
            }
            let metadata = fs::metadata(&output.path).map_err(|err| {
                self.contract_error(
                    output.name.as_str(),
                    &output.path.display().to_string(),
                    &format!("unable to stat output: {err}"),
                )
            })?;
            if metadata.len() == 0 {
                return Err(self.contract_error(
                    output.name.as_str(),
                    &output.path.display().to_string(),
                    "output is empty",
                ));
            }
            tracing::info!(
                target: "exec.contract",
                stage_id = %self.step.step_id.0,
                path = %output.path.display(),
                "artifact verified"
            );
            if matches!(output.role, ArtifactRole::MetricsJson | ArtifactRole::MetricsEnvelope) {
                let raw = fs::read_to_string(&output.path)?;
                serde_json::from_str::<serde_json::Value>(&raw).map_err(|err| {
                    self.contract_error(
                        output.name.as_str(),
                        &output.path.display().to_string(),
                        &format!("metrics output not parseable: {err}"),
                    )
                })?;
            }
        }
        Ok(())
    }

    fn verify_metrics_envelope(&self) -> Result<()> {
        if self.step.metrics_schema_ids.is_empty() {
            return Ok(());
        }
        let metrics_path = self
            .step
            .out_dir
            .join("run_artifacts")
            .join("metrics_envelope.json");
        if !metrics_path.exists() {
            return Err(self.contract_error(
                "metrics_envelope",
                &metrics_path.display().to_string(),
                "missing metrics_envelope.json",
            ));
        }
        let raw = fs::read_to_string(&metrics_path)?;
        serde_json::from_str::<serde_json::Value>(&raw).map_err(|err| {
            self.contract_error(
                "metrics_envelope",
                &metrics_path.display().to_string(),
                &format!("metrics_envelope.json parse failed: {err}"),
            )
        })?;
        Ok(())
    }

    fn verify_required_run_artifacts(&self) -> Result<()> {
        let run_artifacts_dir = self.step.out_dir.join("run_artifacts");
        let required = [
            ("metrics.json", run_artifacts_dir.join("metrics.json")),
            (
                "effective_config.json",
                run_artifacts_dir.join("effective_config.json"),
            ),
            (
                "stage_report.json",
                run_artifacts_dir.join("stage_report.json"),
            ),
            (
                "tool_invocation.json",
                run_artifacts_dir.join("tool_invocation.json"),
            ),
            (
                "execution_record.json",
                run_artifacts_dir.join("execution_record.json"),
            ),
        ];
        for (label, path) in required {
            if !path.exists() {
                return Err(self.contract_error(
                    label,
                    &path.display().to_string(),
                    "missing run artifact",
                ));
            }
            let metadata = fs::metadata(&path).map_err(|err| {
                self.contract_error(
                    label,
                    &path.display().to_string(),
                    &format!("unable to stat run artifact: {err}"),
                )
            })?;
            if metadata.len() == 0 {
                return Err(self.contract_error(
                    label,
                    &path.display().to_string(),
                    "artifact empty",
                ));
            }
        }
        Ok(())
    }
}
