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

use bijux_domain_fastq::{
    bench_fastq_correct, bench_fastq_filter, bench_fastq_merge, bench_fastq_preprocess,
    bench_fastq_qc_post, bench_fastq_screen, bench_fastq_stats_neutral, bench_fastq_trim,
    bench_fastq_umi, bench_fastq_validate_pre, benchmark_runs, print_bench_schema,
    write_benchmark_exports, write_correct_report, write_filter_report, write_merge_report,
    write_qc_post_report, write_stats_report, write_trim_report, write_umi_report,
    write_validate_report,
};
use bijux_engine::api::init_logging;
use bijux_environment::image_qa::run_image_qa;
use cli::{
    bench_args_correct, bench_args_filter, bench_args_from_trim, bench_args_from_validate,
    bench_args_merge, bench_args_preprocess, bench_args_qc_post, bench_args_screen,
    bench_args_stats, bench_args_trim, bench_args_umi, bench_args_validate,
    is_bench_requested_trim, is_bench_requested_validate, preprocess_args_from_cli, BenchCommand,
    BenchFastqCommand, Cli, Commands, EnvCommand, FastqCommand,
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
            let result =
                bijux_bench::compare::compare_runs(&args.run_a, &args.run_b, &args.search_root)?;
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
                        let outcome =
                            bench_fastq_trim(&catalog, &platform, None, &bench_args_trim(args))?;
                        write_trim_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::Validate(args) => {
                        let outcome = bench_fastq_validate_pre(
                            &catalog,
                            &platform,
                            None,
                            &bench_args_validate(args),
                        )?;
                        write_validate_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::Filter(args) => {
                        let outcome = bench_fastq_filter(
                            &catalog,
                            &platform,
                            None,
                            &bench_args_filter(args),
                        )?;
                        write_filter_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::Merge(args) => {
                        let outcome =
                            bench_fastq_merge(&catalog, &platform, None, &bench_args_merge(args))?;
                        write_merge_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::Stats(args) => {
                        let outcome = bench_fastq_stats_neutral(
                            &catalog,
                            &platform,
                            None,
                            &bench_args_stats(args),
                        )?;
                        write_stats_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::Correct(args) => {
                        let outcome = bench_fastq_correct(
                            &catalog,
                            &platform,
                            None,
                            &bench_args_correct(args),
                        )?;
                        write_correct_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::QcPost(args) => {
                        let outcome = bench_fastq_qc_post(
                            &catalog,
                            &platform,
                            None,
                            &bench_args_qc_post(args),
                        )?;
                        write_qc_post_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::Umi(args) => {
                        let outcome =
                            bench_fastq_umi(&catalog, &platform, None, &bench_args_umi(args))?;
                        write_umi_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
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

    if let Some(done) = handle_fastq_discovery(command, registry)? {
        return Ok(done);
    }

    match command {
        FastqCommand::Trim(args) if is_bench_requested_trim(args) => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            let runner = cli::parse_runner_override(args.env.as_deref())?;
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
        FastqCommand::ValidatePre(args) if is_bench_requested_validate(args) => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            let runner = cli::parse_runner_override(args.env.as_deref())?;
            let bench_args = bench_args_from_validate(args)?;
            let outcome = bench_fastq_validate_pre(&catalog, &platform, runner, &bench_args)?;
            write_validate_report(
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
        FastqCommand::Preprocess(args) => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            let runner = cli::parse_runner_override(args.env.as_deref())?;
            let bench_args = preprocess_args_from_cli(args)?;
            bench_fastq_preprocess(&catalog, &platform, runner, &bench_args)?;
            Ok(true)
        }
        FastqCommand::Run(args) => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            let runner = cli::parse_runner_override(args.args.env.as_deref())?;
            let bench_args = preprocess_args_from_cli(&args.args)?;
            bench_fastq_preprocess(&catalog, &platform, runner, &bench_args)?;
            Ok(true)
        }
        FastqCommand::Benchmark(args) => {
            let stage_id = normalize_fastq_stage_id(&args.stage);
            let summary = benchmark_runs(&args.runs, &stage_id, args.objective.into())?;
            let (json_path, csv_path) = write_benchmark_exports(&args.runs, &summary)?;
            println!("{}", serde_json::to_string_pretty(&summary)?);
            println!("benchmark_json: {}", json_path.display());
            println!("benchmark_csv: {}", csv_path.display());
            Ok(true)
        }
        FastqCommand::Analyze(args) => {
            let stage_id = normalize_fastq_stage_id(&args.stage);
            let summary = benchmark_runs(&args.runs, &stage_id, args.objective.into())?;
            let (json_path, csv_path) = write_benchmark_exports(&args.runs, &summary)?;
            println!("{}", serde_json::to_string_pretty(&summary)?);
            println!("benchmark_json: {}", json_path.display());
            println!("benchmark_csv: {}", csv_path.display());
            Ok(true)
        }
        FastqCommand::Compare(args) => {
            let result =
                bijux_bench::compare::compare_runs(&args.run_a, &args.run_b, &args.search_root)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(true)
        }
        _ => {
            let (stage, _tool, common) = cli::resolve_stage_tool(&cli.command);
            if common.list_tools {
                list_fastq_tools(registry, &stage.0);
                return Ok(true);
            }
            Ok(false)
        }
    }
}

fn handle_fastq_discovery(
    command: &FastqCommand,
    registry: &bijux_core::ToolRegistry,
) -> Result<Option<bool>> {
    match command {
        FastqCommand::ListStages => {
            list_fastq_stages(registry);
            Ok(Some(true))
        }
        FastqCommand::ListTools { stage } => {
            let stage_id = normalize_fastq_stage_id(stage);
            list_fastq_tools(registry, &stage_id);
            Ok(Some(true))
        }
        FastqCommand::Explain { stage } => {
            let stage_id = normalize_fastq_stage_id(stage);
            explain_fastq_stage(registry, &stage_id)?;
            Ok(Some(true))
        }
        _ => Ok(None),
    }
}

fn list_fastq_stages(registry: &bijux_core::ToolRegistry) {
    let mut stage_ids: Vec<_> = registry
        .stages()
        .values()
        .filter(|stage| stage.domain == "fastq")
        .map(|stage| stage.stage_id.clone())
        .collect();
    stage_ids.sort();
    for stage_id in stage_ids {
        println!("{stage_id}");
    }
}

fn list_fastq_tools(registry: &bijux_core::ToolRegistry, stage_id: &str) {
    let mut tool_ids: Vec<_> = registry
        .tools_for_stage(stage_id)
        .into_iter()
        .map(|tool| tool.tool_id.clone())
        .collect();
    tool_ids.sort();
    for tool_id in tool_ids {
        println!("{tool_id}");
    }
}

fn explain_fastq_stage(registry: &bijux_core::ToolRegistry, stage_id: &str) -> Result<()> {
    if stage_id == "fastq.preprocess" {
        let args = bijux_domain_fastq::BenchFastqPreprocessArgs {
            sample_id: "explain".to_string(),
            r1: PathBuf::from("reads.fastq.gz"),
            r2: None,
            out: PathBuf::from("artifacts"),
            strict: false,
            auto: false,
            objective: bijux_analyze::selection::Objective::Balanced,
            bench_corpus: None,
            allow_partial: false,
        };
        let plan = bijux_domain_fastq::fastq_preprocess_plan(&args);
        println!("stage: {stage_id}");
        println!("pipeline:");
        for step in plan.stages {
            println!("- {step}");
        }
        return Ok(());
    }
    let stage = registry
        .stages()
        .get(stage_id)
        .ok_or_else(|| anyhow!("unknown stage {stage_id}"))?;
    println!("stage: {}", stage.stage_id);
    if !stage.description.is_empty() {
        println!("description: {}", stage.description);
    }
    println!("inputs:");
    for input in &stage.inputs {
        println!("- {} ({})", input.name, input.data_type);
    }
    println!("outputs:");
    for output in &stage.outputs {
        println!("- {} ({})", output.name, output.data_type);
    }
    Ok(())
}

fn normalize_fastq_stage_id(stage: &str) -> String {
    if stage.contains('.') {
        stage.to_string()
    } else {
        format!("fastq.{stage}")
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
