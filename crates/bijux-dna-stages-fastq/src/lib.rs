//! Stage specs, metrics, and observers for FASTQ.

use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_fastq::execution_support::NormalizationSupport;

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
    closed_execution_stage_ids()
}

#[must_use]
pub fn closed_execution_stage_ids() -> Vec<bijux_dna_core::ids::StageId> {
    bijux_dna_domain_fastq::execution_closed_stage_ids()
}

#[must_use]
pub fn observer_specialized_stage_ids() -> Vec<bijux_dna_core::ids::StageId> {
    closed_execution_stage_ids()
        .into_iter()
        .filter(|stage_id| {
            stage_uses_only_observer_specialized_runtime(stage_id).unwrap_or_else(|| {
                bijux_dna_domain_fastq::execution_support_for_stage(stage_id).is_some_and(
                    |support| support.normalization_support == NormalizationSupport::ObserverSpecialized,
                )
            })
        })
        .collect()
}

#[must_use]
pub fn observer_stage_ids() -> Vec<bijux_dna_core::ids::StageId> {
    observer_specialized_stage_ids()
}

#[must_use]
pub fn observer_stage_tool_bindings() -> Vec<(StageId, ToolId)> {
    vec![
        (
            bijux_dna_domain_fastq::STAGE_VALIDATE_READS,
            ToolId::from_static("fastqvalidator"),
        ),
        (
            bijux_dna_domain_fastq::STAGE_VALIDATE_READS,
            ToolId::from_static("seqtk"),
        ),
        (
            bijux_dna_domain_fastq::STAGE_VALIDATE_READS,
            ToolId::from_static("fqtools"),
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
        (
            bijux_dna_domain_fastq::stages::ids::STAGE_REMOVE_DUPLICATES,
            ToolId::from_static("fastuniq"),
        ),
        (
            bijux_dna_domain_fastq::stages::ids::STAGE_REMOVE_DUPLICATES,
            ToolId::from_static("clumpify"),
        ),
        (
            bijux_dna_domain_fastq::stages::ids::STAGE_TRIM_TERMINAL_DAMAGE,
            ToolId::from_static("cutadapt"),
        ),
        (
            bijux_dna_domain_fastq::stages::ids::STAGE_TRIM_TERMINAL_DAMAGE,
            ToolId::from_static("seqkit"),
        ),
    ]
}

#[must_use]
pub fn runtime_interpretation_for_stage_tool(
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Option<RuntimeInterpretationLevel> {
    if !stage_id
        .as_str()
        .starts_with(bijux_dna_core::id_catalog::FASTQ_PREFIX)
    {
        return None;
    }
    Some(
        if observer_stage_tool_bindings()
            .into_iter()
            .any(|(candidate_stage, candidate_tool)| {
                candidate_stage == *stage_id && candidate_tool == *tool_id
            })
        {
            RuntimeInterpretationLevel::ObserverSpecialized
        } else {
            RuntimeInterpretationLevel::GenericEnvelope
        },
    )
}

#[must_use]
pub fn runtime_interpretation_for_stage(stage_id: &StageId) -> Option<RuntimeInterpretationLevel> {
    if !stage_id
        .as_str()
        .starts_with(bijux_dna_core::id_catalog::FASTQ_PREFIX)
    {
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
        implemented_stages, observer_specialized_stage_ids, runtime_interpretation_for_stage,
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
                &StageId::from_static("fastq.validate_reads"),
                &ToolId::from_static("seqtk"),
            ),
            Some(RuntimeInterpretationLevel::ObserverSpecialized)
        );
        assert_eq!(
            runtime_interpretation_for_stage_tool(
                &StageId::from_static("fastq.validate_reads"),
                &ToolId::from_static("fqtools"),
            ),
            Some(RuntimeInterpretationLevel::ObserverSpecialized)
        );
        assert_eq!(
            runtime_interpretation_for_stage_tool(
                &StageId::from_static("fastq.remove_duplicates"),
                &ToolId::from_static("clumpify"),
            ),
            Some(RuntimeInterpretationLevel::ObserverSpecialized)
        );
        assert_eq!(
            runtime_interpretation_for_stage_tool(
                &StageId::from_static("fastq.trim_terminal_damage"),
                &ToolId::from_static("cutadapt"),
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

    #[test]
    fn stage_level_runtime_interpretation_uses_domain_normalization_truth() {
        assert_eq!(
            runtime_interpretation_for_stage(&StageId::from_static("fastq.detect_adapters")),
            Some(RuntimeInterpretationLevel::ObserverSpecialized)
        );
        assert_eq!(
            runtime_interpretation_for_stage(&StageId::from_static("fastq.validate_reads")),
            Some(RuntimeInterpretationLevel::ObserverSpecialized)
        );
        assert_eq!(
            runtime_interpretation_for_stage(&StageId::from_static("fastq.remove_duplicates")),
            Some(RuntimeInterpretationLevel::ObserverSpecialized)
        );
        assert_eq!(
            runtime_interpretation_for_stage(&StageId::from_static("fastq.trim_terminal_damage")),
            Some(RuntimeInterpretationLevel::ObserverSpecialized)
        );
    }

    #[test]
    fn implemented_stages_cover_closed_execution_surface() {
        assert_eq!(
            implemented_stages(),
            bijux_dna_domain_fastq::execution_closed_stage_ids()
        );
    }

    #[test]
    fn observer_specialized_stage_ids_exclude_mixed_stage_families() {
        let observer_specialized = observer_specialized_stage_ids();
        assert!(observer_specialized.contains(&StageId::from_static("fastq.detect_adapters")));
        assert!(observer_specialized.contains(&StageId::from_static("fastq.report_qc")));
        assert!(observer_specialized.contains(&StageId::from_static(
            "fastq.remove_duplicates"
        )));
        assert!(observer_specialized.contains(&StageId::from_static(
            "fastq.trim_terminal_damage"
        )));
        assert!(!observer_specialized.contains(&StageId::from_static("fastq.validate_reads")));
        assert!(!observer_specialized.contains(&StageId::from_static(
            "fastq.profile_overrepresented_sequences"
        )));
    }

    #[test]
    fn stage_runtime_interpretation_tracks_fully_specialized_runtime_bindings() {
        let validate_stage = StageId::from_static("fastq.validate_reads");
        assert_eq!(
            runtime_interpretation_for_stage(&validate_stage),
            Some(RuntimeInterpretationLevel::ObserverSpecialized)
        );
        for tool_id in ["fastqvalidator", "seqtk", "fqtools"] {
            assert_eq!(
                runtime_interpretation_for_stage_tool(&validate_stage, &ToolId::from_static(tool_id)),
                Some(RuntimeInterpretationLevel::ObserverSpecialized)
            );
        }
    }
}
