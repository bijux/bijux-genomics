use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_fastq::execution_support::NormalizationSupport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeInterpretationLevel {
    ObserverSpecialized,
    GenericEnvelope,
}

#[must_use]
pub fn runtime_interpretation_for_stage_tool(
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Option<RuntimeInterpretationLevel> {
    if !stage_id.as_str().starts_with(bijux_dna_core::id_catalog::FASTQ_PREFIX) {
        return None;
    }
    Some(if crate::observer::is_observer_specialized_stage_tool(stage_id, tool_id) {
        RuntimeInterpretationLevel::ObserverSpecialized
    } else {
        RuntimeInterpretationLevel::GenericEnvelope
    })
}

#[must_use]
pub fn runtime_interpretation_for_stage(stage_id: &StageId) -> Option<RuntimeInterpretationLevel> {
    if !stage_id.as_str().starts_with(bijux_dna_core::id_catalog::FASTQ_PREFIX) {
        return None;
    }
    if stage_uses_only_observer_specialized_runtime(stage_id) == Some(true) {
        return Some(RuntimeInterpretationLevel::ObserverSpecialized);
    }
    bijux_dna_domain_fastq::execution_support_for_stage(stage_id).map(|support| {
        if support.normalization_support == NormalizationSupport::ObserverSpecialized {
            RuntimeInterpretationLevel::ObserverSpecialized
        } else {
            RuntimeInterpretationLevel::GenericEnvelope
        }
    })
}

fn stage_uses_only_observer_specialized_runtime(stage_id: &StageId) -> Option<bool> {
    let runnable_tools = bijux_dna_domain_fastq::stage_tool_governance_profiles_for_stage(stage_id)
        .into_iter()
        .filter(|profile| profile.admitted_runtime_tool && profile.is_runnable())
        .map(|profile| profile.tool_id)
        .collect::<Vec<_>>();
    if runnable_tools.is_empty() {
        return None;
    }
    Some(runnable_tools.into_iter().all(|tool_id| {
        runtime_interpretation_for_stage_tool(stage_id, &tool_id)
            == Some(RuntimeInterpretationLevel::ObserverSpecialized)
    }))
}

#[must_use]
pub fn runtime_interpretation_stage_ids(
    level: RuntimeInterpretationLevel,
) -> Vec<bijux_dna_core::ids::StageId> {
    crate::contract_stage_ids()
        .into_iter()
        .filter(|stage_id| runtime_interpretation_for_stage(stage_id) == Some(level))
        .collect()
}
