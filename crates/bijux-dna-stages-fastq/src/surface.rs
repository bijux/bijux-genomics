use bijux_dna_core::ids::{StageId, ToolId};

use crate::runtime::{runtime_interpretation_for_stage, RuntimeInterpretationLevel};

#[must_use]
pub fn contract_stage_ids() -> Vec<StageId> {
    bijux_dna_domain_fastq::STAGES.to_vec()
}

#[must_use]
pub fn implemented_stages() -> Vec<StageId> {
    closed_execution_stage_ids()
}

#[must_use]
pub fn closed_execution_stage_ids() -> Vec<StageId> {
    bijux_dna_domain_fastq::execution_closed_stage_ids()
}

#[must_use]
pub fn observer_specialized_stage_ids() -> Vec<StageId> {
    closed_execution_stage_ids()
        .into_iter()
        .filter(|stage_id| {
            runtime_interpretation_for_stage(stage_id)
                == Some(RuntimeInterpretationLevel::ObserverSpecialized)
        })
        .collect()
}

#[must_use]
pub fn observer_stage_ids() -> Vec<StageId> {
    observer_specialized_stage_ids()
}

#[must_use]
pub fn observer_stage_tool_bindings() -> Vec<(StageId, ToolId)> {
    crate::observer::observer_specialized_stage_tool_bindings()
}

#[cfg(test)]
mod tests {
    use super::{
        implemented_stages, observer_specialized_stage_ids, runtime_interpretation_for_stage,
        RuntimeInterpretationLevel,
    };
    use crate::runtime::runtime_interpretation_for_stage_tool;
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
                &StageId::from_static("fastq.trim_polyg_tails"),
                &ToolId::from_static("fastp"),
            ),
            Some(RuntimeInterpretationLevel::ObserverSpecialized)
        );
        assert_eq!(
            runtime_interpretation_for_stage_tool(
                &StageId::from_static("fastq.correct_errors"),
                &ToolId::from_static("rcorrector"),
            ),
            Some(RuntimeInterpretationLevel::ObserverSpecialized)
        );
        assert_eq!(
            runtime_interpretation_for_stage_tool(
                &StageId::from_static("fastq.infer_asvs"),
                &ToolId::from_static("dada2"),
            ),
            Some(RuntimeInterpretationLevel::ObserverSpecialized)
        );
        assert_eq!(
            runtime_interpretation_for_stage_tool(
                &StageId::from_static("fastq.profile_overrepresented_sequences"),
                &ToolId::from_static("seqkit"),
            ),
            Some(RuntimeInterpretationLevel::ObserverSpecialized)
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
        assert_eq!(
            runtime_interpretation_for_stage(&StageId::from_static("fastq.trim_polyg_tails")),
            Some(RuntimeInterpretationLevel::ObserverSpecialized)
        );
        assert_eq!(
            runtime_interpretation_for_stage(&StageId::from_static("fastq.trim_reads")),
            Some(RuntimeInterpretationLevel::ObserverSpecialized)
        );
        assert_eq!(
            runtime_interpretation_for_stage(&StageId::from_static("fastq.screen_taxonomy")),
            Some(RuntimeInterpretationLevel::ObserverSpecialized)
        );
        assert_eq!(
            runtime_interpretation_for_stage(&StageId::from_static("fastq.correct_errors")),
            Some(RuntimeInterpretationLevel::ObserverSpecialized)
        );
        assert_eq!(
            runtime_interpretation_for_stage(&StageId::from_static("fastq.infer_asvs")),
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
        assert!(observer_specialized.contains(&StageId::from_static("fastq.validate_reads")));
        assert!(observer_specialized.contains(&StageId::from_static("fastq.remove_duplicates")));
        assert!(observer_specialized.contains(&StageId::from_static("fastq.trim_terminal_damage")));
        assert!(observer_specialized.contains(&StageId::from_static("fastq.trim_polyg_tails")));
        assert!(observer_specialized.contains(&StageId::from_static("fastq.trim_reads")));
        assert!(observer_specialized.contains(&StageId::from_static("fastq.screen_taxonomy")));
        assert!(observer_specialized.contains(&StageId::from_static("fastq.correct_errors")));
        assert!(observer_specialized.contains(&StageId::from_static(
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
                runtime_interpretation_for_stage_tool(
                    &validate_stage,
                    &ToolId::from_static(tool_id)
                ),
                Some(RuntimeInterpretationLevel::ObserverSpecialized)
            );
        }
    }
}
