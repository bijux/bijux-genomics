use anyhow::Result;
use bijux_dna_core::contract::ExecutionStep;

mod payload;
mod writer;

pub(super) fn record_execution(
    step: &ExecutionStep,
    attempt: u32,
    started_at: &str,
    finished_at: &str,
    duration_s: f64,
    exit_code: i32,
) -> Result<()> {
    let payload =
        payload::execution_record_payload(step, attempt, started_at, finished_at, duration_s, exit_code);
    writer::write_execution_record(step, &payload)
}
