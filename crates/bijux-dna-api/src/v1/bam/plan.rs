use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_infra::hash_file_sha256;
use bijux_dna_pipelines::registry;
use bijux_dna_pipelines::PipelineProfile;
use bijux_dna_planner_bam::stage_api::{BamStage, StagePlanRequest};
use bijux_dna_stage_contract::StagePlanV1;

use crate::request_args::BamRunArgs;
#[path = "plan_parse.rs"]
mod plan_parse;
use self::plan_parse::{
    default_params_for_stage, parse_bqsr_mode, parse_contamination_scope, parse_duplicate_action,
    parse_expected_sex, parse_flag_list, parse_optical_duplicates, parse_read_group_policy,
    parse_udg_model, parse_umi_policy,
};

fn stage_status(stage_id: &str) -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let path = bijux_dna_infra::configs_file(&cwd, "ci/stages/stages.toml");
    let raw = std::fs::read_to_string(path).ok()?;
    let parsed = raw.parse::<toml::Value>().ok()?;
    let entries = parsed.get("stages")?.as_array()?;
    entries.iter().find_map(|entry| {
        let id = entry.get("id").and_then(toml::Value::as_str)?;
        if id == stage_id {
            entry
                .get("status")
                .and_then(toml::Value::as_str)
                .map(std::string::ToString::to_string)
        } else {
            None
        }
    })
}

fn enforce_stage_status(stage_id: &str, allow_planned: bool) -> Result<()> {
    match stage_status(stage_id).as_deref() {
        Some("supported") | None => Ok(()),
        Some("planned" | "out_of_scope") if allow_planned => Ok(()),
        Some("planned" | "out_of_scope") => Err(anyhow!(
            "stage {stage_id} is not active in current scope; re-run with --allow-planned to override"
        )),
        Some(other) => Err(anyhow!("stage {stage_id} has unknown status {other}")),
    }
}

/// # Errors
/// Returns an error if planning fails for the stage.
pub fn plan_for_bam_stage(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    spec: &ToolExecutionSpecV1,
    args: &BamRunArgs,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let profile = registry::profile_by_id(bijux_dna_pipelines::Domain::Bam, &args.profile)?;
    plan_for_bam_stage_with_profile(stage, spec, args, &profile, out_dir)
}

#[allow(clippy::too_many_lines)]
/// # Errors
/// Returns an error if stage arguments are invalid or planning fails.
pub fn plan_for_bam_stage_with_profile(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    spec: &ToolExecutionSpecV1,
    args: &BamRunArgs,
    profile: &PipelineProfile,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    enforce_stage_status(stage.as_str(), args.allow_planned)?;
    if !super::feature_flags::downstream_enabled()
        && matches!(
            stage,
            bijux_dna_planner_bam::stage_api::BamStage::Haplogroups
                | bijux_dna_planner_bam::stage_api::BamStage::Genotyping
                | bijux_dna_planner_bam::stage_api::BamStage::Kinship
                | bijux_dna_planner_bam::stage_api::BamStage::BiasMitigation
        )
    {
        return Err(anyhow!(
            "downstream BAM stages are disabled (enable feature 'bam_downstream')"
        ));
    }
    let plan =
        |request: StagePlanRequest<'_>| bijux_dna_planner_bam::stage_api::plan_stage(request);
    match stage {
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
                    preset: args
                        .aligner_preset
                        .clone()
                        .unwrap_or_else(|| "default".to_string()),
                    threads: 1,
                    reference: reference.display().to_string(),
                    reference_digest: digest.clone(),
                    rg_policy: bijux_dna_planner_bam::stage_api::types::ReadGroupPolicy::Regenerate,
                    read_group:
                        bijux_dna_planner_bam::stage_api::params::ReadGroupSpec::with_defaults(
                            sample_id,
                        ),
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
            if let Some(policy) = args.rg_policy.as_deref() {
                params.rg_policy = parse_read_group_policy(policy)?;
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
        bijux_dna_planner_bam::stage_api::BamStage::QcPre => plan(StagePlanRequest {
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
        bijux_dna_planner_bam::stage_api::BamStage::MappingSummary => plan(StagePlanRequest {
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
            let stage_key = bijux_dna_core::ids::StageId::from_static(stage.as_str());
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
            let stage_key = bijux_dna_core::ids::StageId::from_static(stage.as_str());
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
            let stage_key = bijux_dna_core::ids::StageId::from_static(stage.as_str());
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
            let stage_key = bijux_dna_core::ids::StageId::from_static(stage.as_str());
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
                params
                    .projection_points
                    .clone_from(&args.complexity_projection_points);
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
                reference: None,
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
                reference: None,
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
                reference: None,
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
                },
            };
            if !args.contamination_panel.is_empty() {
                params
                    .reference_panels
                    .clone_from(&args.contamination_panel);
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
        #[cfg(feature = "bam_downstream")]
        bijux_dna_planner_bam::stage_api::BamStage::Haplogroups => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_dna_planner_bam::stage_api::params::BamEffectiveParams::Haplogroups(
                    params,
                ) => params,
                _ => bijux_dna_planner_bam::stage_api::params::HaplogroupEffectiveParams {
                    reference_panel: "mito_default".to_string(),
                    min_coverage: None,
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
                    min_overlap_snps: 1000,
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
        | bijux_dna_planner_bam::stage_api::BamStage::Kinship => Err(anyhow!(
            "downstream BAM stages are disabled (enable feature 'bam_downstream')"
        )),
    }
}
