use std::sync::OnceLock;

use bijux_dna_core::ids::{StageId, ToolId};

mod amplicon;
mod core;
mod transform;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserverSpecializationContract {
    pub stage_id: &'static str,
    pub tool_id: &'static str,
    pub semantic_surface: &'static str,
}

const fn contract(
    stage_id: &'static str,
    tool_id: &'static str,
    semantic_surface: &'static str,
) -> ObserverSpecializationContract {
    ObserverSpecializationContract {
        stage_id,
        tool_id,
        semantic_surface,
    }
}

fn specialization_contracts() -> &'static Vec<ObserverSpecializationContract> {
    static CONTRACTS: OnceLock<Vec<ObserverSpecializationContract>> = OnceLock::new();
    CONTRACTS.get_or_init(|| {
        let mut all = Vec::new();
        for group in [core::CONTRACTS, transform::CONTRACTS, amplicon::CONTRACTS] {
            all.extend_from_slice(group);
        }
        all
    })
}

#[must_use]
pub fn observer_specialization_contracts() -> &'static [ObserverSpecializationContract] {
    specialization_contracts().as_slice()
}

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

#[cfg(test)]
mod tests {
    use super::{
        is_observer_specialized_stage_tool, observer_semantic_surface_for_stage_tool,
        observer_specialization_contracts, observer_specialized_stage_tool_bindings,
    };
    use bijux_dna_core::{
        id_catalog,
        ids::{StageId, ToolId},
    };

    #[test]
    fn observer_contracts_cover_current_specialized_fastq_tools() {
        let bindings = observer_specialized_stage_tool_bindings();
        assert!(bindings.contains(&(
            StageId::from_static("fastq.validate_reads"),
            ToolId::from_static("fastqvalidator")
        )));
        assert!(bindings.contains(&(
            StageId::from_static("fastq.report_qc"),
            ToolId::from_static(id_catalog::TOOL_MULTIQC)
        )));
        assert!(bindings.contains(&(
            StageId::from_static("fastq.screen_taxonomy"),
            ToolId::from_static(id_catalog::TOOL_KRAKEN2)
        )));
        assert!(bindings.contains(&(
            StageId::from_static("fastq.correct_errors"),
            ToolId::from_static("lighter")
        )));
        assert!(bindings.contains(&(
            StageId::from_static("fastq.normalize_primers"),
            ToolId::from_static(id_catalog::TOOL_CUTADAPT)
        )));
        assert!(bindings.contains(&(
            StageId::from_static("fastq.normalize_abundance"),
            ToolId::from_static(id_catalog::TOOL_SEQKIT)
        )));
        assert!(bindings.contains(&(
            StageId::from_static("fastq.remove_chimeras"),
            ToolId::from_static("vsearch")
        )));
        assert!(bindings.contains(&(
            StageId::from_static("fastq.trim_reads"),
            ToolId::from_static("alientrimmer")
        )));
        assert!(bindings.contains(&(
            StageId::from_static("fastq.trim_reads"),
            ToolId::from_static("fastx_clipper")
        )));
        assert!(bindings.contains(&(
            StageId::from_static("fastq.trim_reads"),
            ToolId::from_static("leehom")
        )));
        assert!(bindings.contains(&(
            StageId::from_static("fastq.trim_reads"),
            ToolId::from_static("skewer")
        )));
    }

    #[test]
    fn observer_contracts_publish_semantic_surfaces() {
        assert_eq!(
            observer_semantic_surface_for_stage_tool(
                &StageId::from_static("fastq.detect_adapters"),
                &ToolId::from_static(id_catalog::TOOL_FASTQC),
            ),
            Some("report_json")
        );
        assert_eq!(
            observer_semantic_surface_for_stage_tool(
                &StageId::from_static("fastq.trim_polyg_tails"),
                &ToolId::from_static("bbduk"),
            ),
            Some("report_json")
        );
        assert_eq!(
            observer_semantic_surface_for_stage_tool(
                &StageId::from_static("fastq.screen_taxonomy"),
                &ToolId::from_static("centrifuge"),
            ),
            Some("classification_report_json")
        );
        assert_eq!(
            observer_semantic_surface_for_stage_tool(
                &StageId::from_static("fastq.trim_reads"),
                &ToolId::from_static(id_catalog::TOOL_FASTP),
            ),
            Some("report_json")
        );
        assert_eq!(
            observer_semantic_surface_for_stage_tool(
                &StageId::from_static("fastq.trim_reads"),
                &ToolId::from_static("alientrimmer"),
            ),
            Some("report_json")
        );
        assert_eq!(
            observer_semantic_surface_for_stage_tool(
                &StageId::from_static("fastq.trim_reads"),
                &ToolId::from_static("fastx_clipper"),
            ),
            Some("report_json")
        );
        assert_eq!(
            observer_semantic_surface_for_stage_tool(
                &StageId::from_static("fastq.trim_reads"),
                &ToolId::from_static("leehom"),
            ),
            Some("report_json")
        );
        assert_eq!(
            observer_semantic_surface_for_stage_tool(
                &StageId::from_static("fastq.trim_reads"),
                &ToolId::from_static("skewer"),
            ),
            Some("report_json")
        );
        assert_eq!(
            observer_semantic_surface_for_stage_tool(
                &StageId::from_static("fastq.normalize_primers"),
                &ToolId::from_static(id_catalog::TOOL_CUTADAPT),
            ),
            Some("report_json")
        );
        assert_eq!(
            observer_semantic_surface_for_stage_tool(
                &StageId::from_static("fastq.normalize_abundance"),
                &ToolId::from_static(id_catalog::TOOL_SEQKIT),
            ),
            Some("report_json")
        );
        assert_eq!(
            observer_semantic_surface_for_stage_tool(
                &StageId::from_static("fastq.remove_chimeras"),
                &ToolId::from_static("vsearch"),
            ),
            Some("report_json")
        );
    }

    #[test]
    fn observer_contracts_remain_unique_per_stage_tool_pair() {
        let mut seen = std::collections::BTreeSet::new();
        for binding in observer_specialization_contracts() {
            assert!(seen.insert((binding.stage_id, binding.tool_id)));
            assert!(!binding.semantic_surface.is_empty());
            assert!(is_observer_specialized_stage_tool(
                &StageId::from_static(binding.stage_id),
                &ToolId::from_static(binding.tool_id),
            ));
        }
    }
}
