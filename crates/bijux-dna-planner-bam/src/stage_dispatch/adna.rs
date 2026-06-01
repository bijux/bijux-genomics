use anyhow::{anyhow, Result};
use bijux_dna_domain_bam::params::BamEffectiveParams;
use bijux_dna_domain_bam::BamStage;
use bijux_dna_stage_contract::StagePlanV1;

use crate::api::StagePlanRequest;
use crate::params;
use crate::tool_adapters;

/// # Errors
/// Returns an error if the selected ancient-DNA stage cannot be planned.
pub fn plan(stage: BamStage, request: &StagePlanRequest<'_>) -> Result<StagePlanV1> {
    match stage {
        BamStage::Damage => {
            let bam = request.bam.ok_or_else(|| anyhow!("damage requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let BamEffectiveParams::Damage(params) = params else {
                return Err(anyhow!("damage params mismatch"));
            };
            tool_adapters::stages_adna::damage::plan(request.tool, bam, request.out_dir, &params)
        }
        BamStage::Authenticity => {
            let bam = request.bam.ok_or_else(|| anyhow!("authenticity requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let BamEffectiveParams::Authenticity(params) = params else {
                return Err(anyhow!("authenticity params mismatch"));
            };
            tool_adapters::stages_adna::authenticity::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )
        }
        BamStage::Contamination => {
            let bam = request.bam.ok_or_else(|| anyhow!("contamination requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let BamEffectiveParams::Contamination(params) = params else {
                return Err(anyhow!("contamination params mismatch"));
            };
            tool_adapters::stages_adna::contamination::plan(
                request.tool,
                bam,
                request.bam_index,
                request.reference,
                request.out_dir,
                &params,
            )
        }
        BamStage::Sex => {
            let bam = request.bam.ok_or_else(|| anyhow!("sex requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let BamEffectiveParams::Sex(params) = params else {
                return Err(anyhow!("sex params mismatch"));
            };
            tool_adapters::stages_adna::sex::plan(request.tool, bam, request.out_dir, &params)
        }
        _ => Err(anyhow!("stage {} is not handled by the ancient-DNA dispatcher", stage.as_str())),
    }
}
