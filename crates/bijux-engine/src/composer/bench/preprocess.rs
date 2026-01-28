use std::fs;

use anyhow::{Context, Result};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};

use crate::composer::paths::bench_base_dir;

use super::filter::bench_fastq_filter;
use super::stats::bench_fastq_stats;
use super::trim::bench_fastq_trim;
use super::validate::bench_fastq_validate;

pub fn bench_fastq_preprocess(
    catalog: &std::collections::HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::composer::bench::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    let out_dir = bench_base_dir(&args.out, "preprocess", &args.sample_id);
    fs::create_dir_all(&out_dir).context("create preprocess output dir")?;
    let explain = [
        "# Explain: fastq.preprocess",
        "",
        "Pipeline:",
        "- fastq.validate",
        "- fastq.trim",
        "- fastq.filter",
        "- fastq.stats",
    ]
    .join("\n");
    fs::write(out_dir.join("explain.md"), explain).context("write explain.md")?;

    let validate_args = crate::composer::bench::args::BenchFastqValidateArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: vec!["fastqvalidator_official".to_string()],
        explain: false,
        strict: args.strict,
    };
    bench_fastq_validate(catalog, platform, runner_override, &validate_args)?;

    let trim_args = crate::composer::bench::args::BenchFastqTrimArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: vec!["fastp".to_string()],
        explain: false,
    };
    bench_fastq_trim(catalog, platform, runner_override, &trim_args)?;

    let filter_args = crate::composer::bench::args::BenchFastqFilterArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: vec!["fastp".to_string()],
        explain: false,
    };
    bench_fastq_filter(catalog, platform, runner_override, &filter_args)?;

    let stats_args = crate::composer::bench::args::BenchFastqStatsArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: vec!["seqkit_stats".to_string()],
        explain: false,
    };
    bench_fastq_stats(catalog, platform, runner_override, &stats_args)?;

    Ok(())
}
