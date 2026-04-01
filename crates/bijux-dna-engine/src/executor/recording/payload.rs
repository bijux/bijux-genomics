use bijux_dna_core::contract::ExecutionStep;

pub(super) fn execution_record_payload(
    step: &ExecutionStep,
    attempt: u32,
    started_at: &str,
    finished_at: &str,
    duration_s: f64,
    exit_code: i32,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "bijux.execution_record.v1",
        "step_id": step.step_id.to_string(),
        "stage_id": step.stage_id.to_string(),
        "attempt": attempt,
        "started_at": started_at,
        "finished_at": finished_at,
        "duration_s": duration_s,
        "exit_code": exit_code,
    })
}
