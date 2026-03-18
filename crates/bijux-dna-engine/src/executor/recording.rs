use anyhow::Result;
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_infra::ensure_dir;

pub(crate) fn record_execution(
    step: &ExecutionStep,
    attempt: u32,
    started_at: &str,
    finished_at: &str,
    duration_s: f64,
    exit_code: i32,
) -> Result<()> {
    let run_artifacts_dir = step.out_dir.join("run_artifacts");
    ensure_dir(&run_artifacts_dir)?;
    let payload = serde_json::json!({
        "schema_version": "bijux.execution_record.v1",
        "step_id": step.step_id.to_string(),
        "stage_id": step.stage_id.to_string(),
        "attempt": attempt,
        "started_at": started_at,
        "finished_at": finished_at,
        "duration_s": duration_s,
        "exit_code": exit_code,
    });
    let path = run_artifacts_dir.join("execution_record.json");
    bijux_dna_runtime::recording::write_canonical_json(&path, &payload)?;
    Ok(())
}
