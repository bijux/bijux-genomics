#[cfg(not(feature = "bam_downstream"))]
use anyhow::anyhow;

#[cfg(feature = "bam_downstream")]
use super::default_params_for_stage;
use super::{
    BamRunArgs, BamStage, Path, PipelineProfile, Result, StagePlanRequest, StagePlanV1,
    ToolExecutionSpecV1,
};

#[cfg_attr(not(feature = "bam_downstream"), allow(unused_variables))]
#[allow(clippy::too_many_lines)]
pub(super) fn plan_downstream_stage(
    stage: BamStage,
    spec: &ToolExecutionSpecV1,
    args: &BamRunArgs,
    profile: &PipelineProfile,
    out_dir: &Path,
) -> Result<Option<StagePlanV1>> {
    let plan =
        |request: StagePlanRequest<'_>| bijux_dna_planner_bam::stage_api::plan_stage(request);
    let result = match stage {
        #[cfg(feature = "bam_downstream")]
        bijux_dna_planner_bam::stage_api::BamStage::Haplogroups => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::Haplogroups(
                    params,
                ) => params,
                _ => bijux_dna_planner_bam::stage_api::params::HaplogroupEffectiveParams {
                    reference_panel: "mito_default".to_string(),
                    reference_build: "rCRS".to_string(),
                    min_coverage: None,
                    population_scope: Some("mitochondrial_haplogroup_reference".to_string()),
                    refuse_without_population_context: true,
                },
            };
            if let Some(value) = args.haplogroup_panel.clone() {
                params.reference_panel = value;
            }
            if let Some(value) = args.haplogroup_min_coverage {
                params.min_coverage = Some(value);
            }
            let params_json = serde_json::to_value(&params)?;
            plan(StagePlanRequest {
                stage_id: stage.as_str(),
                tool: spec,
                out_dir,
                bam: Some(&args.bam),
                bam_index: args.bai.as_deref(),
                r1: None,
                r2: None,
                reference: None,
                sample_id: args.sample_id.as_deref(),
                params: Some(&params_json),
            })
        }
        #[cfg(feature = "bam_downstream")]
        bijux_dna_planner_bam::stage_api::BamStage::Genotyping => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::Genotyping(
                    params,
                ) => params,
                _ => bijux_dna_planner_bam::stage_api::params::GenotypingEffectiveParams {
                    caller: "angsd".to_string(),
                    min_posterior: None,
                    min_call_rate: None,
                },
            };
            if let Some(value) = args.caller.clone() {
                params.caller = value;
            }
            if let Some(value) = args.min_posterior {
                params.min_posterior = Some(value);
            }
            if let Some(value) = args.min_call_rate {
                params.min_call_rate = Some(value);
            }
            let params_json = serde_json::to_value(&params)?;
            plan(StagePlanRequest {
                stage_id: stage.as_str(),
                tool: spec,
                out_dir,
                bam: Some(&args.bam),
                bam_index: args.bai.as_deref(),
                r1: None,
                r2: None,
                reference: None,
                sample_id: args.sample_id.as_deref(),
                params: Some(&params_json),
            })
        }
        #[cfg(feature = "bam_downstream")]
        bijux_dna_planner_bam::stage_api::BamStage::Kinship => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::Kinship(params) => {
                    params
                }
                _ => bijux_dna_planner_bam::stage_api::params::KinshipEffectiveParams {
                    reference_panel: "king_default".to_string(),
                    reference_build: "grch38".to_string(),
                    population_scope: "human_diploid_panel".to_string(),
                    min_overlap_snps: 1000,
                    requires_cohort_context: true,
                },
            };
            if let Some(value) = args.kinship_panel.clone() {
                params.reference_panel = value;
            }
            if let Some(value) = args.min_overlap_snps {
                params.min_overlap_snps = value;
            }
            let params_json = serde_json::to_value(&params)?;
            plan(StagePlanRequest {
                stage_id: stage.as_str(),
                tool: spec,
                out_dir,
                bam: Some(&args.bam),
                bam_index: args.bai.as_deref(),
                r1: None,
                r2: None,
                reference: None,
                sample_id: args.sample_id.as_deref(),
                params: Some(&params_json),
            })
        }
        #[cfg(not(feature = "bam_downstream"))]
        bijux_dna_planner_bam::stage_api::BamStage::Haplogroups
        | bijux_dna_planner_bam::stage_api::BamStage::Genotyping
        | bijux_dna_planner_bam::stage_api::BamStage::Kinship => {
            Err(anyhow!("downstream BAM stages are disabled (enable feature 'bam_downstream')"))
        }
        _ => return Ok(None),
    }?;
    Ok(Some(result))
}
