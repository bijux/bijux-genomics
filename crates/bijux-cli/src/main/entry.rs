use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_core::{CategorizedError, ErrorCategory};
use bijux_api::v1::run::atomic_write_bytes;
use bijux_api::v1::run::{
    load_manifests, load_profile, new_run_id, DryRunExecutor, Executor,
};
use bijux_api::v1::run::{load_image_catalog, load_platform};
use bijux_api::v1::run::{PathSpec, RunSpec};
use clap::Parser;
use tracing::{info, warn};

use bijux_api::v1::bench::{
    bench_fastq_correct, bench_fastq_filter, bench_fastq_merge, bench_fastq_preprocess,
    bench_fastq_qc_post, bench_fastq_screen, bench_fastq_stats_neutral, bench_fastq_trim,
    bench_fastq_umi, bench_fastq_validate_pre, compare_runs, compare_runs_with_baseline,
    print_bench_schema, RankInput,
};
use bijux_api::v1::bench::fastq_banks::{
    resolve_adapter_selection, resolve_effective_adapters, AdapterSelection,
};
use bijux_api::v1::bench::{benchmark_runs, write_benchmark_exports, AdapterPresetsV1};
use bijux_api::v1::report::{
    load_facts_auto, load_run_summary, write_correct_report, write_filter_report,
    write_merge_report, write_qc_post_report, write_run_report_from_facts,
    write_run_summary_from_facts, write_stage_summary_csv, write_stats_report, write_trim_report,
    write_umi_report, write_validate_report,
};
use bijux_api::v1::run::run_image_qa;
use bijux_api::v1::run::init_logging;
use bijux_api::v1::run::{objective_spec, Objective};
use cli::{
    bench_args_correct, bench_args_filter, bench_args_from_trim, bench_args_from_validate,
    bench_args_merge, bench_args_preprocess, bench_args_qc_post, bench_args_screen,
    bench_args_stats, bench_args_trim, bench_args_umi, bench_args_validate,
    fastq_cross_args_from_cli, is_bench_requested_trim, is_bench_requested_validate,
    preprocess_args_from_cli, AnalyzeCommand,
    BenchBamCommand, BenchCommand, BenchFastqCommand, Cli, Commands, EnvCommand, FastqCommand,
    PipelinesCommand,
};
use env::{env_doctor, print_env_images, print_env_info};
use main_helpers::{
    ensure_profile_run_base_dir, load_profile_for_cli, normalize_fastq_stage_id, qc_class_label,
    render_report_bundle_html, resolve_report_inputs,
};
use bijux_api::v1::run::{normalize_run_base_dir, replay_run};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(exit_code_for_error(&err));
    }
}

fn exit_code_for_error(err: &anyhow::Error) -> i32 {
    if let Some(category) = error_category_from_chain(err) {
        return match category {
            ErrorCategory::UserError => 2,
            ErrorCategory::DataError => 3,
            ErrorCategory::ToolError => 4,
            ErrorCategory::InfraError => 5,
            ErrorCategory::Bug => 70,
        };
    }
    let msg = err.to_string().to_lowercase();
    if msg.contains("invalid arg") || msg.contains("usage:") {
        2
    } else if msg.contains("invalid") || msg.contains("missing") || msg.contains("not found") {
        3
    } else if msg.contains("tool") && msg.contains("failed") {
        4
    } else if msg.contains("contract") || msg.contains("invariant") {
        5
    } else {
        70
    }
}

fn error_category_from_chain(err: &anyhow::Error) -> Option<ErrorCategory> {
    if let Some(cat) = err.downcast_ref::<ErrorCategory>() {
        return Some(*cat);
    }
    if let Some(categorized) = err.downcast_ref::<CategorizedError>() {
        return Some(categorized.category);
    }
    for cause in err.chain() {
        if let Some(cat) = cause.downcast_ref::<ErrorCategory>() {
            return Some(*cat);
        }
        if let Some(categorized) = cause.downcast_ref::<CategorizedError>() {
            return Some(categorized.category);
        }
    }
    None
}

fn run() -> Result<()> {
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
        .join(format!("{}.toml", cli.profile));
    let mut profile = load_profile(&profile_path).map_err(|err| {
        anyhow!(CategorizedError::new(
            ErrorCategory::UserError,
            format!("failed to load profile {}: {err}", profile_path.display())
        ))
    })?;
    profile.run_base_dir = normalize_run_base_dir(&cwd, &profile.run_base_dir);
    if cli.print_effective_config || cli.dump_effective_config {
        let payload = serde_json::json!({
            "profile": profile,
            "platform": cli.platform,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    let registry = load_manifests(&domain_dir).map_err(|err| {
        anyhow!(CategorizedError::new(
            ErrorCategory::DataError,
            format!("manifest validation failed: {err}")
        ))
    })?;

    if handle_fastq_bench(&cli, &registry)? {
        return Ok(());
    }

    if handle_bam_commands(&cli, &registry, &domain_dir)? {
        return Ok(());
    }

    run_plan(&cli, &registry, &domain_dir)
}
