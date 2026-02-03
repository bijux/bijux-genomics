use bijux_core::ToolRegistry;
use bijux_engine::api::{build_tool_execution_spec, execute_stage_plan};
use bijux_env_runtime::api::RunnerKind;
use bijux_pipelines::registry;
use bijux_pipelines::Domain;

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
    let catalog =
        load_image_catalog().map_err(|err| anyhow!("failed to load image catalog: {err}"))?;
    let stage = args.stage.stage();
    let profile = registry::profile_by_id(Domain::Bam, &args.profile)?;
    let tool_id = args.tool.clone().unwrap_or_else(|| {
        profile
            .defaults
            .tools
            .get(stage.as_str())
            .cloned()
            .unwrap_or_else(|| "samtools".to_string())
    });
    let spec =
        build_tool_execution_spec(stage.as_str(), &tool_id, registry, &catalog, &platform)?;

    let out_dir = args.out.clone();
    std::fs::create_dir_all(&out_dir).context("create bam out dir")?;
    let log_path = out_dir.join("bijux_bam.log");
    let _log_guard = init_logging(&log_path)?;

    let plan = bijux_api::bam_plan::plan_for_bam_stage_with_profile(
        stage,
        &spec,
        &bam_run_args_to_api(args),
        &profile,
        out_dir.as_path(),
    )?;
    println!("{}", serde_json::to_string_pretty(&plan)?);
    println!("manifests: {}", domain_dir.display());

    if args.dry_run {
        return Ok(());
    }
    execute_stage_plan(&plan, RunnerKind::Docker, None)?;
    Ok(())
}

fn bam_run_args_to_api(args: &BamRunArgs) -> bijux_api::BamRunArgs {
    bijux_api::BamRunArgs {
        stage: args.stage.stage(),
        profile: args.profile.clone(),
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        bam: args.bam.clone(),
        out: args.out.clone(),
        tool: args.tool.clone(),
        bai: args.bai.clone(),
        reference: args.reference.clone(),
        regions: args.regions.as_ref().map(|path| path.display().to_string()),
        udg_model: args.udg_model.map(udg_model_to_string),
        pmd_threshold_5p: args.pmd_threshold_5p,
        pmd_threshold_3p: args.pmd_threshold_3p,
        trim_5p: args.trim_5p.map(u32::from),
        trim_3p: args.trim_3p.map(u32::from),
        contamination_scope: args.contamination_scope.map(contamination_scope_to_string),
        contamination_panel: args.contamination_panel.clone(),
        contamination_prior: args.contamination_prior,
        sex_specific_contamination: args.sex_specific_contamination,
        contamination_assumptions: args.contamination_assumptions.clone(),
        expected_sex: args.expected_sex.map(expected_sex_to_string),
        sex_method: args.sex_method.clone(),
        min_mapq: args.min_mapq.map(u32::from),
        min_length: args.min_length,
        include_flags: args.include_flags.iter().map(ToString::to_string).collect(),
        exclude_flags: args.exclude_flags.iter().map(ToString::to_string).collect(),
        remove_duplicates: args.remove_duplicates,
        base_quality_threshold: args.base_quality_threshold,
        optical_duplicates: args.optical_duplicates.map(optical_duplicates_to_string),
        umi_policy: args.umi_policy.map(umi_policy_to_string),
        duplicate_action: args.duplicate_action.map(duplicate_action_to_string),
        complexity_min_reads: args
            .complexity_min_reads
            .map(|value| value.try_into().unwrap_or(u32::MAX)),
        complexity_projection_points: args.complexity_projection_points.clone(),
        depth_thresholds: args.depth_thresholds.clone(),
        bqsr_mode: args.bqsr_mode.map(bqsr_mode_to_string),
        known_sites: args.known_sites.clone(),
        bqsr_min_mean_coverage: args.bqsr_min_mean_coverage,
        bqsr_min_breadth_1x: args.bqsr_min_breadth_1x,
        haplogroup_panel: args.haplogroup_panel.clone(),
        haplogroup_min_coverage: args.haplogroup_min_coverage,
        kinship_panel: args.kinship_panel.clone(),
        min_overlap_snps: args.min_overlap_snps,
        caller: args.caller.clone(),
        min_posterior: args.min_posterior,
        min_call_rate: args.min_call_rate,
        gc_bias_correction: args.gc_bias_correction,
        map_bias_correction: args.map_bias_correction,
        authenticity_mode: args.authenticity_mode.clone(),
        aligner_preset: args.aligner_preset.clone(),
        rg_id: args.rg_id.clone(),
        rg_sm: args.rg_sm.clone(),
        rg_pl: args.rg_pl.clone(),
        rg_lb: args.rg_lb.clone(),
        rg_policy: args.rg_policy.map(read_group_policy_to_string),
        build_reference_indices: args.build_reference_indices,
        params_json: args.params_json.as_ref().map(|path| path.display().to_string()),
        dry_run: args.dry_run,
    }
}

fn udg_model_to_string(value: crate::cli::parse::UdgModelArg) -> String {
    match value {
        crate::cli::parse::UdgModelArg::NonUdg => "non_udg",
        crate::cli::parse::UdgModelArg::HalfUdg => "half_udg",
        crate::cli::parse::UdgModelArg::Udg => "udg",
    }
    .to_string()
}

fn contamination_scope_to_string(value: crate::cli::parse::ContaminationScopeArg) -> String {
    match value {
        crate::cli::parse::ContaminationScopeArg::Mito => "mito",
        crate::cli::parse::ContaminationScopeArg::Nuclear => "nuclear",
        crate::cli::parse::ContaminationScopeArg::Both => "both",
    }
    .to_string()
}

fn expected_sex_to_string(value: crate::cli::parse::ExpectedSexArg) -> String {
    match value {
        crate::cli::parse::ExpectedSexArg::Xx => "xx",
        crate::cli::parse::ExpectedSexArg::Xy => "xy",
        crate::cli::parse::ExpectedSexArg::Unknown => "unknown",
    }
    .to_string()
}

fn optical_duplicates_to_string(
    value: crate::cli::parse::OpticalDuplicatePolicyArg,
) -> String {
    match value {
        crate::cli::parse::OpticalDuplicatePolicyArg::None => "none",
        crate::cli::parse::OpticalDuplicatePolicyArg::MarkOnly => "mark_only",
        crate::cli::parse::OpticalDuplicatePolicyArg::Remove => "remove",
    }
    .to_string()
}

fn umi_policy_to_string(value: crate::cli::parse::UmiPolicyArg) -> String {
    match value {
        crate::cli::parse::UmiPolicyArg::Ignore => "ignore",
        crate::cli::parse::UmiPolicyArg::UseTag => "use_tag",
        crate::cli::parse::UmiPolicyArg::Collapse => "collapse",
    }
    .to_string()
}

fn duplicate_action_to_string(value: crate::cli::parse::DuplicateActionArg) -> String {
    match value {
        crate::cli::parse::DuplicateActionArg::Mark => "mark",
        crate::cli::parse::DuplicateActionArg::Remove => "remove",
    }
    .to_string()
}

fn bqsr_mode_to_string(value: crate::cli::parse::BqsrModeArg) -> String {
    match value {
        crate::cli::parse::BqsrModeArg::Standard => "standard",
        crate::cli::parse::BqsrModeArg::Skip => "skip",
        crate::cli::parse::BqsrModeArg::EmitOnly => "emit_only",
    }
    .to_string()
}

fn read_group_policy_to_string(value: crate::cli::parse::ReadGroupPolicyArg) -> String {
    match value {
        crate::cli::parse::ReadGroupPolicyArg::Preserve => "preserve",
        crate::cli::parse::ReadGroupPolicyArg::Merge => "merge",
        crate::cli::parse::ReadGroupPolicyArg::Regenerate => "regenerate",
    }
    .to_string()
}
