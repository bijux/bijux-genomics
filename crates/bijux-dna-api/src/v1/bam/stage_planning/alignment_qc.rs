use super::{
    anyhow, default_params_for_stage, hash_file_sha256, parse_duplicate_action, parse_flag_list,
    parse_optical_duplicates, parse_read_group_policy, parse_umi_policy, BamRunArgs, BamStage,
    Path, PipelineProfile, Result, StagePlanRequest, StagePlanV1, ToolExecutionSpecV1,
};

#[allow(clippy::too_many_lines)]
pub(super) fn plan_alignment_qc_stage(
    stage: BamStage,
    spec: &ToolExecutionSpecV1,
    args: &BamRunArgs,
    profile: &PipelineProfile,
    out_dir: &Path,
) -> Result<Option<StagePlanV1>> {
    let plan =
        |request: StagePlanRequest<'_>| bijux_dna_planner_bam::stage_api::plan_stage(request);
    let result = match stage {
        bijux_dna_planner_bam::stage_api::BamStage::Align => {
            let r1 = args
                .r1
                .as_deref()
                .ok_or_else(|| anyhow!("--r1 is required for {}", BamStage::Align.as_str()))?;
            let reference = args.reference.as_deref().ok_or_else(|| {
                anyhow!("--reference is required for {}", BamStage::Align.as_str())
            })?;
            let sample_id = args.sample_id.as_deref().ok_or_else(|| {
                anyhow!("--sample-id is required for {}", BamStage::Align.as_str())
            })?;
            let digest = hash_file_sha256(reference)?;
            let mut params = match default_params_for_stage(profile, stage) {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::Align(params) => {
                    params
                }
                _ => bijux_dna_planner_bam::stage_api::params::AlignEffectiveParams {
                    aligner: spec.tool_id.to_string(),
                    preset: args.aligner_preset.clone().unwrap_or_else(|| "default".to_string()),
                    threads: 1,
                    reference: reference.display().to_string(),
                    reference_digest: digest.clone(),
                    rg_policy: bijux_dna_planner_bam::stage_api::types::ReadGroupPolicy::Regenerate,
                    read_group:
                        bijux_dna_planner_bam::stage_api::params::ReadGroupSpec::with_defaults(
                            sample_id,
                        ),
                    sensitivity_profile: Some(
                        args.alignment_sensitivity_profile
                            .clone()
                            .unwrap_or_else(|| "default".to_string()),
                    ),
                    seed_length: args.alignment_seed_length,
                    build_indices: args.build_reference_indices,
                    emit_stats: true,
                },
            };
            params.reference = reference.display().to_string();
            params.reference_digest = digest;
            if let Some(preset) = &args.aligner_preset {
                params.preset.clone_from(preset);
            }
            if let Some(rg) = &args.rg_id {
                params.read_group.id.clone_from(rg);
            }
            if let Some(rg) = &args.rg_sm {
                params.read_group.sample.clone_from(rg);
            }
            if let Some(rg) = &args.rg_pl {
                params.read_group.platform.clone_from(rg);
            }
            if let Some(rg) = &args.rg_lb {
                params.read_group.library.clone_from(rg);
            }
            if let Some(rg) = &args.rg_pu {
                params.read_group.platform_unit = Some(rg.clone());
            }
            if let Some(value) = &args.lane_id {
                params.read_group.lane_id = Some(value.clone());
            }
            if let Some(value) = &args.run_id {
                params.read_group.run_id = Some(value.clone());
            }
            if let Some(policy) = args.rg_policy.as_deref() {
                params.rg_policy = parse_read_group_policy(policy)?;
            }
            if let Some(value) = &args.alignment_sensitivity_profile {
                params.sensitivity_profile = Some(value.clone());
            }
            if let Some(value) = args.alignment_seed_length {
                params.seed_length = Some(value);
            }
            params.aligner = spec.tool_id.to_string();
            params.build_indices = args.build_reference_indices;
            let params_json = serde_json::to_value(&params)?;
            plan(StagePlanRequest {
                stage_id: stage.as_str(),
                tool: spec,
                out_dir,
                bam: None,
                bam_index: None,
                r1: Some(r1),
                r2: args.r2.as_deref(),
                reference: Some(reference),
                sample_id: Some(sample_id),
                params: Some(&params_json),
            })
        }
        bijux_dna_planner_bam::stage_api::BamStage::Validate => plan(StagePlanRequest {
            stage_id: stage.as_str(),
            tool: spec,
            out_dir,
            bam: Some(&args.bam),
            bam_index: args.bai.as_deref(),
            r1: None,
            r2: None,
            reference: args.reference.as_deref(),
            sample_id: args.sample_id.as_deref(),
            params: None,
        }),
        bijux_dna_planner_bam::stage_api::BamStage::QcPre
        | bijux_dna_planner_bam::stage_api::BamStage::MappingSummary => plan(StagePlanRequest {
            stage_id: stage.as_str(),
            tool: spec,
            out_dir,
            bam: Some(&args.bam),
            bam_index: args.bai.as_deref(),
            r1: None,
            r2: None,
            reference: None,
            sample_id: args.sample_id.as_deref(),
            params: None,
        }),
        bijux_dna_planner_bam::stage_api::BamStage::Filter => {
            let stage_key = bijux_dna_core::ids::parse_stage_id(stage.as_str())
                .unwrap_or_else(|_| bijux_dna_core::ids::StageId::new(stage.as_str()));
            let default_params = profile
                .defaults
                .params
                .get(&stage_key)
                .map(bijux_dna_pipelines::DefaultParams::to_json)
                .and_then(|value| stage.parse_effective_params(&value).ok())
                .unwrap_or_else(|| {
                    bijux_dna_planner_bam::stage_api::stage_spec(stage).default_params
                });
            let mut params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::Filter(params) => {
                    params
                }
                _ => bijux_dna_planner_bam::stage_api::params::FilterEffectiveParams {
                    mapq_threshold: 30,
                    include_flags: Vec::new(),
                    exclude_flags: Vec::new(),
                    min_length: 30,
                    remove_duplicates: false,
                    base_quality_threshold: 20,
                },
            };
            if let Some(value) = args.min_mapq {
                params.mapq_threshold = value.try_into().unwrap_or(u8::MAX);
            }
            if !args.include_flags.is_empty() {
                params.include_flags = parse_flag_list(&args.include_flags)?;
            }
            if !args.exclude_flags.is_empty() {
                params.exclude_flags = parse_flag_list(&args.exclude_flags)?;
            }
            if let Some(value) = args.min_length {
                params.min_length = value;
            }
            if args.remove_duplicates {
                params.remove_duplicates = true;
            }
            if let Some(value) = args.base_quality_threshold {
                params.base_quality_threshold = value;
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
        bijux_dna_planner_bam::stage_api::BamStage::MapqFilter => {
            let stage_key = bijux_dna_core::ids::parse_stage_id(stage.as_str())
                .unwrap_or_else(|_| bijux_dna_core::ids::StageId::new(stage.as_str()));
            let default_params = profile
                .defaults
                .params
                .get(&stage_key)
                .map(bijux_dna_pipelines::DefaultParams::to_json)
                .and_then(|value| stage.parse_effective_params(&value).ok())
                .unwrap_or_else(|| {
                    bijux_dna_planner_bam::stage_api::stage_spec(stage).default_params
                });
            let mut params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::MapqFilter(
                    params,
                ) => params,
                _ => bijux_dna_planner_bam::stage_api::params::FilterEffectiveParams {
                    mapq_threshold: 30,
                    include_flags: Vec::new(),
                    exclude_flags: Vec::new(),
                    min_length: 0,
                    remove_duplicates: false,
                    base_quality_threshold: 20,
                },
            };
            if let Some(value) = args.min_mapq {
                params.mapq_threshold = value.try_into().unwrap_or(u8::MAX);
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
        bijux_dna_planner_bam::stage_api::BamStage::LengthFilter => {
            let stage_key = bijux_dna_core::ids::parse_stage_id(stage.as_str())
                .unwrap_or_else(|_| bijux_dna_core::ids::StageId::new(stage.as_str()));
            let default_params = profile
                .defaults
                .params
                .get(&stage_key)
                .map(bijux_dna_pipelines::DefaultParams::to_json)
                .and_then(|value| stage.parse_effective_params(&value).ok())
                .unwrap_or_else(|| {
                    bijux_dna_planner_bam::stage_api::stage_spec(stage).default_params
                });
            let mut params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::LengthFilter(
                    params,
                ) => params,
                _ => bijux_dna_planner_bam::stage_api::params::FilterEffectiveParams {
                    mapq_threshold: 0,
                    include_flags: Vec::new(),
                    exclude_flags: Vec::new(),
                    min_length: 30,
                    remove_duplicates: false,
                    base_quality_threshold: 20,
                },
            };
            if let Some(value) = args.min_length {
                params.min_length = value;
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
        bijux_dna_planner_bam::stage_api::BamStage::Markdup => {
            let stage_key = bijux_dna_core::ids::parse_stage_id(stage.as_str())
                .unwrap_or_else(|_| bijux_dna_core::ids::StageId::new(stage.as_str()));
            let default_params = profile
                .defaults
                .params
                .get(&stage_key)
                .map(bijux_dna_pipelines::DefaultParams::to_json)
                .and_then(|value| stage.parse_effective_params(&value).ok())
                .unwrap_or_else(|| {
                    bijux_dna_planner_bam::stage_api::stage_spec(stage).default_params
                });
            let mut params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::Markdup(params) => {
                    params
                }
                _ => bijux_dna_planner_bam::stage_api::params::MarkDupEffectiveParams {
                    optical_duplicates:
                        bijux_dna_planner_bam::stage_api::params::OpticalDuplicatePolicy::MarkOnly,
                    umi_policy: bijux_dna_planner_bam::stage_api::params::UmiPolicy::Ignore,
                    duplicate_action:
                        bijux_dna_planner_bam::stage_api::params::DuplicateAction::Mark,
                },
            };
            if let Some(value) = args.optical_duplicates.as_deref() {
                params.optical_duplicates = parse_optical_duplicates(value)?;
            }
            if let Some(value) = args.umi_policy.as_deref() {
                params.umi_policy = parse_umi_policy(value)?;
            }
            if let Some(value) = args.duplicate_action.as_deref() {
                params.duplicate_action = parse_duplicate_action(value)?;
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
        bijux_dna_planner_bam::stage_api::BamStage::DuplicationMetrics => {
            let default_params = default_params_for_stage(profile, stage);
            let params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::DuplicationMetrics(
                    params,
                ) => params,
                _ => bijux_dna_planner_bam::stage_api::params::MarkDupEffectiveParams {
                    optical_duplicates:
                        bijux_dna_planner_bam::stage_api::params::OpticalDuplicatePolicy::MarkOnly,
                    umi_policy: bijux_dna_planner_bam::stage_api::params::UmiPolicy::Ignore,
                    duplicate_action:
                        bijux_dna_planner_bam::stage_api::params::DuplicateAction::Mark,
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
                reference: None,
                sample_id: args.sample_id.as_deref(),
                params: Some(&params_json),
            })
        }
        bijux_dna_planner_bam::stage_api::BamStage::Complexity => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::Complexity(
                    params,
                ) => params,
                _ => bijux_dna_planner_bam::stage_api::params::ComplexityEffectiveParams {
                    min_reads: 100_000,
                    projection_points: vec![1_000_000, 2_000_000, 5_000_000],
                },
            };
            if let Some(value) = args.complexity_min_reads {
                params.min_reads = u64::from(value);
            }
            if !args.complexity_projection_points.is_empty() {
                params.projection_points.clone_from(&args.complexity_projection_points);
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
        bijux_dna_planner_bam::stage_api::BamStage::Coverage => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::Coverage(params) => {
                    params
                }
                _ => bijux_dna_planner_bam::stage_api::params::CoverageEffectiveParams {
                    regions: None,
                    depth_thresholds: vec![1, 3, 5],
                },
            };
            if let Some(value) = args.regions.as_deref() {
                params.regions = Some(bijux_dna_planner_bam::stage_api::types::BedRegions(
                    std::path::PathBuf::from(value),
                ));
            }
            if !args.depth_thresholds.is_empty() {
                params.depth_thresholds.clone_from(&args.depth_thresholds);
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
        bijux_dna_planner_bam::stage_api::BamStage::InsertSize => {
            let default_params = default_params_for_stage(profile, stage);
            let params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::InsertSize(
                    params,
                ) => params,
                _ => bijux_dna_planner_bam::stage_api::params::CoverageEffectiveParams {
                    regions: None,
                    depth_thresholds: vec![1],
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
        bijux_dna_planner_bam::stage_api::BamStage::GcBias => {
            let default_params = default_params_for_stage(profile, stage);
            let params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::GcBias(params) => {
                    params
                }
                _ => bijux_dna_planner_bam::stage_api::params::CoverageEffectiveParams {
                    regions: None,
                    depth_thresholds: vec![1],
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
        bijux_dna_planner_bam::stage_api::BamStage::EndogenousContent => {
            let default_params = default_params_for_stage(profile, stage);
            let params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::EndogenousContent(
                    params,
                ) => params,
                _ => bijux_dna_planner_bam::stage_api::params::CoverageEffectiveParams {
                    regions: None,
                    depth_thresholds: vec![1],
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
        _ => return Ok(None),
    }?;
    Ok(Some(result))
}
