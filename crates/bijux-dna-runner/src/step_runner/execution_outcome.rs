use crate::command_runner::CommandOutputV1;

#[derive(Debug, Clone)]
pub(super) struct StepExecutionOutcome {
    pub command_output: CommandOutputV1,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub runtime_s: f64,
    pub memory_mb: f64,
}
