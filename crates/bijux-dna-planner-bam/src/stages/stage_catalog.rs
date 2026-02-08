use bijux_dna_domain_bam::{
    contract_for_stage, BamArtifactKind, BamStage, BamStageContract, StageSpec,
};

#[must_use]
pub fn stage_registry() -> Vec<StageSpec> {
    BamStage::all()
        .iter()
        .map(|stage| StageSpec {
            id: stage.id(),
            stage: *stage,
            contract: contract_for_stage(stage.as_str()).unwrap_or(BamStageContract {
                input: BamArtifactKind::Bam,
                output: BamArtifactKind::Report,
                emits_bam: false,
                emits_report: true,
                sorting: "accepts_unsorted_bam",
                indexing: "index_optional",
                read_group_policy: "requires_read_groups_for_sample_metadata",
                duplicate_policy: "no_duplicate_marking",
                mapping_quality_policy: "no_mapping_quality_filter",
                deterministic: true,
                nondeterminism_reason: None,
            }),
        })
        .collect()
}
