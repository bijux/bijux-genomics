use std::collections::HashMap;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::quality::{
    fetch_fastq_filter_low_complexity_v1, insert_fastq_filter_low_complexity_v1,
};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqLowComplexityMetrics};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::params_hash;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::{
    FilterLowComplexityReportV1, PairedMode, FILTER_LOW_COMPLEXITY_REPORT_SCHEMA_VERSION,
};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::select_filter_low_complexity_tools;
use bijux_dna_planner_fastq::stage_api::fastq::filter_low_complexity::{
    plan_low_complexity, LowComplexityPlanOptions,
};
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use bijux_dna_stage_contract::StagePlanV1;
use uuid::Uuid;

use crate::internal::fastq::stages::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context, derive_trim_delta, observe_fastq_stats,
    prepare_trim_bench, TrimBenchInputs,
};
use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_FILTER_LOW_COMPLEXITY,
};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;

/// # Errors
/// Returns an error if planning, execution, metrics derivation, or persistence fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_filter_low_complexity<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqFilterLowComplexityArgs,
) -> Result<BenchOutcome<FastqLowComplexityMetrics>> {
    let tools = select_low_complexity_benchmark_tools(args)?;
    let setup =
        prepare_low_complexity_benchmark_setup(catalog, platform, runner_override, args, &tools)?;

    if args.explain {
        write_low_complexity_benchmark_explain(&setup)?;
    }

    ensure_low_complexity_benchmark_qa(catalog, platform, &setup.tools)?;

    let sqlite_path = setup.bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = setup.bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::<BenchmarkRecord<FastqLowComplexityMetrics>>::new();
    for tool in &setup.tools {
        let tool_plan =
            prepare_low_complexity_tool_plan(catalog, platform, args, &setup, jobs, tool)?;
        if let Ok(Some(record)) = fetch_fastq_filter_low_complexity_v1(
            &conn,
            &tool_plan.tool,
            &tool_plan.tool_spec.tool_version,
            &tool_plan.image_digest,
            &setup.bench_inputs.runner.to_string(),
            &platform.name,
            &setup.input_hash,
            &tool_plan.params_hash,
        ) {
            records.push(record);
            continue;
        }
        let execution = execute_low_complexity_tool(&tool_plan, setup.bench_inputs.runner, jobs)?;
        if let Some(failure) = low_complexity_tool_failure(&tool_plan, execution.exit_code) {
            failures.push(failure);
            continue;
        }
        let record = build_low_complexity_record(&LowComplexityRecordInputs {
            catalog,
            platform,
            bench_inputs: &setup.bench_inputs,
            input_stats_r2: setup.input_stats_r2.as_ref(),
            tool: &tool_plan.tool,
            tool_spec: &tool_plan.tool_spec,
            input_hash: &setup.input_hash,
            params: &tool_plan.plan.params,
            output_reads: &tool_plan.plan.io.outputs[0].path,
            output_reads_r2: tool_plan
                .plan
                .io
                .outputs
                .get(1)
                .map(|artifact| artifact.path.as_path()),
            execution: &execution,
        })?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_filter_low_complexity_v1(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: setup.bench_inputs.bench_dir,
        explain: args.explain,
    })
}

fn select_low_complexity_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqFilterLowComplexityArgs,
) -> Result<Vec<String>> {
    let tools = select_filter_low_complexity_tools(&args.tools)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_FILTER_LOW_COMPLEXITY.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_FILTER_LOW_COMPLEXITY.as_str(), &header);
    Ok(tools)
}

struct LowComplexityBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    bench_inputs: TrimBenchInputs,
    input_hash: String,
    input_stats_r2: Option<SeqkitMetrics>,
    options: LowComplexityPlanOptions,
}

struct LowComplexityToolPlan {
    tool: String,
    tool_spec: ToolExecutionSpecV1,
    plan: StagePlanV1,
    params_hash: String,
    image_digest: String,
}

struct LowComplexityRecordInputs<'a, S: ::std::hash::BuildHasher> {
    catalog: &'a HashMap<String, ToolImageSpec, S>,
    platform: &'a PlatformSpec,
    bench_inputs: &'a TrimBenchInputs,
    input_stats_r2: Option<&'a SeqkitMetrics>,
    tool: &'a str,
    tool_spec: &'a ToolExecutionSpecV1,
    input_hash: &'a str,
    params: &'a serde_json::Value,
    output_reads: &'a std::path::Path,
    output_reads_r2: Option<&'a std::path::Path>,
    execution: &'a StageResultV1,
}

struct LowComplexityReportInputs<'a> {
    tool: &'a str,
    threads: u32,
    params: &'a serde_json::Value,
    before_stats: &'a SeqkitMetrics,
    after_stats: &'a SeqkitMetrics,
    output_stats_r2: Option<&'a SeqkitMetrics>,
    input_stats_r2: Option<&'a SeqkitMetrics>,
    output_reads: &'a std::path::Path,
    output_reads_r2: Option<&'a std::path::Path>,
    execution: &'a StageResultV1,
}

fn prepare_low_complexity_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqFilterLowComplexityArgs,
    setup: &LowComplexityBenchmarkSetup,
    jobs: usize,
    tool: &str,
) -> Result<LowComplexityToolPlan> {
    let out_dir = setup.bench_inputs.tools_root.join(tool);
    bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
    let tool_spec = build_tool_execution_spec(
        STAGE_FILTER_LOW_COMPLEXITY.as_str(),
        tool,
        &setup.registry,
        catalog,
        platform,
    )?;
    let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
    let plan = plan_low_complexity(
        &tool_spec,
        &setup.bench_inputs.r1,
        args.r2.as_deref(),
        &out_dir,
        &setup.options,
    )?;
    let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
    let image_digest = benchmark_image_identity(&tool_spec);
    Ok(LowComplexityToolPlan { tool: tool.to_string(), tool_spec, plan, params_hash, image_digest })
}

fn execute_low_complexity_tool(
    tool_plan: &LowComplexityToolPlan,
    runner: RuntimeKind,
    jobs: usize,
) -> Result<StageResultV1> {
    execute_plans_with_jobs(
        vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&tool_plan.plan)],
        runner,
        jobs,
    )?
    .into_iter()
    .next()
    .ok_or_else(|| anyhow!("missing execution result for {}", tool_plan.tool))
}

fn low_complexity_tool_failure(
    tool_plan: &LowComplexityToolPlan,
    exit_code: i32,
) -> Option<RawFailure> {
    if exit_code == 0 {
        return None;
    }
    Some(RawFailure {
        stage: STAGE_FILTER_LOW_COMPLEXITY.as_str().to_string(),
        tool: tool_plan.tool.clone(),
        reason: format!("tool {} failed with status {exit_code}", tool_plan.tool),
        category: ErrorCategory::ToolError,
    })
}

fn prepare_low_complexity_benchmark_setup<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqFilterLowComplexityArgs,
    selected_tools: &[String],
) -> Result<LowComplexityBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(
        STAGE_FILTER_LOW_COMPLEXITY.as_str(),
        selected_tools,
        &registry,
        false,
    )?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_FILTER_LOW_COMPLEXITY,
    )?;
    let input_hash = if let Some(r2) = args.r2.as_deref() {
        format!("{}+{}", bench_inputs.input_hash, bijux_dna_infra::hash_file_sha256(r2)?)
    } else {
        bench_inputs.input_hash.clone()
    };
    let input_stats_r2 = if let Some(r2) = args.r2.as_deref() {
        Some(observe_fastq_stats(catalog, platform, bench_inputs.runner, r2)?)
    } else {
        None
    };
    let options = LowComplexityPlanOptions {
        entropy_threshold: args.entropy_threshold,
        polyx_threshold: args.polyx_threshold,
    };
    Ok(LowComplexityBenchmarkSetup {
        registry,
        tools,
        bench_inputs,
        input_hash,
        input_stats_r2,
        options,
    })
}

fn write_low_complexity_benchmark_explain(setup: &LowComplexityBenchmarkSetup) -> Result<()> {
    write_explain_md(
        &setup.bench_inputs.bench_dir,
        STAGE_FILTER_LOW_COMPLEXITY.as_str(),
        &setup.tools,
        &[],
        None,
    )?;
    write_explain_plan_json(
        &setup.bench_inputs.bench_dir,
        STAGE_FILTER_LOW_COMPLEXITY.as_str(),
        &setup.tools,
        &setup.registry,
        None,
    )
}

fn ensure_low_complexity_benchmark_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(STAGE_FILTER_LOW_COMPLEXITY.as_str(), tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_FILTER_LOW_COMPLEXITY.as_str(), tools, platform, catalog)
}

fn build_low_complexity_record<S: ::std::hash::BuildHasher>(
    inputs: &LowComplexityRecordInputs<'_, S>,
) -> Result<BenchmarkRecord<FastqLowComplexityMetrics>> {
    let output_stats_r1 = if inputs.execution.exit_code == 0 && inputs.output_reads.exists() {
        observe_fastq_stats(
            inputs.catalog,
            inputs.platform,
            inputs.bench_inputs.runner,
            inputs.output_reads,
        )?
    } else {
        inputs.bench_inputs.input_stats
    };
    let output_stats_r2 = if let Some(output_reads_r2) = inputs.output_reads_r2 {
        if inputs.execution.exit_code == 0 && output_reads_r2.exists() {
            Some(observe_fastq_stats(
                inputs.catalog,
                inputs.platform,
                inputs.bench_inputs.runner,
                output_reads_r2,
            )?)
        } else {
            inputs.input_stats_r2.copied()
        }
    } else {
        None
    };
    let before_stats =
        combine_seqkit_metrics(&inputs.bench_inputs.input_stats, inputs.input_stats_r2);
    let after_stats = combine_seqkit_metrics(&output_stats_r1, output_stats_r2.as_ref());
    let report = build_low_complexity_report(&LowComplexityReportInputs {
        tool: inputs.tool,
        threads: inputs.tool_spec.resources.threads,
        params: inputs.params,
        before_stats: &before_stats,
        after_stats: &after_stats,
        output_stats_r2: output_stats_r2.as_ref(),
        input_stats_r2: inputs.input_stats_r2,
        output_reads: inputs.output_reads,
        output_reads_r2: inputs.output_reads_r2,
        execution: inputs.execution,
    });
    let metrics = low_complexity_metrics_from_report(&report, &before_stats, &after_stats);
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    let out_dir = inputs
        .output_reads
        .parent()
        .ok_or_else(|| anyhow!("low-complexity output has no parent"))?;
    write_low_complexity_report(out_dir, &report)?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write low-complexity metrics")?;

    let context = build_benchmark_context(
        inputs.tool,
        inputs.tool_spec.tool_version.clone(),
        benchmark_image_identity(inputs.tool_spec),
        inputs.bench_inputs.runner,
        inputs.platform,
        inputs.input_hash.to_string(),
        inputs.params.clone(),
    );
    let record = BenchmarkRecord {
        context,
        execution: ExecutionMetrics {
            runtime_s: inputs.execution.runtime_s,
            memory_mb: inputs.execution.memory_mb,
            exit_code: inputs.execution.exit_code,
        },
        metrics: metric_set,
    };
    record.validate()?;
    Ok(record)
}

fn build_low_complexity_report(
    inputs: &LowComplexityReportInputs<'_>,
) -> FilterLowComplexityReportV1 {
    let raw_backend_report = inputs
        .params
        .get("raw_backend_report")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    let raw_backend_report_format = inputs
        .params
        .get("raw_backend_report_format")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    FilterLowComplexityReportV1 {
        schema_version: FILTER_LOW_COMPLEXITY_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_FILTER_LOW_COMPLEXITY.as_str().to_string(),
        stage_id: STAGE_FILTER_LOW_COMPLEXITY.as_str().to_string(),
        tool_id: inputs.tool.to_string(),
        paired_mode: if inputs.input_stats_r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        threads: inputs.threads,
        input_r1: inputs
            .params
            .get("input_r1")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        input_r2: inputs
            .params
            .get("input_r2")
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string),
        output_r1: inputs.output_reads.display().to_string(),
        output_r2: inputs.output_reads_r2.map(|path| path.display().to_string()),
        report_json: inputs
            .params
            .get("report_json")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("low_complexity_report.json")
            .to_string(),
        entropy_threshold: inputs
            .params
            .get("entropy_threshold")
            .and_then(serde_json::Value::as_f64),
        polyx_threshold: inputs
            .params
            .get("polyx_threshold")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok()),
        reads_in: inputs.before_stats.reads,
        reads_out: inputs.after_stats.reads,
        reads_removed_low_complexity: inputs
            .before_stats
            .reads
            .saturating_sub(inputs.after_stats.reads),
        bases_in: inputs.before_stats.bases,
        bases_out: inputs.after_stats.bases,
        pairs_in: inputs
            .input_stats_r2
            .map(|r2| inputs.before_stats.reads.saturating_sub(r2.reads).min(r2.reads)),
        pairs_out: inputs
            .output_stats_r2
            .map(|r2| inputs.after_stats.reads.saturating_sub(r2.reads).min(r2.reads)),
        mean_q_before: inputs.before_stats.mean_q,
        mean_q_after: inputs.after_stats.mean_q,
        runtime_s: Some(inputs.execution.runtime_s),
        memory_mb: Some(inputs.execution.memory_mb),
        exit_code: Some(inputs.execution.exit_code),
        raw_backend_report: raw_backend_report.clone(),
        raw_backend_report_format: raw_backend_report_format.clone(),
        backend_metrics: low_complexity_backend_metrics(
            raw_backend_report.as_deref(),
            raw_backend_report_format.as_deref(),
        ),
    }
}

fn low_complexity_metrics_from_report(
    report: &FilterLowComplexityReportV1,
    before_stats: &SeqkitMetrics,
    after_stats: &SeqkitMetrics,
) -> FastqLowComplexityMetrics {
    FastqLowComplexityMetrics {
        reads_in: report.reads_in,
        reads_out: report.reads_out,
        bases_in: report.bases_in,
        bases_out: report.bases_out,
        reads_removed_low_complexity: report.reads_removed_low_complexity,
        mean_q_before: report.mean_q_before,
        mean_q_after: report.mean_q_after,
        delta_metrics: derive_trim_delta(before_stats, after_stats),
    }
}

fn write_low_complexity_report(
    out_dir: &std::path::Path,
    report: &FilterLowComplexityReportV1,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(&out_dir.join("low_complexity_report.json"), report)
        .context("write low-complexity report")
}

fn low_complexity_backend_metrics(
    raw_backend_report: Option<&str>,
    raw_backend_report_format: Option<&str>,
) -> Option<serde_json::Value> {
    match (raw_backend_report, raw_backend_report_format) {
        (Some(path), Some("bbduk_stats")) => std::fs::read_to_string(path)
            .ok()
            .and_then(|raw| bijux_dna_domain_fastq::observer::parse_bbduk_reads_removed(&raw).ok())
            .map(|reads_removed_reported| {
                serde_json::json!({
                    "reads_removed_reported": reads_removed_reported,
                })
            }),
        _ => None,
    }
}

fn combine_seqkit_metrics(
    primary: &SeqkitMetrics,
    secondary: Option<&SeqkitMetrics>,
) -> SeqkitMetrics {
    let secondary_reads = secondary.map_or(0, |stats| stats.reads);
    let secondary_bases = secondary.map_or(0, |stats| stats.bases);
    let total_bases = primary.bases + secondary_bases;
    let weighted_mean_q = if total_bases == 0 {
        0.0
    } else {
        ((primary.mean_q * u64_to_f64(primary.bases))
            + secondary.map_or(0.0, |stats| stats.mean_q * u64_to_f64(stats.bases)))
            / u64_to_f64(total_bases)
    };
    let weighted_gc = if total_bases == 0 {
        0.0
    } else {
        ((primary.gc_percent * u64_to_f64(primary.bases))
            + secondary.map_or(0.0, |stats| stats.gc_percent * u64_to_f64(stats.bases)))
            / u64_to_f64(total_bases)
    };
    SeqkitMetrics {
        reads: primary.reads + secondary_reads,
        bases: total_bases,
        mean_q: weighted_mean_q,
        gc_percent: weighted_gc,
    }
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}
