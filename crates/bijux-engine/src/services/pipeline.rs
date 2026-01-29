use anyhow::Result;
use bijux_core::{StageId, ToolId};

#[derive(Debug, Clone)]
pub struct StagePlan {
    pub stage: StageId,
    pub tool: ToolId,
}

pub fn run_pipeline<T, RunStage, EmitEvent>(
    steps: &[StagePlan],
    mut emit_event: EmitEvent,
    mut run_stage: RunStage,
) -> Result<Vec<T>>
where
    RunStage: FnMut(&StagePlan) -> Result<T>,
    EmitEvent: FnMut(&str, &StagePlan) -> Result<()>,
{
    let mut entries = Vec::with_capacity(steps.len());
    for step in steps {
        emit_event("tool_selected", step)?;
        emit_event("stage_started", step)?;
        let entry = run_stage(step)?;
        entries.push(entry);
        emit_event("stage_finished", step)?;
    }
    Ok(entries)
}
