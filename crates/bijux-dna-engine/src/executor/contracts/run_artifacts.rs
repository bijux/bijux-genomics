use std::fs;

use anyhow::Result;
use bijux_dna_core::contract::ExecutionStep;

use super::contract_error;

pub(super) fn verify_required_run_artifacts(step: &ExecutionStep) -> Result<()> {
    let run_artifacts_dir = step.out_dir.join("run_artifacts");
    let required = [
        ("metrics.json", run_artifacts_dir.join("metrics.json")),
        (
            "effective_config.json",
            run_artifacts_dir.join("effective_config.json"),
        ),
        ("stage_report.json", run_artifacts_dir.join("stage_report.json")),
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
            return Err(contract_error(
                step,
                label,
                &path.display().to_string(),
                "missing run artifact",
            ));
        }
        let metadata = fs::metadata(&path).map_err(|err| {
            contract_error(
                step,
                label,
                &path.display().to_string(),
                &format!("unable to stat run artifact: {err}"),
            )
        })?;
        if metadata.len() == 0 {
            return Err(contract_error(
                step,
                label,
                &path.display().to_string(),
                "artifact empty",
            ));
        }
    }
    Ok(())
}
