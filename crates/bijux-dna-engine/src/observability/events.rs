use bijux_dna_core::ids::StepId;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum EngineEvent {
    StepStart { step_id: StepId, attempt: u32 },
    StepEnd { step_id: StepId, attempt: u32, success: bool },
    Retry { step_id: StepId, attempt: u32, exit_code: i32 },
    ArtifactVerified { step_id: StepId, path: String },
}
