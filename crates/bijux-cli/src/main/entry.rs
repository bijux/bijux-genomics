use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_core::{
    build_execution_plan, load_manifests, load_profile, new_run_id, DryRunExecutor, Executor,
    PathSpec, RunSpec,
};
use bijux_env_runtime::api::{load_image_catalog, load_platform};
use clap::Parser;
use tracing::{info, warn};

use crate::adapter_bank::{
    resolve_adapter_selection, resolve_effective_adapters, AdapterSelection,
};
use crate::fastq_exec::{
    bench_fastq_correct, bench_fastq_filter, bench_fastq_merge, bench_fastq_preprocess,
    bench_fastq_qc_post, bench_fastq_screen, bench_fastq_stats_neutral, bench_fastq_trim,
    bench_fastq_umi, bench_fastq_validate_pre,
};
use bijux_analyze::compare::compare_runs_with_baseline;
use bijux_analyze::export::write_stage_summary_csv;
use bijux_analyze::{
    compare_runs, load_facts_auto, load_run_summary, print_bench_schema, write_correct_report,
    write_filter_report, write_merge_report, write_qc_post_report, write_run_report_from_facts,
    write_run_summary_from_facts, write_stats_report, write_trim_report, write_umi_report,
    write_validate_report, RankInput,
};
use bijux_core::selection::{objective_spec, Objective};
use bijux_engine::api::init_logging;
use bijux_env_builder::image_qa::run_image_qa;
use bijux_stages_fastq::{benchmark_runs, write_benchmark_exports, AdapterPresetsV1};
use cli::{
    bench_args_correct, bench_args_filter, bench_args_from_trim, bench_args_from_validate,
    bench_args_merge, bench_args_preprocess, bench_args_qc_post, bench_args_screen,
    bench_args_stats, bench_args_trim, bench_args_umi, bench_args_validate,
    is_bench_requested_trim, is_bench_requested_validate, preprocess_args_from_cli, AnalyzeCommand,
    BenchCommand, BenchFastqCommand, Cli, Commands, EnvCommand, FastqCommand,
};
use env::{env_doctor, print_env_images, print_env_info};
use main_helpers::{
    ensure_profile_run_base_dir, load_profile_for_cli, normalize_fastq_stage_id, qc_class_label,
    render_report_bundle_html, resolve_report_inputs,
};
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
