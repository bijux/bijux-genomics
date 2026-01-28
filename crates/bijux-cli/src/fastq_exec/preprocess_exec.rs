use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use uuid::Uuid;

use bijux_engine::api::bench_base_dir;
use bijux_engine::services::pipeline::{run_pipeline, StagePlan};

use bijux_core::{build_run_metadata_v1, RunMetadataV1, ToolInvocationV1};
use bijux_domain_fastq::FastqLayout;

use crate::fastq_exec::helpers::write_explain_plan_json;
use crate::fastq_exec::{
    bench_fastq_correct, bench_fastq_filter, bench_fastq_merge, bench_fastq_stats_neutral,
    bench_fastq_trim, bench_fastq_validate_pre,
};
use bijux_core::events::RunEvent;
use bijux_domain_fastq::{
    append_event, assess_input_dir, bench_corpus, canonical_tool_defaults, create_run_layout,
    now_string, update_run_index, write_input_assessment, write_run_metadata, RunEnvironment,
    RunIndexEntry, RunLayout, RunManifest, RunStageEntry, ToolImageDigest,
};

/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning, execution, or metric recording fails.
pub fn bench_fastq_preprocess<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_domain_fastq::args::BenchFastqPreprocessArgs,
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
    args: &bijux_domain_fastq::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    let out_dir = bench_base_dir(&args.out, "preprocess", &args.sample_id);
    fs::create_dir_all(&out_dir).context("create preprocess output dir")?;
    let started_at = chrono::Utc::now();
    let (run_id, layout) = create_run_layout(&args.out)?;
    let input_dir = args
        .r1
        .parent()
        .map_or_else(|| args.out.clone(), PathBuf::from);
    let assessment = assess_input_dir(&input_dir)?;
    if layout.assessment_path.exists() {
        return Err(anyhow!(
            "input assessment already exists at {}",
            layout.assessment_path.display()
        ));
    }
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
        bijux_domain_fastq::fastq_default_pipeline(bijux_domain_fastq::DefaultPipelineOptions {
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
    let defaults = canonical_tool_defaults();
    let mut selected_tools: Vec<String> = pipeline
        .stages
        .iter()
        .map(|stage| {
            defaults
                .get(stage.as_str())
                .copied()
                .unwrap_or("fastp")
                .to_string()
        })
        .collect();

    let mut objective_name: Option<String> = None;
    if args.auto {
        let corpus_id = args
            .bench_corpus
            .ok_or_else(|| anyhow::anyhow!("--bench-corpus is required with --auto"))?;
        let corpus = bench_corpus(corpus_id);
        let objective = bijux_analyze::selection::objective_spec(args.objective);
        objective_name = Some(objective.name.clone());
        let mut selections = Vec::new();
        for stage in &pipeline.stages {
            let tool_ids: Vec<String> = registry
                .tools_for_stage(stage)
                .iter()
                .map(|tool| tool.tool_id.clone())
                .collect();
            let mut tool_records = Vec::new();
            for tool in &tool_ids {
                let records = bijux_domain_fastq::get_results(stage, tool, &corpus, &args.out)?;
                tool_records.push((tool.clone(), records));
            }
            let selection = bijux_analyze::selection::select_stage(
                stage,
                &tool_records,
                &objective,
                args.allow_partial,
            );
            if selection.selected.is_none() {
                return Err(anyhow::anyhow!(
                    "no eligible tools for {stage}; check bench corpus/results"
                ));
            }
            selections.push(selection);
        }
        bijux_analyze::selection::write_selection_report(
            &out_dir,
            &objective,
            corpus_id.as_str(),
            selections.clone(),
        )?;
        selected_tools = selections
            .into_iter()
            .filter_map(|selection| selection.selected)
            .collect();
    }

    let selected_by_stage: Vec<StagePlan> = pipeline
        .stages
        .iter()
        .cloned()
        .zip(selected_tools.iter().cloned())
        .map(|(stage, tool)| StagePlan { stage, tool })
        .collect();

    append_event(
        &layout,
        &RunEvent {
            timestamp: now_string(),
            event: "pipeline_started".to_string(),
            stage: None,
            tool: None,
            detail: Some("fastq.preprocess".to_string()),
        },
    )?;

    write_explain_plan_json(
        &out_dir,
        "fastq.preprocess",
        &selected_tools,
        &registry,
        None,
    )?;
    let mut stage_entries = run_pipeline(
        &selected_by_stage,
        |event, step| {
            append_event(
                &layout,
                &RunEvent {
                    timestamp: now_string(),
                    event: event.to_string(),
                    stage: Some(step.stage.clone()),
                    tool: Some(step.tool.clone()),
                    detail: None,
                },
            )
        },
        |step| {
            let stage = step.stage.as_str();
            let tool_id = step.tool.clone();
            match stage {
                "fastq.validate_pre" => {
                    let validate_args = bijux_domain_fastq::args::BenchFastqValidateArgs {
                        sample_id: args.sample_id.clone(),
                        r1: args.r1.clone(),
                        out: args.out.clone(),
                        tools: vec![tool_id.clone()],
                        explain: false,
                        strict: args.strict,
                    };
                    let outcome = bench_fastq_validate_pre(
                        catalog,
                        platform,
                        runner_override,
                        &validate_args,
                    )?;
                    stage_entry_from_outcome(
                        "fastq.validate_pre",
                        &tool_id,
                        &args.out,
                        &args.sample_id,
                        &outcome.records,
                    )
                }
                "fastq.trim" => {
                    let trim_args = bijux_domain_fastq::args::BenchFastqTrimArgs {
                        sample_id: args.sample_id.clone(),
                        r1: args.r1.clone(),
                        out: args.out.clone(),
                        tools: vec![tool_id.clone()],
                        explain: false,
                    };
                    let outcome = bench_fastq_trim(catalog, platform, runner_override, &trim_args)?;
                    stage_entry_from_outcome(
                        "fastq.trim",
                        &tool_id,
                        &args.out,
                        &args.sample_id,
                        &outcome.records,
                    )
                }
                "fastq.correct" => {
                    let correct_args = bijux_domain_fastq::args::BenchFastqCorrectArgs {
                        sample_id: args.sample_id.clone(),
                        r1: args.r1.clone(),
                        r2: derived_r2.clone(),
                        out: args.out.clone(),
                        tools: vec![tool_id.clone()],
                        explain: false,
                    };
                    let outcome =
                        bench_fastq_correct(catalog, platform, runner_override, &correct_args)?;
                    stage_entry_from_outcome(
                        "fastq.correct",
                        &tool_id,
                        &args.out,
                        &args.sample_id,
                        &outcome.records,
                    )
                }
                "fastq.merge" => {
                    let r2 = derived_r2
                        .clone()
                        .ok_or_else(|| anyhow!("merge requires --r2"))?;
                    let merge_args = bijux_domain_fastq::args::BenchFastqMergeArgs {
                        sample_id: args.sample_id.clone(),
                        r1: args.r1.clone(),
                        r2,
                        out: args.out.clone(),
                        tools: vec![tool_id.clone()],
                        explain: false,
                    };
                    let outcome =
                        bench_fastq_merge(catalog, platform, runner_override, &merge_args)?;
                    stage_entry_from_outcome(
                        "fastq.merge",
                        &tool_id,
                        &args.out,
                        &args.sample_id,
                        &outcome.records,
                    )
                }
                "fastq.filter" => {
                    let filter_args = bijux_domain_fastq::args::BenchFastqFilterArgs {
                        sample_id: args.sample_id.clone(),
                        r1: args.r1.clone(),
                        out: args.out.clone(),
                        tools: vec![tool_id.clone()],
                        explain: false,
                    };
                    let outcome =
                        bench_fastq_filter(catalog, platform, runner_override, &filter_args)?;
                    stage_entry_from_outcome(
                        "fastq.filter",
                        &tool_id,
                        &args.out,
                        &args.sample_id,
                        &outcome.records,
                    )
                }
                "fastq.stats_neutral" => {
                    let stats_args = bijux_domain_fastq::args::BenchFastqStatsArgs {
                        sample_id: args.sample_id.clone(),
                        r1: args.r1.clone(),
                        out: args.out.clone(),
                        tools: vec![tool_id.clone()],
                        explain: false,
                    };
                    let outcome =
                        bench_fastq_stats_neutral(catalog, platform, runner_override, &stats_args)?;
                    stage_entry_from_outcome(
                        "fastq.stats_neutral",
                        &tool_id,
                        &args.out,
                        &args.sample_id,
                        &outcome.records,
                    )
                }
                _ => Err(anyhow!("unsupported stage {stage}")),
            }
        },
    )?;

    populate_run_layout(&layout, &mut stage_entries)?;

    let finished_at = chrono::Utc::now();
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
    bijux_domain_fastq::write_environment(&layout, &env)?;

    let manifest = RunManifest {
        run_id: run_id.clone(),
        started_at: started_at.to_rfc3339(),
        finished_at: finished_at.to_rfc3339(),
        pipeline: "fastq.preprocess".to_string(),
        layout: layout_kind,
        stages: stage_entries,
    };
    bijux_domain_fastq::write_manifest(&layout, &manifest)?;

    let deltas_path = layout.summary_dir.join("metrics_deltas.json");
    if !deltas_path.exists() {
        std::fs::write(&deltas_path, "{}")?;
    }
    let report_path = layout.summary_dir.join("report.json");
    if !report_path.exists() {
        std::fs::write(&report_path, "{}")?;
    }

    let platform_runner = platform.runner.to_string();
    let git_commit = std::env::var("BIJUX_GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string());
    let metadata: RunMetadataV1 = build_run_metadata_v1(
        Uuid::parse_str(&run_id)?,
        started_at,
        finished_at,
        &platform_runner,
        "unknown",
        env!("CARGO_PKG_VERSION"),
        &git_commit,
    );
    write_run_metadata(&layout, &metadata)?;

    update_run_index(
        &args.out,
        RunIndexEntry {
            run_id,
            domain: "fastq".to_string(),
            pipeline: "fastq.preprocess".to_string(),
            stages: pipeline.stages,
            layout: layout_kind,
            tools: selected_tools,
            objective: objective_name,
            platform: platform.runner.to_string(),
            success: true,
        },
    )?;

    append_event(
        &layout,
        &RunEvent {
            timestamp: now_string(),
            event: "pipeline_finished".to_string(),
            stage: None,
            tool: None,
            detail: Some("fastq.preprocess".to_string()),
        },
    )?;

    Ok(())
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
    let params_hash = crate::fastq_exec::helpers::params_hash(&record.context.parameters)?;
    let run_id = crate::fastq_exec::helpers::compute_run_id(
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
        execution_metrics_path: run_dir.join("metrics.json"),
        domain_metrics_path: run_dir.join("metrics.json"),
        logs_dir: run_dir.join("logs"),
        outputs_dir: run_dir.join("artifacts"),
        tool_invocation_path: run_dir.join("manifest.json"),
    })
}

fn populate_run_layout(layout: &RunLayout, entries: &mut [RunStageEntry]) -> Result<()> {
    for entry in entries {
        let stage_name = entry.stage_id.trim_start_matches("fastq.");
        let stage_dir = layout.stages_dir.join(stage_name);
        let outputs_dir = stage_dir.join("outputs");
        let logs_dir = stage_dir.join("logs");
        std::fs::create_dir_all(&outputs_dir).context("create stage outputs dir")?;
        std::fs::create_dir_all(&logs_dir).context("create stage logs dir")?;
        let execution_metrics_path = stage_dir.join("execution_metrics.json");
        let domain_metrics_path = stage_dir.join("metrics.json");
        let tool_invocation_path = stage_dir.join("tool_invocation.json");

        if entry.execution_metrics_path.exists() {
            let data = std::fs::read_to_string(&entry.execution_metrics_path)?;
            let payload: serde_json::Value = serde_json::from_str(&data)?;
            let execution = payload
                .get("execution")
                .cloned()
                .ok_or_else(|| anyhow!("missing execution metrics"))?;
            let metrics = payload
                .get("metrics")
                .cloned()
                .ok_or_else(|| anyhow!("missing domain metrics"))?;
            std::fs::write(
                &execution_metrics_path,
                serde_json::to_vec_pretty(&execution)?,
            )
            .context("write execution_metrics.json")?;
            std::fs::write(&domain_metrics_path, serde_json::to_vec_pretty(&metrics)?)
                .context("write metrics.json")?;
        }
        let source_run_dir = entry
            .execution_metrics_path
            .parent()
            .ok_or_else(|| anyhow!("missing run dir for metrics"))?;
        let manifest_path = source_run_dir.join("manifest.json");
        if manifest_path.exists() {
            let manifest_data = std::fs::read_to_string(&manifest_path)?;
            let manifest: bijux_engine::api::ExecutionManifest =
                serde_json::from_str(&manifest_data)?;
            let invocation = ToolInvocationV1 {
                stage: manifest.stage,
                tool: manifest.tool,
                version: manifest.tool_version,
                image: manifest.image_digest,
                command: manifest.command,
                threads: 0,
                inputs: manifest.input_files,
                outputs: vec![manifest.output_dir],
            };
            std::fs::write(
                &tool_invocation_path,
                serde_json::to_vec_pretty(&invocation)?,
            )
            .context("write tool_invocation.json")?;
        }
        let tool_log = entry.logs_dir.join("tool.log");
        if tool_log.exists() {
            std::fs::copy(&tool_log, logs_dir.join("tool.log"))
                .context("copy tool.log into run layout")?;
        }

        entry.execution_metrics_path = execution_metrics_path;
        entry.domain_metrics_path = domain_metrics_path;
        entry.logs_dir = logs_dir;
        entry.outputs_dir = outputs_dir;
        entry.tool_invocation_path = tool_invocation_path;
    }
    Ok(())
}
