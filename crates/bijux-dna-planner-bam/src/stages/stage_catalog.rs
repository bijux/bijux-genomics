use anyhow::{anyhow, Result};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::ids::StageId;
use bijux_dna_domain_bam::{contract_for_stage, BamStage, StageSpec};

#[must_use]
pub fn stage_registry(registry: &ToolRegistry) -> Result<Vec<StageSpec>> {
    let mut stage_ids = registry
        .stages()
        .keys()
        .filter(|stage_id| stage_id.as_str().starts_with("bam."))
        .cloned()
        .collect::<Vec<StageId>>();
    stage_ids.sort();
    stage_ids
        .iter()
        .map(|stage_id| {
            let stage = BamStage::try_from(stage_id.as_str())?;
            let contract = contract_for_stage(stage_id.as_str())
                .ok_or_else(|| anyhow!("missing BAM stage contract for {}", stage_id.as_str()))?;
            Ok(StageSpec {
                id: stage.id(),
                stage,
                contract,
            })
        })
        .collect()
}
