use anyhow::{anyhow, Result};
use bijux_dna_domain_bam::BamStage;
use bijux_dna_stage_contract::StagePlanV1;

use crate::api::StagePlanRequest;

/// # Errors
/// Returns an error if the selected downstream stage cannot be planned.
pub fn plan(stage: BamStage, _request: &StagePlanRequest<'_>) -> Result<StagePlanV1> {
    match stage {
        BamStage::BiasMitigation => {
            #[cfg(feature = "bam_downstream")]
            {
                use bijux_dna_domain_bam::params::BamEffectiveParams;

                let bam = _request.bam.ok_or_else(|| anyhow!("bias_mitigation requires bam"))?;
                let params = crate::params::effective_params_for_stage(stage, _request.params)?;
                let BamEffectiveParams::BiasMitigation(params) = params else {
                    return Err(anyhow!("bias_mitigation params mismatch"));
                };
                crate::tool_adapters::stages_downstream::bias_mitigation::plan(
                    _request.tool,
                    bam,
                    _request.out_dir,
                    &params,
                )
            }
            #[cfg(not(feature = "bam_downstream"))]
            {
                Err(anyhow!("bias_mitigation requires bam_downstream feature"))
            }
        }
        BamStage::Haplogroups => {
            #[cfg(feature = "bam_downstream")]
            {
                use bijux_dna_domain_bam::params::BamEffectiveParams;

                let bam = _request.bam.ok_or_else(|| anyhow!("haplogroups requires bam"))?;
                let params = crate::params::effective_params_for_stage(stage, _request.params)?;
                let BamEffectiveParams::Haplogroups(params) = params else {
                    return Err(anyhow!("haplogroups params mismatch"));
                };
                crate::tool_adapters::stages_downstream::haplogroups::plan(
                    _request.tool,
                    bam,
                    _request.bam_index,
                    _request.out_dir,
                    &params,
                )
            }
            #[cfg(not(feature = "bam_downstream"))]
            {
                Err(anyhow!("haplogroups requires bam_downstream feature"))
            }
        }
        BamStage::Genotyping => {
            #[cfg(feature = "bam_downstream")]
            {
                use bijux_dna_domain_bam::params::BamEffectiveParams;

                let bam = _request.bam.ok_or_else(|| anyhow!("genotyping requires bam"))?;
                let params = crate::params::effective_params_for_stage(stage, _request.params)?;
                let BamEffectiveParams::Genotyping(params) = params else {
                    return Err(anyhow!("genotyping params mismatch"));
                };
                crate::tool_adapters::stages_downstream::genotyping::plan(
                    _request.tool,
                    bam,
                    _request.out_dir,
                    &params,
                )
            }
            #[cfg(not(feature = "bam_downstream"))]
            {
                Err(anyhow!("genotyping requires bam_downstream feature"))
            }
        }
        BamStage::Kinship => {
            #[cfg(feature = "bam_downstream")]
            {
                use bijux_dna_domain_bam::params::BamEffectiveParams;

                let bam = _request.bam.ok_or_else(|| anyhow!("kinship requires bam"))?;
                let params = crate::params::effective_params_for_stage(stage, _request.params)?;
                let BamEffectiveParams::Kinship(params) = params else {
                    return Err(anyhow!("kinship params mismatch"));
                };
                crate::tool_adapters::stages_downstream::kinship::plan(
                    _request.tool,
                    bam,
                    _request.out_dir,
                    &params,
                )
            }
            #[cfg(not(feature = "bam_downstream"))]
            {
                Err(anyhow!("kinship requires bam_downstream feature"))
            }
        }
        _ => Err(anyhow!("stage {} is not handled by the downstream dispatcher", stage.as_str())),
    }
}
