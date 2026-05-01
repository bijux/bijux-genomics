use crate::commands::cli::parse::{BamCommand, BamRunArgs};
use crate::commands::support::prelude::{
    anyhow, init_logging, load_image_catalog, load_platform, render, Cli, Context, DnaCommand,
    Path, Result, StageId, ToolId,
};
use bijux_dna_api::v1::api::plan::Domain;
use bijux_dna_api::v1::api::run::ExecuteRunRequest;
use bijux_dna_api::v1::api::run::RuntimeKind;
use bijux_dna_api::v1::api::run::ToolRegistry;
use bijux_dna_api::v1::api::run::{build_tool_execution_spec, execute_run};

/// Handle top-level BAM command dispatch.
///
/// # Errors
/// Returns an error when manifest lookup, plan building, or execution fails.
pub fn handle_bam_commands(
    cli: &Cli,
    dna_command: &DnaCommand,
    registry: &ToolRegistry,
    domain_dir: &Path,
) -> Result<bool> {
    let DnaCommand::Bam(args) = dna_command else {
        return Ok(false);
    };
    let command = &args.command;

    match command {
        BamCommand::ListStages => {
            for stage in bijux_dna_api::v1::api::bench::BamStage::all() {
                println!("{}", stage.as_str());
            }
            Ok(true)
        }
        BamCommand::Explain { stage } => {
            let stage_id_raw = stage.stage().as_str();
            let stage_id =
                StageId::try_from(stage_id_raw).unwrap_or_else(|_| StageId::new(stage_id_raw));
            let manifest = registry
                .stages()
                .get(&stage_id)
                .ok_or_else(|| anyhow!("stage {stage_id_raw} missing from manifests"))?;
            render::json::print_pretty(manifest)?;
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
    let profile = bijux_dna_api::v1::api::plan::select_pipeline(Domain::Bam, &args.profile)?;
    let stage_key = StageId::from_static(stage.as_str());
    let tool_id = args.tool.clone().map_or_else(
        || {
            profile
                .defaults
                .tools
                .get(&stage_key)
                .cloned()
                .unwrap_or_else(|| ToolId::from_static("samtools"))
        },
        ToolId::new,
    );
    let spec =
        build_tool_execution_spec(stage.as_str(), tool_id.as_str(), registry, &catalog, &platform)?;

    let out_dir = args.out.clone();
    bijux_dna_api::v1::api::run::ensure_dir(&out_dir).context("create bam out dir")?;
    let log_path = out_dir.join("bijux_bam.log");
    let _log_guard = init_logging(&log_path)?;

    let plan = bijux_dna_api::v1::api::plan::plan_for_bam_stage_with_profile(
        stage,
        &spec,
        &bam_run_args_to_api(args),
        &profile,
        out_dir.as_path(),
    )?;
    render::json::print_pretty(&plan)?;
    println!("manifests: {}", domain_dir.display());

    if args.dry_run {
        return Ok(());
    }
    execute_run(&ExecuteRunRequest { plan, runner: RuntimeKind::Docker })?;
    Ok(())
}

fn bam_run_args_to_api(args: &BamRunArgs) -> bijux_dna_api::v1::api::bench::BamRunArgs {
    bijux_dna_api::v1::api::bench::BamRunArgs {
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
        alignment_sensitivity_profile: None,
        alignment_seed_length: None,
        rg_id: args.rg_id.clone(),
        rg_sm: args.rg_sm.clone(),
        rg_pl: args.rg_pl.clone(),
        rg_lb: args.rg_lb.clone(),
        rg_pu: None,
        lane_id: None,
        run_id: None,
        subject_id: None,
        cohort_id: None,
        rg_policy: args.rg_policy.map(read_group_policy_to_string),
        build_reference_indices: args.build_reference_indices,
        params_json: args.params_json.as_ref().map(|path| path.display().to_string()),
        dry_run: args.dry_run,
        allow_planned: args.allow_planned,
    }
}

fn udg_model_to_string(value: crate::commands::cli::parse::UdgModelArg) -> String {
    match value {
        crate::commands::cli::parse::UdgModelArg::NonUdg => "non_udg",
        crate::commands::cli::parse::UdgModelArg::HalfUdg => "half_udg",
        crate::commands::cli::parse::UdgModelArg::Udg => "udg",
    }
    .to_string()
}

fn contamination_scope_to_string(
    value: crate::commands::cli::parse::ContaminationScopeArg,
) -> String {
    match value {
        crate::commands::cli::parse::ContaminationScopeArg::Mito => "mito",
        crate::commands::cli::parse::ContaminationScopeArg::Nuclear => "nuclear",
        crate::commands::cli::parse::ContaminationScopeArg::Both => "both",
    }
    .to_string()
}

fn expected_sex_to_string(value: crate::commands::cli::parse::ExpectedSexArg) -> String {
    match value {
        crate::commands::cli::parse::ExpectedSexArg::Xx => "xx",
        crate::commands::cli::parse::ExpectedSexArg::Xy => "xy",
        crate::commands::cli::parse::ExpectedSexArg::Unknown => "unknown",
    }
    .to_string()
}

fn optical_duplicates_to_string(
    value: crate::commands::cli::parse::OpticalDuplicatePolicyArg,
) -> String {
    match value {
        crate::commands::cli::parse::OpticalDuplicatePolicyArg::None => "none",
        crate::commands::cli::parse::OpticalDuplicatePolicyArg::MarkOnly => "mark_only",
        crate::commands::cli::parse::OpticalDuplicatePolicyArg::Remove => "remove",
    }
    .to_string()
}

fn umi_policy_to_string(value: crate::commands::cli::parse::UmiPolicyArg) -> String {
    match value {
        crate::commands::cli::parse::UmiPolicyArg::Ignore => "ignore",
        crate::commands::cli::parse::UmiPolicyArg::UseTag => "use_tag",
        crate::commands::cli::parse::UmiPolicyArg::Collapse => "collapse",
    }
    .to_string()
}

fn duplicate_action_to_string(value: crate::commands::cli::parse::DuplicateActionArg) -> String {
    match value {
        crate::commands::cli::parse::DuplicateActionArg::Mark => "mark",
        crate::commands::cli::parse::DuplicateActionArg::Remove => "remove",
    }
    .to_string()
}

fn bqsr_mode_to_string(value: crate::commands::cli::parse::BqsrModeArg) -> String {
    match value {
        crate::commands::cli::parse::BqsrModeArg::Standard => "standard",
        crate::commands::cli::parse::BqsrModeArg::Skip => "skip",
        crate::commands::cli::parse::BqsrModeArg::EmitOnly => "emit_only",
    }
    .to_string()
}

fn read_group_policy_to_string(value: crate::commands::cli::parse::ReadGroupPolicyArg) -> String {
    match value {
        crate::commands::cli::parse::ReadGroupPolicyArg::Preserve => "preserve",
        crate::commands::cli::parse::ReadGroupPolicyArg::Merge => "merge",
        crate::commands::cli::parse::ReadGroupPolicyArg::Regenerate => "regenerate",
    }
    .to_string()
}
