use bijux_dna_core::ids::{StageId, ToolId};

use crate::FastqArtifactKind;

use super::layout_catalog::stage_tool_input_layout_contract;

#[must_use]
pub fn tool_supports_input_layout(stage_id: &StageId, tool_id: &ToolId, paired_end: bool) -> bool {
    let Some(contract) = crate::contract_for_stage(stage_id.as_str()) else {
        return false;
    };
    let required_kind =
        if paired_end { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    if !contract.accepted_input_kinds.iter().any(|kind| kind == &required_kind) {
        return false;
    }
    match stage_tool_input_layout_contract(stage_id, tool_id) {
        Some(layout_contract) => {
            if paired_end {
                layout_contract.supports_paired_end
            } else {
                layout_contract.supports_single_end
            }
        }
        None => true,
    }
}

#[must_use]
pub fn filter_tools_for_input_layout(
    stage_id: &StageId,
    tool_ids: Vec<ToolId>,
    paired_end: bool,
) -> Vec<ToolId> {
    tool_ids
        .into_iter()
        .filter(|tool_id| tool_supports_input_layout(stage_id, tool_id, paired_end))
        .collect()
}
