use std::collections::HashMap;
use std::fs;

use anyhow::{Context, Result};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};

use bijux_engine::api::bench_base_dir;

use crate::core::filter::bench_fastq_filter;
use crate::core::stats::bench_fastq_stats;
use crate::core::trim::bench_fastq_trim;
use crate::core::validate::bench_fastq_validate;
use crate::stages::helpers::write_explain_plan_json;

/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning, execution, or metric recording fails.
pub fn bench_fastq_preprocess<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::stages::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    fastq_preprocess_run(catalog, platform, runner_override, args)
}

/// Execute the preprocess pipeline.
///
/// # Errors
/// Returns an error if any stage fails.
pub fn fastq_preprocess_run<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::stages::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    let out_dir = bench_base_dir(&args.out, "preprocess", &args.sample_id);
    fs::create_dir_all(&out_dir).context("create preprocess output dir")?;
    let pipeline = crate::meta::preprocess::fastq_preprocess_plan(args);
    let explain = format!(
        "# Explain: fastq.preprocess\n\nPipeline:\n- {}",
        pipeline.stages.join("\n- ")
    );
    fs::write(out_dir.join("explain.md"), explain).context("write explain.md")?;
    let selected_tools = vec![
        "fastqvalidator_official".to_string(),
        "fastp".to_string(),
        "fastp".to_string(),
        "seqkit_stats".to_string(),
    ];
    let registry = bijux_engine::api::load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow::anyhow!("manifest validation failed: {err}"))?;
    write_explain_plan_json(
        &out_dir,
        "fastq.preprocess",
        &selected_tools,
        &registry,
        None,
    )?;

    let validate_args = crate::stages::args::BenchFastqValidateArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: vec!["fastqvalidator_official".to_string()],
        explain: false,
        strict: args.strict,
    };
    let _ = bench_fastq_validate(catalog, platform, runner_override, &validate_args)?;

    let trim_args = crate::stages::args::BenchFastqTrimArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: vec!["fastp".to_string()],
        explain: false,
    };
    let _ = bench_fastq_trim(catalog, platform, runner_override, &trim_args)?;

    let filter_args = crate::stages::args::BenchFastqFilterArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: vec!["fastp".to_string()],
        explain: false,
    };
    let _ = bench_fastq_filter(catalog, platform, runner_override, &filter_args)?;

    let stats_args = crate::stages::args::BenchFastqStatsArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: vec!["seqkit_stats".to_string()],
        explain: false,
    };
    let _ = bench_fastq_stats(catalog, platform, runner_override, &stats_args)?;

    Ok(())
}
