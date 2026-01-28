use std::collections::HashMap;
use std::fs;

use anyhow::{Context, Result};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};

use bijux_engine::api::bench_base_dir;

use crate::pipeline::{bench_corpus, rank_tools_for_stage, write_selection_report};
use crate::stages::helpers::write_explain_plan_json;
use crate::stages::{
    bench_fastq_correct, bench_fastq_filter, bench_fastq_merge, bench_fastq_stats_neutral,
    bench_fastq_trim, bench_fastq_validate_pre,
};

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
#[allow(clippy::too_many_lines)]
pub fn fastq_preprocess_run<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::stages::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    let out_dir = bench_base_dir(&args.out, "preprocess", &args.sample_id);
    fs::create_dir_all(&out_dir).context("create preprocess output dir")?;
    let pipeline = crate::stages::fastq_preprocess_plan(args);
    let explain = format!(
        "# Explain: fastq.preprocess\n\nPipeline:\n- {}",
        pipeline.stages.join("\n- ")
    );
    fs::write(out_dir.join("explain.md"), explain).context("write explain.md")?;
    let registry = bijux_engine::api::load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow::anyhow!("manifest validation failed: {err}"))?;
    let mut selected_tools: Vec<String> = pipeline
        .stages
        .iter()
        .map(|stage| default_tool_for_stage(stage).to_string())
        .collect();

    if args.auto {
        let corpus_id = args
            .bench_corpus
            .ok_or_else(|| anyhow::anyhow!("--bench-corpus is required with --auto"))?;
        let corpus = bench_corpus(corpus_id);
        let objective = args.objective;
        let mut selections = Vec::new();
        for stage in &pipeline.stages {
            let tool_ids: Vec<String> = registry
                .tools_for_stage(stage)
                .iter()
                .map(|tool| tool.tool_id.clone())
                .collect();
            let selection = rank_tools_for_stage(
                stage,
                &tool_ids,
                objective,
                &corpus,
                &args.out,
                args.allow_partial,
            )?;
            if selection.selected.is_none() {
                return Err(anyhow::anyhow!(
                    "no eligible tools for {stage}; check bench corpus/results"
                ));
            }
            selections.push(selection);
        }
        write_selection_report(&out_dir, objective, corpus_id, selections.clone())?;
        selected_tools = selections
            .into_iter()
            .filter_map(|selection| selection.selected)
            .collect();
    }

    let selected_by_stage: Vec<(String, String)> = pipeline
        .stages
        .iter()
        .cloned()
        .zip(selected_tools.iter().cloned())
        .collect();

    write_explain_plan_json(
        &out_dir,
        "fastq.preprocess",
        &selected_tools,
        &registry,
        None,
    )?;
    for (stage, tool) in selected_by_stage {
        match stage.as_str() {
            "fastq.validate_pre" => {
                let validate_args = crate::stages::args::BenchFastqValidateArgs {
                    sample_id: args.sample_id.clone(),
                    r1: args.r1.clone(),
                    out: args.out.clone(),
                    tools: vec![tool],
                    explain: false,
                    strict: args.strict,
                };
                let _ =
                    bench_fastq_validate_pre(catalog, platform, runner_override, &validate_args)?;
            }
            "fastq.trim" => {
                let trim_args = crate::stages::args::BenchFastqTrimArgs {
                    sample_id: args.sample_id.clone(),
                    r1: args.r1.clone(),
                    out: args.out.clone(),
                    tools: vec![tool],
                    explain: false,
                };
                let _ = bench_fastq_trim(catalog, platform, runner_override, &trim_args)?;
            }
            "fastq.correct" => {
                let correct_args = crate::stages::args::BenchFastqCorrectArgs {
                    sample_id: args.sample_id.clone(),
                    r1: args.r1.clone(),
                    r2: args.r2.clone(),
                    out: args.out.clone(),
                    tools: vec![tool],
                    explain: false,
                };
                let _ = bench_fastq_correct(catalog, platform, runner_override, &correct_args)?;
            }
            "fastq.merge" => {
                let r2 = args
                    .r2
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("merge requires --r2"))?;
                let merge_args = crate::stages::args::BenchFastqMergeArgs {
                    sample_id: args.sample_id.clone(),
                    r1: args.r1.clone(),
                    r2,
                    out: args.out.clone(),
                    tools: vec![tool],
                    explain: false,
                };
                let _ = bench_fastq_merge(catalog, platform, runner_override, &merge_args)?;
            }
            "fastq.filter" => {
                let filter_args = crate::stages::args::BenchFastqFilterArgs {
                    sample_id: args.sample_id.clone(),
                    r1: args.r1.clone(),
                    out: args.out.clone(),
                    tools: vec![tool],
                    explain: false,
                };
                let _ = bench_fastq_filter(catalog, platform, runner_override, &filter_args)?;
            }
            "fastq.stats_neutral" => {
                let stats_args = crate::stages::args::BenchFastqStatsArgs {
                    sample_id: args.sample_id.clone(),
                    r1: args.r1.clone(),
                    out: args.out.clone(),
                    tools: vec![tool],
                    explain: false,
                };
                let _ = bench_fastq_stats_neutral(catalog, platform, runner_override, &stats_args)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn default_tool_for_stage(stage: &str) -> &'static str {
    match stage {
        "fastq.validate_pre" => "fastqvalidator_official",
        "fastq.stats_neutral" => "seqkit_stats",
        "fastq.merge" => "vsearch",
        "fastq.correct" => "rcorrector",
        _ => "fastp",
    }
}
