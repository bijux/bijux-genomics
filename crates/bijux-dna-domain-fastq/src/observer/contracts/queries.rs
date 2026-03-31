use bijux_dna_core::ids::{StageId, ToolId};

use super::{observer_specialization_contracts, ObserverSpecializationContract};

#[must_use]
pub fn observer_specialization_contract_for_stage_tool(
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Option<ObserverSpecializationContract> {
    observer_specialization_contracts()
        .iter()
        .copied()
        .find(|binding| {
            binding.stage_id == stage_id.as_str() && binding.tool_id == tool_id.as_str()
        })
}

#[must_use]
pub fn observer_specialized_stage_tool_bindings() -> Vec<(StageId, ToolId)> {
    observer_specialization_contracts()
        .iter()
        .map(|binding| {
            (
                StageId::from_static(binding.stage_id),
                ToolId::from_static(binding.tool_id),
            )
        })
        .collect()
}

#[must_use]
pub fn observer_semantic_surface_for_stage_tool(
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Option<&'static str> {
    observer_specialization_contract_for_stage_tool(stage_id, tool_id)
        .map(|binding| binding.semantic_surface)
}

#[must_use]
pub fn is_observer_specialized_stage_tool(stage_id: &StageId, tool_id: &ToolId) -> bool {
    observer_specialization_contract_for_stage_tool(stage_id, tool_id).is_some()
}
