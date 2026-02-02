use bijux_core::ToolRegistry;
use bijux_engine::api::{build_tool_execution_spec, execute_stage_plan};
use bijux_env_runtime::api::RunnerKind;
use bijux_pipelines_bam::{profile_by_id, BamPipelineProfile};

use crate::cli::parse::{BamCommand, BamRunArgs};
// imports provided by entry.rs

#[allow(clippy::missing_errors_doc)]
pub fn handle_bam_commands(
    cli: &Cli,
    registry: &ToolRegistry,
    domain_dir: &Path,
) -> Result<bool> {
    let Commands::Bam { command } = &cli.command else {
        return Ok(false);
    };

    match command {
        BamCommand::ListStages => {
            for stage in bijux_domain_bam::BamStage::all() {
                println!("{}", stage.as_str());
            }
            Ok(true)
        }
        BamCommand::Explain { stage } => {
            let stage_id = stage.stage().as_str();
            let manifest = registry
                .stages()
                .get(stage_id)
                .ok_or_else(|| anyhow!("stage {stage_id} missing from manifests"))?;
            println!("{}", serde_json::to_string_pretty(manifest)?);
            Ok(true)
        }
        BamCommand::Run(args) => {
            run_bam_stage(cli, registry, domain_dir, args)?;
            Ok(true)
        }
    }
}

fn run_bam_stage(
    cli: &Cli,
    registry: &ToolRegistry,
    domain_dir: &Path,
    args: &BamRunArgs,
) -> Result<()> {
    let platform = load_platform(cli.platform.as_deref())
        .map_err(|err| anyhow!("failed to load platform: {err}"))?;
    let catalog = load_image_catalog()
        .map_err(|err| anyhow!("failed to load image catalog: {err}"))?;
    let stage = args.stage.stage();
    let profile = profile_by_id(&args.profile)?;
    let tool_id = args.tool.clone().unwrap_or_else(|| {
        profile
            .default_tool(stage)
            .unwrap_or("samtools")
            .to_string()
    });
    let spec =
        build_tool_execution_spec(stage.as_str(), &tool_id, registry, &catalog, &platform)?;

    let out_dir = args.out.clone();
    std::fs::create_dir_all(&out_dir).context("create bam out dir")?;
    let log_path = out_dir.join("bijux_bam.log");
    let _log_guard = init_logging(&log_path)?;

    let plan = plan_for_bam_stage(stage, &spec, &profile, args, out_dir.as_path())?;
    println!("{}", serde_json::to_string_pretty(&plan)?);
    println!("manifests: {}", domain_dir.display());

    if args.dry_run {
        return Ok(());
    }
    execute_stage_plan(&plan, RunnerKind::Docker, None)?;
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub(crate) fn plan_for_bam_stage(
    stage: bijux_domain_bam::BamStage,
    spec: &bijux_core::ToolExecutionSpecV1,
    profile: &BamPipelineProfile,
    args: &BamRunArgs,
    out_dir: &Path,
) -> Result<bijux_core::StagePlanV1> {
    match stage {
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
            let params = bijux_domain_bam::params::FilterEffectiveParams {
                mapq_threshold: args.min_mapq.unwrap_or(30),
                include_flags: args.include_flags.clone(),
                exclude_flags: args.exclude_flags.clone(),
                min_length: args.min_length.unwrap_or(30),
                remove_duplicates: args.remove_duplicates,
                base_quality_threshold: args.base_quality_threshold.unwrap_or(20),
            };
            bijux_stages_bam::bam::filter::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Markdup => {
            let params = bijux_domain_bam::params::MarkDupEffectiveParams {
                optical_duplicates: args
                    .optical_duplicates
                    .map_or(bijux_domain_bam::params::OpticalDuplicatePolicy::MarkOnly, Into::into),
                umi_policy: args
                    .umi_policy
                    .map_or(bijux_domain_bam::params::UmiPolicy::Ignore, Into::into),
                duplicate_action: args
                    .duplicate_action
                    .map_or(bijux_domain_bam::params::DuplicateAction::Mark, Into::into),
            };
            bijux_stages_bam::bam::markdup::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Complexity => {
            let params = bijux_domain_bam::params::ComplexityEffectiveParams {
                min_reads: args.complexity_min_reads.unwrap_or(100_000),
                projection_points: if args.complexity_projection_points.is_empty() {
                    vec![1_000_000, 2_000_000, 5_000_000]
                } else {
                    args.complexity_projection_points.clone()
                },
            };
            bijux_stages_bam::bam::complexity::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Coverage => {
            let params = bijux_domain_bam::params::CoverageEffectiveParams {
                regions: args.regions.clone().map(bijux_domain_bam::types::BedRegions),
                depth_thresholds: if args.depth_thresholds.is_empty() {
                    vec![1, 3, 5]
                } else {
                    args.depth_thresholds.clone()
                },
            };
            bijux_stages_bam::bam::coverage::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Damage => {
            let params = bijux_domain_bam::params::DamageEffectiveParams {
                udg_model: args
                    .udg_model
                    .map_or(bijux_domain_bam::params::UdgModel::NonUdg, Into::into),
                pmd_threshold_5p: args.pmd_threshold_5p.unwrap_or(0.3),
                pmd_threshold_3p: args.pmd_threshold_3p.unwrap_or(0.3),
                trim_5p: args.trim_5p.unwrap_or(0),
                trim_3p: args.trim_3p.unwrap_or(0),
            };
            bijux_stages_bam::bam::damage::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Authenticity => {
            let params = bijux_domain_bam::params::AuthenticityEffectiveParams {
                mode: args
                    .authenticity_mode
                    .clone()
                    .unwrap_or_else(|| "aggregate".to_string()),
            };
            bijux_stages_bam::bam::authenticity::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Contamination => {
            let params = bijux_domain_bam::params::ContaminationEffectiveParams {
                reference_panels: args.contamination_panel.clone(),
                scope: args
                    .contamination_scope
                    .map_or(bijux_domain_bam::params::ContaminationScope::Both, Into::into),
                prior: args.contamination_prior,
                sex_specific: args.sex_specific_contamination,
                assumptions: args.contamination_assumptions.clone(),
            };
            bijux_stages_bam::bam::contamination::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Sex => {
            let params = bijux_domain_bam::params::SexEffectiveParams {
                expected_sex: args.expected_sex.map(Into::into),
                method: args.sex_method.clone(),
            };
            bijux_stages_bam::bam::sex::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::BiasMitigation => {
            let params = bijux_domain_bam::params::BiasMitigationEffectiveParams {
                gc_bias_correction: args.gc_bias_correction,
                map_bias_correction: args.map_bias_correction,
            };
            bijux_stages_bam::bam::bias_mitigation::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Recalibration => {
            let params = bijux_domain_bam::params::BqsrEffectiveParams {
                known_sites: args.known_sites.clone(),
                mode: args
                    .bqsr_mode
                    .map_or(bijux_domain_bam::params::BqsrMode::Skip, Into::into),
                skip_criteria: bijux_domain_bam::params::RecalibrationSkipCriteria {
                    min_mean_coverage: args.bqsr_min_mean_coverage.unwrap_or(1.0),
                    min_breadth_1x: args.bqsr_min_breadth_1x.unwrap_or(0.1),
                },
            };
            bijux_stages_bam::bam::recalibration::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Haplogroups => {
            let params = bijux_domain_bam::params::HaplogroupEffectiveParams {
                reference_panel: args
                    .haplogroup_panel
                    .clone()
                    .unwrap_or_else(|| "mito_default".to_string()),
                min_coverage: args.haplogroup_min_coverage.or(Some(1.0)),
            };
            bijux_stages_bam::bam::haplogroups::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Genotyping => {
            let params = bijux_domain_bam::params::GenotypingEffectiveParams {
                caller: args
                    .caller
                    .clone()
                    .unwrap_or_else(|| "angsd".to_string()),
                min_posterior: args.min_posterior.or(Some(0.9)),
                min_call_rate: args.min_call_rate.or(Some(0.5)),
            };
            bijux_stages_bam::bam::genotyping::plan(spec, &args.bam, out_dir, &params)
        }
        bijux_domain_bam::BamStage::Kinship => {
            let params = bijux_domain_bam::params::KinshipEffectiveParams {
                reference_panel: args
                    .kinship_panel
                    .clone()
                    .unwrap_or_else(|| "king_default".to_string()),
                min_overlap_snps: args.min_overlap_snps.unwrap_or(1000),
            };
            bijux_stages_bam::bam::kinship::plan(spec, &args.bam, out_dir, &params)
        }
    }
}
