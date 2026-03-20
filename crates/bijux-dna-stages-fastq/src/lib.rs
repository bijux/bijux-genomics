//! Stage specs, metrics, and observers for FASTQ.

use bijux_dna_core::ids::{StageId, ToolId};

pub mod metrics;
pub mod observer;
mod plugin;
pub mod stage_specs;

pub use bijux_dna_stage_contract::StagePlanJsonV1 as StagePlanJson;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeInterpretationLevel {
    ObserverSpecialized,
    GenericEnvelope,
}

#[must_use]
pub fn contract_stage_ids() -> Vec<bijux_dna_core::ids::StageId> {
    bijux_dna_domain_fastq::STAGES.to_vec()
}

#[must_use]
pub fn implemented_stages() -> Vec<bijux_dna_core::ids::StageId> {
    observer_stage_ids()
}

#[must_use]
pub fn closed_execution_stage_ids() -> Vec<bijux_dna_core::ids::StageId> {
    bijux_dna_domain_fastq::execution_closed_stage_ids()
}

#[must_use]
pub fn observer_stage_ids() -> Vec<bijux_dna_core::ids::StageId> {
    vec![
        bijux_dna_domain_fastq::STAGE_VALIDATE_READS,
        bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_READ_LENGTHS,
        bijux_dna_domain_fastq::STAGE_DETECT_ADAPTERS,
        bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_OVERREPRESENTED_SEQUENCES,
        bijux_dna_domain_fastq::STAGE_PROFILE_READS,
        bijux_dna_domain_fastq::STAGE_REPORT_QC,
    ]
}

#[must_use]
pub fn observer_stage_tool_bindings() -> Vec<(StageId, ToolId)> {
    vec![
        (
            bijux_dna_domain_fastq::STAGE_VALIDATE_READS,
            ToolId::from_static("fastqvalidator"),
        ),
        (
            bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_READ_LENGTHS,
            ToolId::from_static("seqkit_stats"),
        ),
        (
            bijux_dna_domain_fastq::STAGE_DETECT_ADAPTERS,
            ToolId::from_static("fastqc"),
        ),
        (
            bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_OVERREPRESENTED_SEQUENCES,
            ToolId::from_static("fastqc"),
        ),
        (
            bijux_dna_domain_fastq::STAGE_PROFILE_READS,
            ToolId::from_static("seqkit_stats"),
        ),
        (
            bijux_dna_domain_fastq::STAGE_REPORT_QC,
            ToolId::from_static("multiqc"),
        ),
    ]
}

#[must_use]
pub fn runtime_interpretation_for_stage_tool(
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Option<RuntimeInterpretationLevel> {
    if !stage_id.as_str().starts_with(bijux_dna_core::id_catalog::FASTQ_PREFIX) {
        return None;
    }
    Some(
        if observer_stage_tool_bindings()
            .into_iter()
            .any(|(candidate_stage, candidate_tool)| {
                candidate_stage == *stage_id && candidate_tool == *tool_id
            }) {
            RuntimeInterpretationLevel::ObserverSpecialized
        } else {
            RuntimeInterpretationLevel::GenericEnvelope
        },
    )
}

#[must_use]
pub fn runtime_interpretation_for_stage(stage_id: &StageId) -> Option<RuntimeInterpretationLevel> {
    if !stage_id.as_str().starts_with(bijux_dna_core::id_catalog::FASTQ_PREFIX) {
        return None;
    }
    Some(if observer_stage_ids().into_iter().any(|candidate| candidate == *stage_id) {
        RuntimeInterpretationLevel::ObserverSpecialized
    } else {
        RuntimeInterpretationLevel::GenericEnvelope
    })
}

#[must_use]
pub fn runtime_interpretation_stage_ids(
    level: RuntimeInterpretationLevel,
) -> Vec<bijux_dna_core::ids::StageId> {
    contract_stage_ids()
        .into_iter()
        .filter(|stage_id| runtime_interpretation_for_stage(stage_id) == Some(level))
        .collect()
}

pub mod contracts {
    pub use bijux_dna_domain_fastq::contract_for_stage;
    pub use bijux_dna_domain_fastq::FastqStageContract as StageContract;
}

#[cfg(test)]
mod tests {
    use super::{
        runtime_interpretation_for_stage_tool, RuntimeInterpretationLevel,
    };
    use bijux_dna_core::ids::{StageId, ToolId};

    #[test]
    fn runtime_interpretation_is_bound_to_stage_tool_pairs() {
        assert_eq!(
            runtime_interpretation_for_stage_tool(
                &StageId::from_static("fastq.profile_overrepresented_sequences"),
                &ToolId::from_static("fastqc"),
            ),
            Some(RuntimeInterpretationLevel::ObserverSpecialized)
        );
        assert_eq!(
            runtime_interpretation_for_stage_tool(
                &StageId::from_static("fastq.profile_overrepresented_sequences"),
                &ToolId::from_static("seqkit"),
            ),
            Some(RuntimeInterpretationLevel::GenericEnvelope)
        );
    }
}
