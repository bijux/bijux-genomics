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

mod adapter_bank;
mod cli;
mod contaminant_bank;
mod env;
mod fastq_exec;
mod fastq_router;
mod fastq_stats_neutral;
mod polyx_bank;
mod replay;
mod utils;

use crate::adapter_bank::{
    resolve_adapter_selection, resolve_effective_adapters, AdapterSelection,
};
use crate::fastq_exec::{
    bench_fastq_correct, bench_fastq_filter, bench_fastq_merge, bench_fastq_preprocess,
    bench_fastq_qc_post, bench_fastq_screen, bench_fastq_stats_neutral, bench_fastq_trim,
    bench_fastq_umi, bench_fastq_validate_pre,
};
use bijux_analyze::{
    compare_runs, print_bench_schema, write_correct_report, write_filter_report,
    write_merge_report, write_qc_post_report, write_stats_report, write_trim_report,
    write_umi_report, write_validate_report,
};
use bijux_core::selection::{objective_spec, Objective};
use bijux_engine::api::init_logging;
use bijux_environment::image_qa::run_image_qa;
use bijux_stages_fastq::{benchmark_runs, write_benchmark_exports, AdapterPresetsV1};
use cli::{
    bench_args_correct, bench_args_filter, bench_args_from_trim, bench_args_from_validate,
    bench_args_merge, bench_args_preprocess, bench_args_qc_post, bench_args_screen,
    bench_args_stats, bench_args_trim, bench_args_umi, bench_args_validate,
    is_bench_requested_trim, is_bench_requested_validate, preprocess_args_from_cli, AnalyzeCommand,
    BenchCommand, BenchFastqCommand, Cli, Commands, EnvCommand, FastqCommand,
};
use env::{env_doctor, print_env_images, print_env_info};
use replay::replay_run;
use utils::normalize_run_base_dir;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().context("resolve current directory")?;
    if let Some(path) = &cli.telemetry_jsonl {
        let telemetry_path = if path.is_absolute() {
            path.clone()
        } else {
            cwd.join(path)
        };
        std::env::set_var("BIJUX_TELEMETRY_JSONL", telemetry_path);
    }
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
    if cli.print_effective_config {
        let payload = serde_json::json!({
            "profile": profile,
            "platform": cli.platform,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

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
            let objective = objective_spec(Objective::Balanced);
            let run_a = args.search_root.join(&args.run_a);
            let run_b = args.search_root.join(&args.run_b);
            let result = compare_runs(&run_a, &run_b, &objective)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(true)
        }
        Commands::Analyze { command } => {
            match command {
                AnalyzeCommand::Runs(args) => {
                    let query = bijux_core::run_index::RunQuery {
                        stage: args.stage.clone(),
                        tool: args.tool.clone(),
                        objective: args.objective.map(|obj| obj.as_str().to_string()),
                        success: args.success,
                    };
                    let runs = bijux_core::run_index::query_runs(&args.index, &query)?;
                    println!("{}", serde_json::to_string_pretty(&runs)?);
                }
            }
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
                        let qc_class = qc_class_label("fastq.validate_pre");
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
                        let qc_class = qc_class_label("fastq.qc_post");
                        write_qc_post_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            qc_class,
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

#[allow(clippy::too_many_lines)]
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
            let qc_class = qc_class_label("fastq.validate_pre");
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
            let objective = objective_spec(Objective::Balanced);
            let run_a = args.search_root.join(&args.run_a);
            let run_b = args.search_root.join(&args.run_b);
            let result = compare_runs(&run_a, &run_b, &objective)?;
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
            list_fastq_stages();
            Ok(Some(true))
        }
        FastqCommand::Stages => {
            list_fastq_stage_registry();
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
        FastqCommand::Trim(args) => {
            if args.list_adapter_presets {
                let selection = load_adapter_selection(
                    args.adapter_bank_preset.as_deref(),
                    args.adapter_bank.as_deref(),
                    args.adapter_bank_file.as_deref(),
                )?;
                list_adapter_presets(&selection.presets);
                return Ok(Some(true));
            }
            if args.list_adapters {
                let selection = load_adapter_selection(
                    args.adapter_bank_preset.as_deref(),
                    args.adapter_bank.as_deref(),
                    args.adapter_bank_file.as_deref(),
                )?;
                let effective = resolve_effective_adapters(
                    &selection,
                    &args.enable_adapter,
                    &args.disable_adapter,
                )?;
                list_adapters(&effective);
                return Ok(Some(true));
            }
            Ok(None)
        }
        FastqCommand::Preprocess(args) => {
            if args.list_adapter_presets {
                let selection = load_adapter_selection(
                    args.adapter_bank_preset.as_deref(),
                    args.adapter_bank.as_deref(),
                    args.adapter_bank_file.as_deref(),
                )?;
                list_adapter_presets(&selection.presets);
                return Ok(Some(true));
            }
            if args.list_adapters {
                let selection = load_adapter_selection(
                    args.adapter_bank_preset.as_deref(),
                    args.adapter_bank.as_deref(),
                    args.adapter_bank_file.as_deref(),
                )?;
                let effective = resolve_effective_adapters(
                    &selection,
                    &args.enable_adapter,
                    &args.disable_adapter,
                )?;
                list_adapters(&effective);
                return Ok(Some(true));
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn list_fastq_stages() {
    for stage in bijux_stages_fastq::fastq::registry() {
        println!("{}", stage.id);
    }
}

fn list_fastq_stage_registry() {
    for stage in bijux_stages_fastq::fastq::registry() {
        println!("{} v{}", stage.id, stage.version.0);
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

fn load_adapter_selection(
    adapter_bank_preset: Option<&str>,
    legacy_adapter_bank: Option<&str>,
    adapter_bank_file: Option<&Path>,
) -> Result<AdapterSelection> {
    resolve_adapter_selection(adapter_bank_preset, legacy_adapter_bank, adapter_bank_file)
}

fn list_adapter_presets(presets: &AdapterPresetsV1) {
    for preset in &presets.presets {
        let categories = if preset.tags.is_empty() {
            "none".to_string()
        } else {
            preset.tags.join(", ")
        };
        println!("{}: categories: {}", preset.name, categories);
    }
}

fn list_adapters(effective: &bijux_stages_fastq::EffectiveAdapterSet) {
    println!("preset: {}", effective.preset);
    println!("id\ttags\tname\tread_scope\tenabled_by_default");
    for adapter in &effective.adapters {
        let read_scope = match adapter.read_scope {
            bijux_stages_fastq::ReadScope::R1 => "r1",
            bijux_stages_fastq::ReadScope::R2 => "r2",
            bijux_stages_fastq::ReadScope::Both => "both",
            bijux_stages_fastq::ReadScope::SingleEnd => "single_end",
            bijux_stages_fastq::ReadScope::PairedEnd => "paired_end",
            bijux_stages_fastq::ReadScope::Unknown => "unknown",
        };
        let tags = if adapter.tags.is_empty() {
            "none".to_string()
        } else {
            adapter.tags.join(",")
        };
        println!(
            "{}\t{}\t{}\t{}\t{}",
            adapter.id, tags, adapter.name, read_scope, adapter.enabled_by_default
        );
    }
}

fn explain_fastq_stage(registry: &bijux_core::ToolRegistry, stage_id: &str) -> Result<()> {
    if stage_id == "fastq.preprocess" {
        let args = bijux_stages_fastq::args::BenchFastqPreprocessArgs {
            sample_id: "explain".to_string(),
            r1: PathBuf::from("reads.fastq.gz"),
            r2: None,
            out: PathBuf::from("artifacts"),
            strict: false,
            auto: false,
            objective: bijux_core::selection::Objective::Balanced,
            bench_corpus: None,
            allow_partial: false,
            adapter_bank_preset: None,
            adapter_bank: Some(format!(
                "preset:{}",
                crate::adapter_bank::DEFAULT_ADAPTER_PRESET
            )),
            adapter_bank_file: None,
            enable_adapters: Vec::new(),
            disable_adapters: Vec::new(),
            polyx_preset: None,
            contaminant_preset: None,
        };
        let plan = crate::fastq_exec::fastq_preprocess_plan(&args);
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

fn qc_class_label(stage: &str) -> Option<&'static str> {
    match bijux_stages_fastq::qc_class_for_stage(stage) {
        Some(bijux_stages_fastq::QcClass::Structural) => Some("structural"),
        Some(bijux_stages_fastq::QcClass::Statistical) => Some("statistical"),
        None => None,
    }
}
