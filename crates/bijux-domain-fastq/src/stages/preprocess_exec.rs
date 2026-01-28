use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};

use bijux_engine::api::bench_base_dir;

use crate::contracts::FastqLayout;
use crate::pipeline::{
    assess_input_dir, bench_corpus, create_run_layout, now_string, rank_tools_for_stage,
    update_run_index, write_input_assessment, write_selection_report, RunEnvironment,
    RunIndexEntry, RunLayout, RunManifest, RunStageEntry, ToolImageDigest,
};
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
    let (run_id, layout) = create_run_layout(&args.out)?;
    let input_dir = args
        .r1
        .parent()
        .map_or_else(|| args.out.clone(), PathBuf::from);
    let assessment = assess_input_dir(&input_dir)?;
    write_input_assessment(&layout.assessment_path, &assessment)?;
    let matched_sample = assessment
        .samples
        .iter()
        .find(|sample| sample.id.r1_path == args.r1);
    if args.r2.is_some()
        && matched_sample
            .as_ref()
            .and_then(|sample| sample.id.r2_path.clone())
            .is_none()
    {
        return Err(anyhow!(
            "input assessment did not find a paired R2 for the provided R1"
        ));
    }
    let derived_r2 = match (
        args.r2.clone(),
        matched_sample.and_then(|s| s.id.r2_path.clone()),
    ) {
        (Some(r2), _) | (None, Some(r2)) => Some(r2),
        (None, None) => None,
    };
    let layout_kind = if derived_r2.is_some() {
        FastqLayout::PairedEnd
    } else {
        FastqLayout::SingleEnd
    };
    let pipeline =
        crate::pipeline::fastq_default_pipeline(crate::pipeline::DefaultPipelineOptions {
            paired: derived_r2.is_some(),
            ..Default::default()
        });
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
    let mut stage_entries = Vec::new();
    for (stage, tool) in selected_by_stage {
        let tool_id = tool.clone();
        match stage.as_str() {
            "fastq.validate_pre" => {
                let validate_args = crate::stages::args::BenchFastqValidateArgs {
                    sample_id: args.sample_id.clone(),
                    r1: args.r1.clone(),
                    out: args.out.clone(),
                    tools: vec![tool_id.clone()],
                    explain: false,
                    strict: args.strict,
                };
                let outcome =
                    bench_fastq_validate_pre(catalog, platform, runner_override, &validate_args)?;
                stage_entries.push(stage_entry_from_outcome(
                    "fastq.validate_pre",
                    &tool_id,
                    &args.out,
                    &args.sample_id,
                    &outcome.records,
                )?);
            }
            "fastq.trim" => {
                let trim_args = crate::stages::args::BenchFastqTrimArgs {
                    sample_id: args.sample_id.clone(),
                    r1: args.r1.clone(),
                    out: args.out.clone(),
                    tools: vec![tool_id.clone()],
                    explain: false,
                };
                let outcome = bench_fastq_trim(catalog, platform, runner_override, &trim_args)?;
                stage_entries.push(stage_entry_from_outcome(
                    "fastq.trim",
                    &tool_id,
                    &args.out,
                    &args.sample_id,
                    &outcome.records,
                )?);
            }
            "fastq.correct" => {
                let correct_args = crate::stages::args::BenchFastqCorrectArgs {
                    sample_id: args.sample_id.clone(),
                    r1: args.r1.clone(),
                    r2: derived_r2.clone(),
                    out: args.out.clone(),
                    tools: vec![tool_id.clone()],
                    explain: false,
                };
                let outcome =
                    bench_fastq_correct(catalog, platform, runner_override, &correct_args)?;
                stage_entries.push(stage_entry_from_outcome(
                    "fastq.correct",
                    &tool_id,
                    &args.out,
                    &args.sample_id,
                    &outcome.records,
                )?);
            }
            "fastq.merge" => {
                let r2 = derived_r2
                    .clone()
                    .ok_or_else(|| anyhow!("merge requires --r2"))?;
                let merge_args = crate::stages::args::BenchFastqMergeArgs {
                    sample_id: args.sample_id.clone(),
                    r1: args.r1.clone(),
                    r2,
                    out: args.out.clone(),
                    tools: vec![tool_id.clone()],
                    explain: false,
                };
                let outcome = bench_fastq_merge(catalog, platform, runner_override, &merge_args)?;
                stage_entries.push(stage_entry_from_outcome(
                    "fastq.merge",
                    &tool_id,
                    &args.out,
                    &args.sample_id,
                    &outcome.records,
                )?);
            }
            "fastq.filter" => {
                let filter_args = crate::stages::args::BenchFastqFilterArgs {
                    sample_id: args.sample_id.clone(),
                    r1: args.r1.clone(),
                    out: args.out.clone(),
                    tools: vec![tool_id.clone()],
                    explain: false,
                };
                let outcome = bench_fastq_filter(catalog, platform, runner_override, &filter_args)?;
                stage_entries.push(stage_entry_from_outcome(
                    "fastq.filter",
                    &tool_id,
                    &args.out,
                    &args.sample_id,
                    &outcome.records,
                )?);
            }
            "fastq.stats_neutral" => {
                let stats_args = crate::stages::args::BenchFastqStatsArgs {
                    sample_id: args.sample_id.clone(),
                    r1: args.r1.clone(),
                    out: args.out.clone(),
                    tools: vec![tool_id.clone()],
                    explain: false,
                };
                let outcome =
                    bench_fastq_stats_neutral(catalog, platform, runner_override, &stats_args)?;
                stage_entries.push(stage_entry_from_outcome(
                    "fastq.stats_neutral",
                    &tool_id,
                    &args.out,
                    &args.sample_id,
                    &outcome.records,
                )?);
            }
            _ => {}
        }
    }

    populate_run_layout(&layout, &mut stage_entries)?;

    let env = RunEnvironment {
        hostname: std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string()),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        runner: platform.runner.to_string(),
        platform: platform.name.clone(),
        tool_images: selected_tools
            .iter()
            .filter_map(|tool| {
                catalog.get(tool).map(|spec| ToolImageDigest {
                    tool: tool.clone(),
                    image: format!(
                        "{}/{}:{}-{}",
                        platform.image_prefix, spec.tool, spec.version, platform.arch
                    ),
                    digest: spec.digest.clone().unwrap_or_else(|| "unknown".to_string()),
                })
            })
            .collect(),
    };
    crate::pipeline::write_environment(&layout, &env)?;

    let manifest = RunManifest {
        run_id: run_id.clone(),
        timestamp: now_string(),
        pipeline: "fastq.preprocess".to_string(),
        layout: layout_kind,
        stages: stage_entries,
    };
    crate::pipeline::write_manifest(&layout, &manifest)?;

    update_run_index(
        &args.out,
        RunIndexEntry {
            run_id,
            pipeline: "fastq.preprocess".to_string(),
            stages: pipeline.stages,
            layout: layout_kind,
            tools: selected_tools,
            objective: if args.auto {
                Some(args.objective.as_str().to_string())
            } else {
                None
            },
        },
    )?;

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

fn stage_entry_from_outcome<T: bijux_analyze::StageMetricSchema>(
    stage: &str,
    tool: &str,
    out_dir: &Path,
    sample_id: &str,
    records: &[bijux_analyze::BenchmarkRecord<T>],
) -> Result<RunStageEntry> {
    let record = records
        .first()
        .ok_or_else(|| anyhow!("missing bench record for {stage}"))?;
    let params_hash = crate::stages::helpers::params_hash(&record.context.parameters)?;
    let run_id = crate::stages::helpers::compute_run_id(
        stage,
        tool,
        &record.context.image_digest,
        &record.context.input_hash,
        &params_hash,
    );
    let stage_dir = stage.trim_start_matches("fastq.");
    let run_dir = bijux_engine::api::bench_tools_dir(out_dir, stage_dir, sample_id)
        .join(tool)
        .join("run")
        .join(&run_id);
    Ok(RunStageEntry {
        stage_id: stage.to_string(),
        tool_id: tool.to_string(),
        metrics_path: run_dir.join("metrics.json"),
        logs_dir: run_dir.join("logs"),
        outputs_dir: run_dir.join("artifacts"),
    })
}

fn populate_run_layout(layout: &RunLayout, entries: &mut [RunStageEntry]) -> Result<()> {
    for entry in entries {
        let stage_name = entry.stage_id.trim_start_matches("fastq.");
        let stage_dir = layout.stages_dir.join(stage_name).join(&entry.tool_id);
        let outputs_dir = stage_dir.join("outputs");
        let logs_dir = stage_dir.join("logs");
        std::fs::create_dir_all(&outputs_dir).context("create stage outputs dir")?;
        std::fs::create_dir_all(&logs_dir).context("create stage logs dir")?;
        let metrics_path = stage_dir.join("metrics.json");

        if entry.metrics_path.exists() {
            std::fs::copy(&entry.metrics_path, &metrics_path)
                .context("copy metrics.json into run layout")?;
        }
        let tool_log = entry.logs_dir.join("tool.log");
        if tool_log.exists() {
            std::fs::copy(&tool_log, logs_dir.join("tool.log"))
                .context("copy tool.log into run layout")?;
        }

        entry.metrics_path = metrics_path;
        entry.logs_dir = logs_dir;
        entry.outputs_dir = outputs_dir;
    }
    Ok(())
}
