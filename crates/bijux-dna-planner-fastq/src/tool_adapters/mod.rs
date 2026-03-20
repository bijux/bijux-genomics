pub mod fastq;
pub mod stages;
pub mod template_render;

use bijux_dna_core::ids::{StageId, StepId, ToolId};

pub const MODULE: &str = "tool_adapters";

#[must_use]
pub fn default_stage_instance_id(stage_id: &StageId, tool_id: &ToolId) -> StepId {
    StepId::new(format!("{}.tool.{}", stage_id.as_str(), tool_id.as_str()))
}
