use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_core::{
    build_execution_plan, load_manifests, load_profile, new_run_id, DryRunExecutor, Executor,
    PathSpec, RunSpec,
};
use bijux_environment::api::{load_image_catalog, load_platform};
use clap::Parser;
use tracing::{info, warn};

mod cli;
mod env;
mod replay;
mod utils;

use bijux_engine::api::bench::{
    bench_fastq_correct, bench_fastq_filter, bench_fastq_merge, bench_fastq_preprocess,
    bench_fastq_qc2, bench_fastq_screen, bench_fastq_stats, bench_fastq_trim, bench_fastq_umi,
    bench_fastq_validate, print_bench_schema,
};
use bijux_engine::api::image_qa::run_image_qa;
use bijux_engine::api::init_logging;
use cli::{
    bench_args_correct, bench_args_filter, bench_args_from_trim, bench_args_from_validate,
    bench_args_merge, bench_args_preprocess, bench_args_qc2, bench_args_screen, bench_args_stats,
    bench_args_trim, bench_args_umi, bench_args_validate, is_bench_requested_trim,
    is_bench_requested_validate, BenchCommand, BenchFastqCommand, Cli, Commands, EnvCommand,
    FastqCommand,
};
use env::{env_doctor, print_env_images, print_env_info};
use replay::replay_run;
use utils::normalize_run_base_dir;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().context("resolve current directory")?;
    let domain_dir = cwd.join("domain");

    if handle_meta_commands(&cli, &domain_dir)? {
        return Ok(());
    }

    let profile_path = cwd
        .join("configs")
        .join("profiles")
        .join(format!("{}.yaml", cli.profile));
    let mut profile = load_profile(&profile_path)
        .map_err(|err| anyhow!("failed to load profile {}: {err}", profile_path.display()))?;
    profile.run_base_dir = normalize_run_base_dir(&cwd, &profile.run_base_dir);

    let registry =
        load_manifests(&domain_dir).map_err(|err| anyhow!("manifest validation failed: {err}"))?;

    if handle_fastq_bench(&cli, &registry, &domain_dir)? {
        return Ok(());
    }

    run_plan(&cli, &registry, &domain_dir)
}

#[allow(clippy::too_many_lines)]
fn handle_meta_commands(cli: &Cli, domain_dir: &Path) -> Result<bool> {
    match &cli.command {
        Commands::ValidateManifests => {
            let registry = load_manifests(domain_dir)
                .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
            println!(
                "validated {} stages and {} tools",
                registry.stages().len(),
                registry
                    .stages()
                    .keys()
                    .map(|stage| registry.tools_for_stage(stage).len())
                    .sum::<usize>()
            );
            Ok(true)
        }
        Commands::Platform => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            println!("{}", serde_json::to_string_pretty(&platform)?);
            Ok(true)
        }
        Commands::ImageQa => {
            run_image_qa(cli.platform.as_deref())?;
            Ok(true)
        }
        Commands::Replay(args) => {
            replay_run(&args.run_id, &args.search_root)?;
            Ok(true)
        }
        Commands::Compare(args) => {
            let result = bijux_engine::api::compare::compare_runs(
                &args.run_a,
                &args.run_b,
                &args.search_root,
            )?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(true)
        }
        Commands::Env { command } => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            match command {
                EnvCommand::Images => {
                    print_env_images(&catalog, &platform)?;
                }
                EnvCommand::Info => {
                    print_env_info(&catalog, &platform);
                }
                EnvCommand::Doctor => {
                    env_doctor(&catalog, &platform);
                }
            }
            Ok(true)
        }
        Commands::Bench { command } => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            match command {
                BenchCommand::Fastq { command } => match command {
                    BenchFastqCommand::Trim(args) => {
                        bench_fastq_trim(&catalog, &platform, None, &bench_args_trim(args))?;
                    }
                    BenchFastqCommand::Validate(args) => {
                        bench_fastq_validate(
                            &catalog,
                            &platform,
                            None,
                            &bench_args_validate(args),
                        )?;
                    }
                    BenchFastqCommand::Filter(args) => {
                        bench_fastq_filter(&catalog, &platform, None, &bench_args_filter(args))?;
                    }
                    BenchFastqCommand::Merge(args) => {
                        bench_fastq_merge(&catalog, &platform, None, &bench_args_merge(args))?;
                    }
                    BenchFastqCommand::Stats(args) => {
                        bench_fastq_stats(&catalog, &platform, None, &bench_args_stats(args))?;
                    }
                    BenchFastqCommand::Correct(args) => {
                        bench_fastq_correct(&catalog, &platform, None, &bench_args_correct(args))?;
                    }
                    BenchFastqCommand::Qc2(args) => {
                        bench_fastq_qc2(&catalog, &platform, None, &bench_args_qc2(args))?;
                    }
                    BenchFastqCommand::Umi(args) => {
                        bench_fastq_umi(&catalog, &platform, None, &bench_args_umi(args))?;
                    }
                    BenchFastqCommand::Screen(args) => {
                        bench_fastq_screen(&catalog, &platform, None, &bench_args_screen(args))?;
                    }
                    BenchFastqCommand::Preprocess(args) => {
                        bench_fastq_preprocess(
                            &catalog,
                            &platform,
                            None,
                            &bench_args_preprocess(args),
                        )?;
                    }
                },
                BenchCommand::Schema { stage } => {
                    print_bench_schema(stage)?;
                }
            }
            Ok(true)
        }
        Commands::Fastq { .. } => Ok(false),
    }
}

fn handle_fastq_bench(
    cli: &Cli,
    registry: &bijux_core::ToolRegistry,
    _domain_dir: &Path,
) -> Result<bool> {
    let Commands::Fastq { command } = &cli.command else {
        return Ok(false);
    };

    match command {
        FastqCommand::Trim(args) if is_bench_requested_trim(args) => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            let runner = cli::parse_runner_override(args.env.as_deref())?;
            let bench_args = bench_args_from_trim(args)?;
            bench_fastq_trim(&catalog, &platform, runner, &bench_args)?;
            Ok(true)
        }
        FastqCommand::Validate(args) if is_bench_requested_validate(args) => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            let runner = cli::parse_runner_override(args.env.as_deref())?;
            let bench_args = bench_args_from_validate(args)?;
            bench_fastq_validate(&catalog, &platform, runner, &bench_args)?;
            Ok(true)
        }
        _ => {
            let (stage, _tool, common) = cli::resolve_stage_tool(&cli.command);
            if common.list_tools {
                let mut tool_ids: Vec<_> = registry
                    .tools_for_stage(&stage.0)
                    .into_iter()
                    .map(|tool| tool.tool_id.clone())
                    .collect();
                tool_ids.sort();
                for tool_id in tool_ids {
                    println!("{tool_id}");
                }
                return Ok(true);
            }
            Ok(false)
        }
    }
}

fn run_plan(cli: &Cli, registry: &bijux_core::ToolRegistry, domain_dir: &Path) -> Result<()> {
    let (stage, tool, common) = cli::resolve_stage_tool(&cli.command);

    let run_id = new_run_id();
    let run_spec = RunSpec {
        stage: stage.clone(),
        tool: tool.clone(),
        paths: PathSpec {
            input: Vec::new(),
            output: Vec::new(),
            work: PathBuf::new(),
        },
        params: BTreeMap::new(),
    };

    let mut profile = load_profile_for_cli(cli)?;
    ensure_profile_run_base_dir(&stage, &tool, &mut profile);
    let plan = build_execution_plan(run_spec, registry, profile, run_id.clone())
        .map_err(|err| anyhow!("failed to build plan: {err}"))?;

    std::fs::create_dir_all(&plan.logs_dir).context("create logs directory")?;
    std::fs::create_dir_all(&plan.artifacts_dir).context("create artifacts directory")?;
    let log_path = plan.logs_dir.join("bijux.log");
    let _log_guard = init_logging(&log_path)?;

    println!("{}", serde_json::to_string_pretty(&plan)?);
    println!("manifests: {}", domain_dir.display());

    if !common.dry_run {
        warn!(
            run_id = %plan.run_id.0,
            stage = %plan.stage.stage_id,
            tool = %plan.tool.tool_id,
            "no executor implemented yet, falling back to dry-run"
        );
    }

    let executor = DryRunExecutor;
    executor.run(&plan)?;
    info!(
        run_id = %plan.run_id.0,
        stage = %plan.stage.stage_id,
        tool = %plan.tool.tool_id,
        "report written"
    );

    Ok(())
}

fn load_profile_for_cli(cli: &Cli) -> Result<bijux_core::Profile> {
    let cwd = std::env::current_dir().context("resolve current directory")?;
    let profile_path = cwd
        .join("configs")
        .join("profiles")
        .join(format!("{}.yaml", cli.profile));
    let mut profile = load_profile(&profile_path)
        .map_err(|err| anyhow!("failed to load profile {}: {err}", profile_path.display()))?;
    profile.run_base_dir = normalize_run_base_dir(&cwd, &profile.run_base_dir);
    Ok(profile)
}

fn ensure_profile_run_base_dir(
    stage: &bijux_core::StageId,
    tool: &bijux_core::ToolId,
    profile: &mut bijux_core::Profile,
) {
    let run_dir = bijux_core::run_dir(&profile.run_base_dir, &new_run_id(), stage, tool);
    if run_dir.starts_with(profile.run_base_dir.join("runs")) {
        let base = profile
            .run_base_dir
            .parent()
            .unwrap_or(&profile.run_base_dir);
        profile.run_base_dir = base.to_path_buf();
    }
}
