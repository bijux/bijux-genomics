use anyhow::{anyhow, Result};
use bijux_dna_domain_bam::{contract_for_stage, BamStage, StageSpec};

#[must_use]
pub fn stage_registry() -> Result<Vec<StageSpec>> {
    BamStage::all()
        .iter()
        .map(|stage| {
            let contract = contract_for_stage(stage.as_str())
                .ok_or_else(|| anyhow!("missing BAM stage contract for {}", stage.as_str()))?;
            Ok(StageSpec {
                id: stage.id(),
                stage: *stage,
                contract,
            })
        })
        .collect()
}
