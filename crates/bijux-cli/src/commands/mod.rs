#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_api::v1::bench::{objective_spec, Objective};
use bijux_api::v1::env::{load_image_catalog, load_platform, run_image_qa};
use bijux_api::v1::run::{
    atomic_write_bytes, load_manifests, new_run_id, PathSpec, RunSpec, StageId,
};
use bijux_api::v1::run::{DryRunExecutor, Executor};
use tracing::{info, warn};

use bijux_api::v1::bench::fastq_banks::{
    resolve_adapter_selection, resolve_effective_adapters, AdapterSelection,
};
use bijux_api::v1::bench::{
    bench_fastq_correct, bench_fastq_filter, bench_fastq_merge, bench_fastq_preprocess,
    bench_fastq_qc_post, bench_fastq_screen, bench_fastq_stats_neutral, bench_fastq_trim,
    bench_fastq_umi, bench_fastq_validate_pre, compare_runs, compare_runs_with_baseline,
    print_bench_schema, RankInput,
};
use bijux_api::v1::bench::{benchmark_runs, write_benchmark_exports, AdapterPresetsV1};
use bijux_api::v1::report::render_report_bundle_html;
use bijux_api::v1::report::{
    load_facts_auto, load_run_summary, write_correct_report, write_filter_report,
    write_merge_report, write_qc_post_report, write_run_report_from_facts,
    write_run_summary_from_facts, write_stage_summary_csv, write_stats_report, write_trim_report,
    write_umi_report, write_validate_report,
};
use bijux_api::v1::run::init_logging;

use crate::commands::cli::env::{env_doctor, print_env_images, print_env_info};
use crate::commands::cli::{
    bench_args_correct, bench_args_filter, bench_args_from_trim, bench_args_from_validate,
    bench_args_merge, bench_args_preprocess, bench_args_qc_post, bench_args_screen,
    bench_args_stats, bench_args_trim, bench_args_umi, bench_args_validate,
    fastq_cross_args_from_cli, is_bench_requested_trim, is_bench_requested_validate,
    preprocess_args_from_cli, AnalyzeCommand, BenchBamCommand, BenchCommand, BenchFastqCommand,
    Cli, Commands, EnvCommand, FastqCommand, PipelinesCommand, PoliciesCommand,
};
use crate::commands::formatting::{normalize_fastq_stage_id, qc_class_label};
use crate::commands::rendering::resolve_report_inputs;
use crate::commands::validation::{ensure_profile_run_base_dir, load_profile_for_cli};
use crate::render;

pub mod cli;
pub(crate) mod formatting;
pub(crate) mod rendering;
pub(crate) mod validation;

include!("bench.rs");
include!("fastq.rs");
include!("bam.rs");
include!("other.rs");
include!("policies.rs");
