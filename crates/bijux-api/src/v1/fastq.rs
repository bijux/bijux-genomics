//! FASTQ domain helpers for v1.

pub use bijux_planner_fastq::stage_api as fastq_banks;
pub use bijux_planner_fastq::stage_api::{
    adapter_bank_path, adapter_presets_path, benchmark_runs, contaminant_motifs_path,
    contaminant_presets_path, contaminant_references_dir, load_adapter_bank, load_adapter_presets,
    load_contaminant_motifs, load_contaminant_presets, load_polyx_bank, load_polyx_presets,
    polyx_bank_path, polyx_presets_path, qc_class_for_stage, write_benchmark_exports,
    AdapterPresetsV1, BenchCorpusId, EffectiveAdapterSet, QcClass, ReadScope, STAGES,
};

pub use bijux_planner_fastq::stage_api::args as fastq_args;

use std::collections::HashMap;
use std::hash::BuildHasher;

use anyhow::Result;
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};

use crate::fastq_router::BenchOutcome;

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_correct<S: BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_planner_fastq::stage_api::args::BenchFastqCorrectArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqCorrectMetrics>> {
    crate::fastq_router::bench_fastq_correct(catalog, platform, runner_override, args)
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_filter<S: BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_planner_fastq::stage_api::args::BenchFastqFilterArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqFilterMetrics>> {
    crate::fastq_router::bench_fastq_filter(catalog, platform, runner_override, args)
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_merge<S: BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_planner_fastq::stage_api::args::BenchFastqMergeArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqMergeMetrics>> {
    crate::fastq_router::bench_fastq_merge(catalog, platform, runner_override, args)
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_preprocess<S: BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_planner_fastq::stage_api::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    crate::fastq_router::bench_fastq_preprocess(catalog, platform, runner_override, args)
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_qc_post<S: BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_planner_fastq::stage_api::args::BenchFastqQcPostArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqQcPostMetrics>> {
    crate::fastq_router::bench_fastq_qc_post(catalog, platform, runner_override, args)
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_screen<S: BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_planner_fastq::stage_api::args::BenchFastqScreenArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqScreenMetrics>> {
    crate::fastq_router::bench_fastq_screen(catalog, platform, runner_override, args)
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_stats_neutral<S: BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_planner_fastq::stage_api::args::BenchFastqStatsArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqStatsMetrics>> {
    crate::fastq_router::bench_fastq_stats_neutral(catalog, platform, runner_override, args)
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_trim<S: BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_planner_fastq::stage_api::args::BenchFastqTrimArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqTrimMetrics>> {
    crate::fastq_router::bench_fastq_trim(catalog, platform, runner_override, args)
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_umi<S: BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_planner_fastq::stage_api::args::BenchFastqUmiArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqUmiMetrics>> {
    crate::fastq_router::bench_fastq_umi(catalog, platform, runner_override, args)
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_validate_pre<S: BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_planner_fastq::stage_api::args::BenchFastqValidateArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqValidateMetrics>> {
    crate::fastq_router::bench_fastq_validate_pre(catalog, platform, runner_override, args)
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn fastq_preprocess_run<S: BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_planner_fastq::stage_api::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    crate::fastq_router::fastq_preprocess_run(catalog, platform, runner_override, args)
}
