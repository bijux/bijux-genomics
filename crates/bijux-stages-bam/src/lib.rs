//! BAM stage planning and contract access.

pub mod bam;
pub mod bam_tools_registry;
pub mod metrics;
pub mod observer;
pub mod plugin;
pub mod stages_adna;
#[cfg(feature = "bam_downstream")]
pub mod stages_downstream;
pub mod stages_post;
pub mod stages_pre;
pub mod stages_support;
pub mod tools;

pub use bijux_core::{ArtifactRef, StageIO, StagePlanJsonV1, StagePlanV1};
pub type StagePlanJson = StagePlanJsonV1;

pub use bijux_domain_bam as domain_bam;

pub struct StagePlanRequest<'a> {
    pub stage_id: &'a str,
    pub tool: &'a bijux_core::contract::ToolExecutionSpecV1,
    pub out_dir: &'a std::path::Path,
    pub bam: Option<&'a std::path::Path>,
    pub bam_index: Option<&'a std::path::Path>,
    pub r1: Option<&'a std::path::Path>,
    pub r2: Option<&'a std::path::Path>,
    pub reference: Option<&'a std::path::Path>,
    pub sample_id: Option<&'a str>,
    pub params: Option<&'a serde_json::Value>,
}

fn effective_params_for_stage(
    stage: bijux_domain_bam::BamStage,
    params: Option<&serde_json::Value>,
) -> anyhow::Result<bijux_domain_bam::params::BamEffectiveParams> {
    if let Some(value) = params {
        return stage.parse_effective_params(value);
    }
    Ok(bijux_domain_bam::stage_spec(stage).default_params.clone())
}

/// # Errors
/// Returns an error if the stage cannot be planned with the provided inputs.
#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
pub fn plan_stage(request: StagePlanRequest<'_>) -> anyhow::Result<StagePlanV1> {
    let stage = bijux_domain_bam::BamStage::try_from(request.stage_id)?;
    match stage {
        bijux_domain_bam::BamStage::Align => {
            let r1 = request
                .r1
                .ok_or_else(|| anyhow::anyhow!("align requires r1"))?;
            let reference = request
                .reference
                .ok_or_else(|| anyhow::anyhow!("align requires reference"))?;
            let sample_id = request
                .sample_id
                .ok_or_else(|| anyhow::anyhow!("align requires sample_id"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Align(params) = params else {
                return Err(anyhow::anyhow!("align params mismatch"));
            };
            stages_pre::align::plan(
                request.tool,
                r1,
                request.r2,
                reference,
                sample_id,
                &params,
                request.out_dir,
            )
        }
        bijux_domain_bam::BamStage::Validate => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow::anyhow!("validate requires bam"))?;
            stages_pre::validate::plan(
                request.tool,
                bam,
                request.bam_index,
                request.reference,
                request.out_dir,
            )
        }
        bijux_domain_bam::BamStage::QcPre => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow::anyhow!("qc_pre requires bam"))?;
            stages_pre::qc_pre::plan(request.tool, bam, request.out_dir)
        }
        bijux_domain_bam::BamStage::Filter => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow::anyhow!("filter requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Filter(params) = params else {
                return Err(anyhow::anyhow!("filter params mismatch"));
            };
            stages_pre::filter::plan(request.tool, bam, request.out_dir, &params)
        }
        bijux_domain_bam::BamStage::Markdup => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow::anyhow!("markdup requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Markdup(params) = params else {
                return Err(anyhow::anyhow!("markdup params mismatch"));
            };
            stages_post::markdup::plan(request.tool, bam, request.out_dir, &params)
        }
        bijux_domain_bam::BamStage::Complexity => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow::anyhow!("complexity requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Complexity(params) = params else {
                return Err(anyhow::anyhow!("complexity params mismatch"));
            };
            stages_post::complexity::plan(request.tool, bam, request.out_dir, &params)
        }
        bijux_domain_bam::BamStage::Coverage => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow::anyhow!("coverage requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Coverage(params) = params else {
                return Err(anyhow::anyhow!("coverage params mismatch"));
            };
            stages_post::coverage::plan(request.tool, bam, request.out_dir, &params)
        }
        bijux_domain_bam::BamStage::Recalibration => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow::anyhow!("recalibration requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Recalibration(params) = params else {
                return Err(anyhow::anyhow!("recalibration params mismatch"));
            };
            stages_post::recalibration::plan(request.tool, bam, request.out_dir, &params)
        }
        bijux_domain_bam::BamStage::Damage => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow::anyhow!("damage requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Damage(params) = params else {
                return Err(anyhow::anyhow!("damage params mismatch"));
            };
            stages_adna::damage::plan(request.tool, bam, request.out_dir, &params)
        }
        bijux_domain_bam::BamStage::Authenticity => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow::anyhow!("authenticity requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Authenticity(params) = params else {
                return Err(anyhow::anyhow!("authenticity params mismatch"));
            };
            stages_adna::authenticity::plan(request.tool, bam, request.out_dir, &params)
        }
        bijux_domain_bam::BamStage::Contamination => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow::anyhow!("contamination requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Contamination(params) = params else {
                return Err(anyhow::anyhow!("contamination params mismatch"));
            };
            stages_adna::contamination::plan(request.tool, bam, request.out_dir, &params)
        }
        bijux_domain_bam::BamStage::Sex => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow::anyhow!("sex requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Sex(params) = params else {
                return Err(anyhow::anyhow!("sex params mismatch"));
            };
            stages_adna::sex::plan(request.tool, bam, request.out_dir, &params)
        }
        bijux_domain_bam::BamStage::BiasMitigation => {
            #[cfg(feature = "bam_downstream")]
            {
                let bam = request
                    .bam
                    .ok_or_else(|| anyhow::anyhow!("bias_mitigation requires bam"))?;
                let params = effective_params_for_stage(stage, request.params)?;
                let bijux_domain_bam::params::BamEffectiveParams::BiasMitigation(params) = params
                else {
                    return Err(anyhow::anyhow!("bias_mitigation params mismatch"));
                };
                stages_downstream::bias_mitigation::plan(
                    request.tool,
                    bam,
                    request.out_dir,
                    &params,
                )
            }
            #[cfg(not(feature = "bam_downstream"))]
            {
                return Err(anyhow::anyhow!(
                    "bias_mitigation planning requires bam_downstream feature"
                ));
            }
        }
        bijux_domain_bam::BamStage::Haplogroups => {
            #[cfg(feature = "bam_downstream")]
            {
                let bam = request
                    .bam
                    .ok_or_else(|| anyhow::anyhow!("haplogroups requires bam"))?;
                let params = effective_params_for_stage(stage, request.params)?;
                let bijux_domain_bam::params::BamEffectiveParams::Haplogroups(params) = params
                else {
                    return Err(anyhow::anyhow!("haplogroups params mismatch"));
                };
                stages_downstream::haplogroups::plan(request.tool, bam, request.out_dir, &params)
            }
            #[cfg(not(feature = "bam_downstream"))]
            {
                return Err(anyhow::anyhow!(
                    "haplogroups planning requires bam_downstream feature"
                ));
            }
        }
        bijux_domain_bam::BamStage::Genotyping => {
            #[cfg(feature = "bam_downstream")]
            {
                let bam = request
                    .bam
                    .ok_or_else(|| anyhow::anyhow!("genotyping requires bam"))?;
                let params = effective_params_for_stage(stage, request.params)?;
                let bijux_domain_bam::params::BamEffectiveParams::Genotyping(params) = params
                else {
                    return Err(anyhow::anyhow!("genotyping params mismatch"));
                };
                stages_downstream::genotyping::plan(request.tool, bam, request.out_dir, &params)
            }
            #[cfg(not(feature = "bam_downstream"))]
            {
                return Err(anyhow::anyhow!(
                    "genotyping planning requires bam_downstream feature"
                ));
            }
        }
        bijux_domain_bam::BamStage::Kinship => {
            #[cfg(feature = "bam_downstream")]
            {
                let bam = request
                    .bam
                    .ok_or_else(|| anyhow::anyhow!("kinship requires bam"))?;
                let params = effective_params_for_stage(stage, request.params)?;
                let bijux_domain_bam::params::BamEffectiveParams::Kinship(params) = params else {
                    return Err(anyhow::anyhow!("kinship params mismatch"));
                };
                stages_downstream::kinship::plan(request.tool, bam, request.out_dir, &params)
            }
            #[cfg(not(feature = "bam_downstream"))]
            {
                return Err(anyhow::anyhow!(
                    "kinship planning requires bam_downstream feature"
                ));
            }
        }
    }
}
