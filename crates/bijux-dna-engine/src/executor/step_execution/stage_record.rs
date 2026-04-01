use bijux_dna_core::contract::{ExecutionStep, StageExecutionRecordV1};

#[must_use]
pub(super) fn stage_execution_record(
    step: &ExecutionStep,
    attempt: u32,
    success: bool,
) -> StageExecutionRecordV1 {
    StageExecutionRecordV1 {
        stage_id: step.step_id.to_string(),
        attempt,
        success,
        cached: false,
    }
}
