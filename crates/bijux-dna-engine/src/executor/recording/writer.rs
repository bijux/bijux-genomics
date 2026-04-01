use anyhow::Result;
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_infra::ensure_dir;

pub(super) fn write_execution_record(
    step: &ExecutionStep,
    payload: &serde_json::Value,
) -> Result<()> {
    let run_artifacts_dir = step.out_dir.join("run_artifacts");
    ensure_dir(&run_artifacts_dir)?;
    let path = run_artifacts_dir.join("execution_record.json");
    bijux_dna_runtime::recording::write_canonical_json(&path, payload)?;
    Ok(())
}
