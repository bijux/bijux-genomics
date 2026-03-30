use bijux_dna_core::{
    id_catalog,
    ids::{StageId, ToolId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserverSpecializationContract {
    pub stage_id: &'static str,
    pub tool_id: &'static str,
    pub semantic_surface: &'static str,
}

const OBSERVER_SPECIALIZATION_CONTRACTS: &[ObserverSpecializationContract] = &[
    ObserverSpecializationContract {
        stage_id: "fastq.index_reference",
        tool_id: "bowtie2_build",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.index_reference",
        tool_id: "star",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.filter_reads",
        tool_id: id_catalog::TOOL_FASTP,
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.filter_reads",
        tool_id: "seqkit",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.filter_reads",
        tool_id: "prinseq",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.filter_reads",
        tool_id: "bbduk",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.filter_low_complexity",
        tool_id: "prinseq",
        semantic_surface: "filter_report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.filter_low_complexity",
        tool_id: "bbduk",
        semantic_surface: "filter_report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.validate_reads",
        tool_id: "fastqvalidator",
        semantic_surface: "validation_report",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.validate_reads",
        tool_id: "fastqc",
        semantic_surface: "validation_report",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.validate_reads",
        tool_id: "fastq_scan",
        semantic_surface: "validation_report",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.validate_reads",
        tool_id: "seqtk",
        semantic_surface: "validation_report",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.validate_reads",
        tool_id: "fqtools",
        semantic_surface: "validation_report",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.profile_read_lengths",
        tool_id: "seqkit_stats",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.detect_adapters",
        tool_id: "fastqc",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.deplete_rrna",
        tool_id: "sortmerna",
        semantic_surface: "rrna_report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.deplete_host",
        tool_id: "bowtie2",
        semantic_surface: "host_depletion_report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.deplete_reference_contaminants",
        tool_id: "bowtie2",
        semantic_surface: "contaminant_screen_report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.profile_overrepresented_sequences",
        tool_id: "fastqc",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.profile_overrepresented_sequences",
        tool_id: "fastq_scan",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.profile_overrepresented_sequences",
        tool_id: "seqkit",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.profile_reads",
        tool_id: "seqkit_stats",
        semantic_surface: "qc_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.extract_umis",
        tool_id: "umi_tools",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.normalize_primers",
        tool_id: id_catalog::TOOL_CUTADAPT,
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.normalize_abundance",
        tool_id: "seqkit",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.infer_asvs",
        tool_id: "dada2",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.cluster_otus",
        tool_id: "vsearch",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.merge_pairs",
        tool_id: "adapterremoval",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.merge_pairs",
        tool_id: "pear",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.merge_pairs",
        tool_id: "vsearch",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.merge_pairs",
        tool_id: "bbmerge",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.merge_pairs",
        tool_id: "flash2",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.merge_pairs",
        tool_id: "leehom",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.report_qc",
        tool_id: "multiqc",
        semantic_surface: "multiqc_data",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.screen_taxonomy",
        tool_id: id_catalog::TOOL_KRAKEN2,
        semantic_surface: "classification_report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.screen_taxonomy",
        tool_id: "krakenuniq",
        semantic_surface: "classification_report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.screen_taxonomy",
        tool_id: "centrifuge",
        semantic_surface: "classification_report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.screen_taxonomy",
        tool_id: "kaiju",
        semantic_surface: "classification_report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_reads",
        tool_id: id_catalog::TOOL_FASTP,
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_reads",
        tool_id: id_catalog::TOOL_CUTADAPT,
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_reads",
        tool_id: "atropos",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_reads",
        tool_id: "bbduk",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_reads",
        tool_id: "adapterremoval",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_reads",
        tool_id: "alientrimmer",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_reads",
        tool_id: "trimmomatic",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_reads",
        tool_id: "trim_galore",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_reads",
        tool_id: "prinseq",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_reads",
        tool_id: "seqkit",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_reads",
        tool_id: "fastx_clipper",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_reads",
        tool_id: "leehom",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_reads",
        tool_id: "skewer",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_reads",
        tool_id: "seqpurge",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.remove_duplicates",
        tool_id: "fastuniq",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.remove_duplicates",
        tool_id: "clumpify",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.remove_chimeras",
        tool_id: "vsearch",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_terminal_damage",
        tool_id: "adapterremoval",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_terminal_damage",
        tool_id: id_catalog::TOOL_CUTADAPT,
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_terminal_damage",
        tool_id: "seqkit",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_polyg_tails",
        tool_id: id_catalog::TOOL_FASTP,
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.trim_polyg_tails",
        tool_id: "bbduk",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.correct_errors",
        tool_id: "rcorrector",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.correct_errors",
        tool_id: "musket",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.correct_errors",
        tool_id: "lighter",
        semantic_surface: "report_json",
    },
    ObserverSpecializationContract {
        stage_id: "fastq.correct_errors",
        tool_id: "bayeshammer",
        semantic_surface: "report_json",
    },
];

#[must_use]
pub fn observer_specialization_contracts() -> &'static [ObserverSpecializationContract] {
    OBSERVER_SPECIALIZATION_CONTRACTS
}

#[must_use]
pub fn observer_specialization_contract_for_stage_tool(
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Option<ObserverSpecializationContract> {
    OBSERVER_SPECIALIZATION_CONTRACTS
        .iter()
        .copied()
        .find(|binding| {
            binding.stage_id == stage_id.as_str() && binding.tool_id == tool_id.as_str()
        })
}

#[must_use]
pub fn observer_specialized_stage_tool_bindings() -> Vec<(StageId, ToolId)> {
    OBSERVER_SPECIALIZATION_CONTRACTS
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
