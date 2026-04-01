use std::fs;

use anyhow::Result;
use bijux_dna_core::contract::ExecutionStep;

use super::contract_error;

pub(super) fn verify_metrics_envelope(step: &ExecutionStep) -> Result<()> {
    if step.metrics_schema_ids.is_empty() {
        return Ok(());
    }
    let metrics_path = step
        .out_dir
        .join("run_artifacts")
        .join("metrics_envelope.json");
    if !metrics_path.exists() {
        return Err(contract_error(
            step,
            "metrics_envelope",
            &metrics_path.display().to_string(),
            "missing metrics_envelope.json",
        ));
    }
    let raw = fs::read_to_string(&metrics_path)?;
    serde_json::from_str::<serde_json::Value>(&raw).map_err(|err| {
        contract_error(
            step,
            "metrics_envelope",
            &metrics_path.display().to_string(),
            &format!("metrics_envelope.json parse failed: {err}"),
        )
    })?;
    Ok(())
}
