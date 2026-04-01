#![allow(clippy::too_many_lines)]

use crate::commands::support::prelude::{
    anyhow, bench_args_from_trim, bench_args_from_validate, bench_fastq_preprocess,
    bench_fastq_trim, bench_fastq_validate_reads, cli, compare_runs, compare_runs_with_baseline,
    env_doctor, fastq_cross_args_from_cli, is_bench_requested_trim, is_bench_requested_validate,
    load_image_catalog, load_platform, normalize_fastq_stage_id, objective_spec,
    preprocess_args_from_cli, qc_class_label, render, resolve_adapter_selection,
    resolve_effective_adapters, write_trim_report, write_validate_report, AdapterPresetsV1,
    AdapterSelection, Cli, DnaCommand, FastqCommand, Objective, Path, Result, StageId,
};

mod adapter_discovery;
mod discovery;
mod explain;
mod tool_policy;

pub(crate) use self::tool_policy::set_tool_tier_policy;

pub(crate) fn handle_fastq_bench(
    cli: &Cli,
    dna_command: &DnaCommand,
    registry: &bijux_dna_api::v1::api::run::ToolRegistry,
) -> Result<bool> {
    let DnaCommand::Fastq(args) = dna_command else {
        return Ok(false);
    };
    let command = &args.command;

    let (allow_silver, allow_experimental) = tool_policy::tool_tier_policy_for_fastq(command);
    tool_policy::set_tool_tier_policy(allow_silver, allow_experimental);

    if let Some(done) = discovery::handle_fastq_discovery(command, registry)? {
        return Ok(done);
    }

    match command {
        FastqCommand::Doctor => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            env_doctor(&catalog, &platform);
            Ok(true)
        }
        FastqCommand::Trim(args) if is_bench_requested_trim(args) => {
            tool_policy::set_tool_tier_policy(
                args.common.allow_silver,
                args.common.allow_experimental,
            );
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            let runner = None;
            let bench_args = bench_args_from_trim(args)?;
            let outcome = bench_fastq_trim(&catalog, &platform, runner, &bench_args)?;
            write_trim_report(
                &outcome.bench_dir,
                &outcome.records,
                &outcome.failures,
                outcome.explain,
            )?;
            if !outcome.failures.is_empty() {
                return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
            }
            Ok(true)
        }
        FastqCommand::ValidateReads(args) if is_bench_requested_validate(args) => {
            tool_policy::set_tool_tier_policy(
                args.common.allow_silver,
                args.common.allow_experimental,
            );
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            let runner = None;
            let bench_args = bench_args_from_validate(args)?;
            let outcome = bench_fastq_validate_reads(&catalog, &platform, runner, &bench_args)?;
            let qc_class = qc_class_label("fastq.validate_reads");
            write_validate_report(
                &outcome.bench_dir,
                &outcome.records,
                &outcome.failures,
                qc_class,
                outcome.explain,
            )?;
            if !outcome.failures.is_empty() {
                return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
            }
            Ok(true)
        }
        FastqCommand::Preprocess(args) => {
            tool_policy::set_tool_tier_policy(
                args.common.allow_silver,
                args.common.allow_experimental,
            );
            tool_policy::set_scientific_preset(args.scientific_preset);
            if let Some(profile_id) = args.pipeline_profile.as_ref() {
                if let Ok(profile) = bijux_dna_api::v1::api::plan::select_pipeline(
                    bijux_dna_api::v1::api::plan::Domain::Cross,
                    profile_id,
                ) {
                    let platform = load_platform(cli.platform.as_deref())
                        .map_err(|err| anyhow!("failed to load platform: {err}"))?;
                    let catalog = load_image_catalog()
                        .map_err(|err| anyhow!("failed to load images: {err}"))?;
                    let runner = None;
                    let bench_args = preprocess_args_from_cli(args)?;
                    let cross_args = fastq_cross_args_from_cli(args);
                    bijux_dna_api::v1::api::run::run_fastq_to_bam_profile(
                        registry,
                        &catalog,
                        &platform,
                        runner,
                        &bench_args,
                        &cross_args,
                        &profile,
                    )?;
                    return Ok(true);
                }
            }
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            let runner = None;
            let bench_args = preprocess_args_from_cli(args)?;
            bench_fastq_preprocess(&catalog, &platform, runner, &bench_args)?;
            Ok(true)
        }
        FastqCommand::Run(args) => {
            tool_policy::set_tool_tier_policy(
                args.args.common.allow_silver,
                args.args.common.allow_experimental,
            );
            tool_policy::set_scientific_preset(args.args.scientific_preset);
            if let Some(profile_id) = args.args.pipeline_profile.as_ref() {
                if let Ok(profile) = bijux_dna_api::v1::api::plan::select_pipeline(
                    bijux_dna_api::v1::api::plan::Domain::Cross,
                    profile_id,
                ) {
                    let platform = load_platform(cli.platform.as_deref())
                        .map_err(|err| anyhow!("failed to load platform: {err}"))?;
                    let catalog = load_image_catalog()
                        .map_err(|err| anyhow!("failed to load images: {err}"))?;
                    let runner = None;
                    let bench_args = preprocess_args_from_cli(&args.args)?;
                    let cross_args = fastq_cross_args_from_cli(&args.args);
                    bijux_dna_api::v1::api::run::run_fastq_to_bam_profile(
                        registry,
                        &catalog,
                        &platform,
                        runner,
                        &bench_args,
                        &cross_args,
                        &profile,
                    )?;
                    return Ok(true);
                }
            }
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            let runner = None;
            let bench_args = preprocess_args_from_cli(&args.args)?;
            bench_fastq_preprocess(&catalog, &platform, runner, &bench_args)?;
            Ok(true)
        }
        FastqCommand::Compare(args) => {
            let objective = objective_spec(Objective::Balanced);
            let run_a = args.search_root.join(&args.run_a);
            let run_b = args.search_root.join(&args.run_b);
            let result = if let Some(baseline) = args.baseline.as_ref() {
                let baseline_dir = args.search_root.join(baseline);
                compare_runs_with_baseline(&run_a, &run_b, &baseline_dir, &objective)?
            } else {
                compare_runs(&run_a, &run_b, &objective)?
            };
            render::json::print_pretty(&result)?;
            Ok(true)
        }
        _ => {
            let (stage, _tool, common) = cli::resolve_stage_tool(dna_command);
            if common.list_tools {
                discovery::list_fastq_tools(registry, &stage.0);
                return Ok(true);
            }
            Ok(false)
        }
    }
}
