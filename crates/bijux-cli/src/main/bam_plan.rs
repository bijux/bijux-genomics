use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_core::{StagePlanV1, ToolExecutionSpecV1};
use bijux_engine::api::hash_file_sha256;
use bijux_pipelines::registry;
use bijux_pipelines::PipelineProfile;

use crate::cli::parse::BamRunArgs;

pub(crate) fn plan_for_bam_stage(
    stage: bijux_domain_bam::BamStage,
    spec: &ToolExecutionSpecV1,
    args: &BamRunArgs,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let profile = registry::profile_by_id(bijux_pipelines::Domain::Bam, &args.profile)?;
    plan_for_bam_stage_with_profile(stage, spec, args, &profile, out_dir)
}

#[allow(clippy::too_many_lines)]
pub(crate) fn plan_for_bam_stage_with_profile(
    stage: bijux_domain_bam::BamStage,
    spec: &ToolExecutionSpecV1,
    args: &BamRunArgs,
    profile: &PipelineProfile,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    if !crate::downstream_enabled()
        && matches!(
            stage,
            bijux_domain_bam::BamStage::Haplogroups
                | bijux_domain_bam::BamStage::Genotyping
                | bijux_domain_bam::BamStage::Kinship
                | bijux_domain_bam::BamStage::BiasMitigation
        )
    {
        return Err(anyhow!(
            "downstream BAM stages are disabled (enable feature 'bam_downstream')"
        ));
    }
    match stage {
        bijux_domain_bam::BamStage::Align => {
            let r1 = args
                .r1
                .as_deref()
                .ok_or_else(|| anyhow!("--r1 is required for bam.align"))?;
            let reference = args
                .reference
                .as_deref()
                .ok_or_else(|| anyhow!("--reference is required for bam.align"))?;
            let sample_id = args
                .sample_id
                .as_deref()
                .ok_or_else(|| anyhow!("--sample-id is required for bam.align"))?;
            let digest = hash_file_sha256(reference)?;
            let mut params = match default_params_for_stage(profile, stage) {
                bijux_domain_bam::params::BamEffectiveParams::Align(params) => params,
                _ => bijux_domain_bam::params::AlignEffectiveParams {
                    aligner: spec.tool_id.0.clone(),
                    preset: args
                        .aligner_preset
                        .clone()
                        .unwrap_or_else(|| "default".to_string()),
                    threads: 1,
                    reference: reference.display().to_string(),
                    reference_digest: digest.clone(),
                    rg_policy: bijux_domain_bam::types::ReadGroupPolicy::Regenerate,
                    read_group: bijux_domain_bam::params::ReadGroupSpec::with_defaults(sample_id),
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
            if let Some(policy) = args.rg_policy {
                params.rg_policy = policy.into();
            }
            params.aligner.clone_from(&spec.tool_id.0);
            params.build_indices = args.build_reference_indices;
            bijux_stages_bam::bam::align::plan(
                spec,
                r1,
                args.r2.as_deref(),
                reference,
                sample_id,
                &params,
                out_dir,
            )
        }
        bijux_domain_bam::BamStage::Validate => bijux_stages_bam::bam::validate::plan(
            spec,
            &args.bam,
            args.bai.as_deref(),
            args.reference.as_deref(),
            out_dir,
        ),
        bijux_domain_bam::BamStage::QcPre => {
            bijux_stages_bam::bam::qc_pre::plan(spec, &args.bam, out_dir)
        }
        bijux_domain_bam::BamStage::Filter => {
            let default_params = profile
                .defaults
                .params
                .get(stage.as_str())
                .and_then(|value| stage.parse_effective_params(value).ok())
                .unwrap_or_else(|| bijux_domain_bam::stage_spec(stage).default_params);
            let mut params = match default_params {
                bijux_domain_bam::params::BamEffectiveParams::Filter(params) => params,
                _ => bijux_domain_bam::params::FilterEffectiveParams {
                    mapq_threshold: 30,
                    include_flags: Vec::new(),
                    exclude_flags: Vec::new(),
                    min_length: 30,
                    remove_duplicates: false,
                    base_quality_threshold: 20,
                },
            };
            if let Some(value) = args.min_mapq {
                params.mapq_threshold = value;
            }
            if !args.include_flags.is_empty() {
                params.include_flags.clone_from(&args.include_flags);
            }
            if !args.exclude_flags.is_empty() {
                params.exclude_flags.clone_from(&args.exclude_flags);
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
            bijux_stages_bam::bam::filter::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Markdup => {
            let default_params = profile
                .defaults
                .params
                .get(stage.as_str())
                .and_then(|value| stage.parse_effective_params(value).ok())
                .unwrap_or_else(|| bijux_domain_bam::stage_spec(stage).default_params);
            let mut params = match default_params {
                bijux_domain_bam::params::BamEffectiveParams::Markdup(params) => params,
                _ => bijux_domain_bam::params::MarkDupEffectiveParams {
                    optical_duplicates: bijux_domain_bam::params::OpticalDuplicatePolicy::MarkOnly,
                    umi_policy: bijux_domain_bam::params::UmiPolicy::Ignore,
                    duplicate_action: bijux_domain_bam::params::DuplicateAction::Mark,
                },
            };
            if let Some(value) = args.optical_duplicates {
                params.optical_duplicates = value.into();
            }
            if let Some(value) = args.umi_policy {
                params.umi_policy = value.into();
            }
            if let Some(value) = args.duplicate_action {
                params.duplicate_action = value.into();
            }
            bijux_stages_bam::bam::markdup::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Complexity => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_domain_bam::params::BamEffectiveParams::Complexity(params) => params,
                _ => bijux_domain_bam::params::ComplexityEffectiveParams {
                    min_reads: 100_000,
                    projection_points: vec![1_000_000, 2_000_000, 5_000_000],
                },
            };
            if let Some(value) = args.complexity_min_reads {
                params.min_reads = value;
            }
            if !args.complexity_projection_points.is_empty() {
                params
                    .projection_points
                    .clone_from(&args.complexity_projection_points);
            }
            bijux_stages_bam::bam::complexity::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Coverage => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_domain_bam::params::BamEffectiveParams::Coverage(params) => params,
                _ => bijux_domain_bam::params::CoverageEffectiveParams {
                    regions: None,
                    depth_thresholds: vec![1, 3, 5],
                },
            };
            if let Some(value) = args.regions.clone() {
                params.regions = Some(bijux_domain_bam::types::BedRegions(value));
            }
            if !args.depth_thresholds.is_empty() {
                params.depth_thresholds.clone_from(&args.depth_thresholds);
            }
            bijux_stages_bam::bam::coverage::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Damage => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_domain_bam::params::BamEffectiveParams::Damage(params) => params,
                _ => bijux_domain_bam::params::DamageEffectiveParams {
                    udg_model: bijux_domain_bam::params::UdgModel::NonUdg,
                    pmd_threshold_5p: 0.3,
                    pmd_threshold_3p: 0.3,
                    trim_5p: 0,
                    trim_3p: 0,
                },
            };
            if let Some(value) = args.udg_model {
                params.udg_model = value.into();
            }
            if let Some(value) = args.pmd_threshold_5p {
                params.pmd_threshold_5p = value;
            }
            if let Some(value) = args.pmd_threshold_3p {
                params.pmd_threshold_3p = value;
            }
            if let Some(value) = args.trim_5p {
                params.trim_5p = value;
            }
            if let Some(value) = args.trim_3p {
                params.trim_3p = value;
            }
            bijux_stages_bam::bam::damage::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Authenticity => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_domain_bam::params::BamEffectiveParams::Authenticity(params) => params,
                _ => bijux_domain_bam::params::AuthenticityEffectiveParams {
                    mode: "aggregate".to_string(),
                },
            };
            if let Some(value) = args.authenticity_mode.clone() {
                params.mode = value;
            }
            bijux_stages_bam::bam::authenticity::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Contamination => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_domain_bam::params::BamEffectiveParams::Contamination(params) => params,
                _ => bijux_domain_bam::params::ContaminationEffectiveParams {
                    reference_panels: Vec::new(),
                    scope: bijux_domain_bam::params::ContaminationScope::Both,
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
            if let Some(value) = args.contamination_scope {
                params.scope = value.into();
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
            bijux_stages_bam::bam::contamination::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Sex => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_domain_bam::params::BamEffectiveParams::Sex(params) => params,
                _ => bijux_domain_bam::params::SexEffectiveParams {
                    expected_sex: None,
                    method: "rxy".to_string(),
                },
            };
            if let Some(value) = args.expected_sex {
                params.expected_sex = Some(value.into());
            }
            if !args.sex_method.is_empty() {
                params.method.clone_from(&args.sex_method);
            }
            bijux_stages_bam::bam::sex::plan(spec, &args.bam, out_dir, &params)
        }
        #[cfg(feature = "bam_downstream")]
        bijux_domain_bam::BamStage::BiasMitigation => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_domain_bam::params::BamEffectiveParams::BiasMitigation(params) => params,
                _ => bijux_domain_bam::params::BiasMitigationEffectiveParams {
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
            bijux_stages_bam::bam::bias_mitigation::plan(spec, &args.bam, out_dir, &params)
        }
        #[cfg(not(feature = "bam_downstream"))]
        bijux_domain_bam::BamStage::BiasMitigation => Err(anyhow!(
            "bam.bias_mitigation is disabled without feature 'bam_downstream'"
        )),
        bijux_domain_bam::BamStage::Recalibration => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_domain_bam::params::BamEffectiveParams::Recalibration(params) => params,
                _ => bijux_domain_bam::params::BqsrEffectiveParams {
                    known_sites: Vec::new(),
                    mode: bijux_domain_bam::params::BqsrMode::Skip,
                    skip_criteria: bijux_domain_bam::params::RecalibrationSkipCriteria {
                        min_mean_coverage: 1.0,
                        min_breadth_1x: 0.1,
                    },
                },
            };
            if !args.known_sites.is_empty() {
                params.known_sites.clone_from(&args.known_sites);
            }
            if let Some(value) = args.bqsr_mode {
                params.mode = value.into();
            }
            if let Some(value) = args.bqsr_min_mean_coverage {
                params.skip_criteria.min_mean_coverage = value;
            }
            if let Some(value) = args.bqsr_min_breadth_1x {
                params.skip_criteria.min_breadth_1x = value;
            }
            bijux_stages_bam::bam::recalibration::plan(spec, &args.bam, out_dir, &params)
        }
        #[cfg(feature = "bam_downstream")]
        bijux_domain_bam::BamStage::Haplogroups => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_domain_bam::params::BamEffectiveParams::Haplogroups(params) => params,
                _ => bijux_domain_bam::params::HaplogroupEffectiveParams {
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
            bijux_stages_bam::bam::haplogroups::plan(spec, &args.bam, out_dir, &params)
        }
        #[cfg(feature = "bam_downstream")]
        bijux_domain_bam::BamStage::Genotyping => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_domain_bam::params::BamEffectiveParams::Genotyping(params) => params,
                _ => bijux_domain_bam::params::GenotypingEffectiveParams {
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
            bijux_stages_bam::bam::genotyping::plan(spec, &args.bam, out_dir, &params)
        }
        #[cfg(feature = "bam_downstream")]
        bijux_domain_bam::BamStage::Kinship => {
            let default_params = default_params_for_stage(profile, stage);
            let mut params = match default_params {
                bijux_domain_bam::params::BamEffectiveParams::Kinship(params) => params,
                _ => bijux_domain_bam::params::KinshipEffectiveParams {
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
            bijux_stages_bam::bam::kinship::plan(spec, &args.bam, out_dir, &params)
        }
        #[cfg(not(feature = "bam_downstream"))]
        bijux_domain_bam::BamStage::Haplogroups
        | bijux_domain_bam::BamStage::Genotyping
        | bijux_domain_bam::BamStage::Kinship => Err(anyhow!(
            "downstream BAM stages are disabled (enable feature 'bam_downstream')"
        )),
    }
}

fn default_params_for_stage(
    profile: &PipelineProfile,
    stage: bijux_domain_bam::BamStage,
) -> bijux_domain_bam::params::BamEffectiveParams {
    profile
        .defaults
        .params
        .get(stage.as_str())
        .and_then(|value| stage.parse_effective_params(value).ok())
        .unwrap_or_else(|| bijux_domain_bam::stage_spec(stage).default_params)
}
