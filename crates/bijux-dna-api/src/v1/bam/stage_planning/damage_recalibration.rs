#[cfg(not(feature = "bam_downstream"))]
use anyhow::anyhow;

use super::{
    default_params_for_stage, hash_file_sha256, parse_bqsr_mode, parse_contamination_scope,
    parse_expected_sex, parse_udg_model, BamRunArgs, BamStage, Path, PipelineProfile, Result,
    StagePlanRequest, StagePlanV1, ToolExecutionSpecV1,
};

#[allow(clippy::too_many_lines)]
pub(super) fn plan_damage_recalibration_stage(
    stage: BamStage,
    spec: &ToolExecutionSpecV1,
    args: &BamRunArgs,
    profile: &PipelineProfile,
    out_dir: &Path,
) -> Result<Option<StagePlanV1>> {
    let plan =
        |request: StagePlanRequest<'_>| bijux_dna_planner_bam::stage_api::plan_stage(request);
    let result = match stage {
        bijux_dna_planner_bam::stage_api::BamStage::OverlapCorrection => {
            let default_params = default_params_for_stage(profile, stage);
            let params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::OverlapCorrection(
                    params,
                ) => params,
                _ => bijux_dna_planner_bam::stage_api::params::FilterEffectiveParams {
                    mapq_threshold: 0,
                    include_flags: Vec::new(),
                    exclude_flags: Vec::new(),
                    min_length: 0,
                    remove_duplicates: false,
                    base_quality_threshold: 20,
                },
            };
            let params_json = serde_json::to_value(&params)?;
            plan(StagePlanRequest {
                stage_id: stage.as_str(),
                tool: spec,
                out_dir,
                bam: Some(&args.bam),
                bam_index: args.bai.as_deref(),
                r1: None,
                r2: None,
                reference: args.reference.as_deref(),
                sample_id: args.sample_id.as_deref(),
                params: Some(&params_json),
            })
        }
        bijux_dna_planner_bam::stage_api::BamStage::Damage => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::Damage(params) => {
                    params
                }
                _ => bijux_dna_planner_bam::stage_api::params::DamageEffectiveParams {
                    udg_model: bijux_dna_planner_bam::stage_api::params::UdgModel::NonUdg,
                    pmd_threshold_5p: 0.3,
                    pmd_threshold_3p: 0.3,
                    trim_5p: 0,
                    trim_3p: 0,
                    damage_tool_profile: Some("ancient_dna_evidence".to_string()),
                    evidence_only: true,
                },
            };
            if let Some(value) = args.udg_model.as_deref() {
                params.udg_model = parse_udg_model(value)?;
            }
            if let Some(value) = args.pmd_threshold_5p {
                params.pmd_threshold_5p = value;
            }
            if let Some(value) = args.pmd_threshold_3p {
                params.pmd_threshold_3p = value;
            }
            if let Some(value) = args.trim_5p {
                params.trim_5p = value.try_into().unwrap_or(u8::MAX);
            }
            if let Some(value) = args.trim_3p {
                params.trim_3p = value.try_into().unwrap_or(u8::MAX);
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
                reference: args.reference.as_deref(),
                sample_id: args.sample_id.as_deref(),
                params: Some(&params_json),
            })
        }
        bijux_dna_planner_bam::stage_api::BamStage::Authenticity => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::Authenticity(
                    params,
                ) => params,
                _ => bijux_dna_planner_bam::stage_api::params::AuthenticityEffectiveParams {
                    mode: "aggregate".to_string(),
                    evidence_only: true,
                    disallow_certification: true,
                },
            };
            if let Some(value) = args.authenticity_mode.clone() {
                params.mode = value;
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
                reference: args.reference.as_deref(),
                sample_id: args.sample_id.as_deref(),
                params: Some(&params_json),
            })
        }
        bijux_dna_planner_bam::stage_api::BamStage::Contamination => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::Contamination(
                    params,
                ) => params,
                _ => bijux_dna_planner_bam::stage_api::params::ContaminationEffectiveParams {
                    reference_panels: Vec::new(),
                    scope: bijux_dna_planner_bam::stage_api::params::ContaminationScope::Both,
                    prior: None,
                    sex_specific: false,
                    assumptions: None,
                    required_reference_digest: None,
                    chromosome_system: Some("xy".to_string()),
                    minimum_mean_coverage: Some(0.5),
                    emit_confidence_caveats: true,
                },
            };
            if !args.contamination_panel.is_empty() {
                params.reference_panels.clone_from(&args.contamination_panel);
            }
            if let Some(value) = args.contamination_scope.as_deref() {
                params.scope = parse_contamination_scope(value)?;
            }
            if let Some(value) = args.contamination_prior {
                params.prior = Some(value);
            }
            if args.sex_specific_contamination {
                params.sex_specific = true;
            }
            if let Some(value) = args.contamination_assumptions.clone() {
                params.assumptions = Some(value);
            }
            if let Some(reference) = args.reference.as_deref() {
                params.required_reference_digest = Some(hash_file_sha256(reference)?);
            }
            if args.sex_specific_contamination {
                params.chromosome_system = Some("xy".to_string());
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
        bijux_dna_planner_bam::stage_api::BamStage::Sex => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::Sex(params) => params,
                _ => bijux_dna_planner_bam::stage_api::params::SexEffectiveParams {
                    expected_sex: None,
                    method: "rxy".to_string(),
                    chromosome_system: Some("xy".to_string()),
                    minimum_y_sites: Some(100),
                    refuse_without_context: true,
                },
            };
            if let Some(value) = args.expected_sex.as_deref() {
                params.expected_sex = Some(parse_expected_sex(value)?);
            }
            if !args.sex_method.is_empty() {
                params.method.clone_from(&args.sex_method);
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
        bijux_dna_planner_bam::stage_api::BamStage::BiasMitigation => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::BiasMitigation(
                    params,
                ) => params,
                _ => bijux_dna_planner_bam::stage_api::params::BiasMitigationEffectiveParams {
                    gc_bias_correction: false,
                    map_bias_correction: false,
                },
            };
            if args.gc_bias_correction {
                params.gc_bias_correction = true;
            }
            if args.map_bias_correction {
                params.map_bias_correction = true;
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
        bijux_dna_planner_bam::stage_api::BamStage::BiasMitigation => Err(anyhow!(format!(
            "{} is disabled without feature 'bam_downstream'",
            BamStage::BiasMitigation.as_str()
        ))),
        bijux_dna_planner_bam::stage_api::BamStage::Recalibration => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::Recalibration(
                    params,
                ) => params,
                _ => bijux_dna_planner_bam::stage_api::params::BqsrEffectiveParams {
                    known_sites: Vec::new(),
                    mode: bijux_dna_planner_bam::stage_api::params::BqsrMode::Skip,
                    skip_criteria:
                        bijux_dna_planner_bam::stage_api::params::RecalibrationSkipCriteria {
                            min_mean_coverage: 1.0,
                            min_breadth_1x: 0.1,
                        },
                },
            };
            if !args.known_sites.is_empty() {
                params.known_sites.clone_from(&args.known_sites);
            }
            if let Some(value) = args.bqsr_mode.as_deref() {
                params.mode = parse_bqsr_mode(value)?;
            }
            if let Some(value) = args.bqsr_min_mean_coverage {
                params.skip_criteria.min_mean_coverage = value;
            }
            if let Some(value) = args.bqsr_min_breadth_1x {
                params.skip_criteria.min_breadth_1x = value;
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
        _ => return Ok(None),
    }?;
    Ok(Some(result))
}
