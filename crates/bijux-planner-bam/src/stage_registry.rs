use bijux_domain_bam::{contract_for_stage, BamArtifactKind, BamStage, BamStageContract, StageSpec};

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
            }),
        })
        .collect()
}
