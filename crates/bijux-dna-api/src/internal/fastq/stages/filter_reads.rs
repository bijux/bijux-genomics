use std::collections::HashMap;
use std::path::Path;

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::internal::fastq::stages::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context, derive_trim_delta, observe_fastq_stats,
    prepare_trim_bench,
};
use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_FILTER_READS,
};
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::quality::{fetch_fastq_filter_v2, insert_fastq_filter_v2};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqFilterMetrics};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::{FilterReadsReportV1, FILTER_READS_REPORT_SCHEMA_VERSION};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::select_filter_tools;
use bijux_dna_planner_fastq::stage_api::fastq::filter_reads::{plan_filter, FilterPlanOptions};
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use bijux_dna_stage_contract::StagePlanV1;

use crate::internal::fastq::stages::trim_bench_common::TrimBenchInputs;

fn apply_thread_override(
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    threads: Option<u32>,
) -> bijux_dna_core::prelude::ToolExecutionSpecV1 {
    let mut spec = tool_spec.clone();
    if let Some(threads) = threads {
        spec.resources.threads = threads.max(1);
    }
    spec
}

/// # Errors
/// Returns an error if planning or execution fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_filter<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqFilterArgs,
) -> Result<BenchOutcome<bijux_dna_analyze::FastqFilterMetrics>> {
    let selected_tools = select_filter_benchmark_tools(args)?;
    let setup =
        prepare_filter_benchmark_setup(catalog, platform, runner_override, args, &selected_tools)?;

    if args.explain {
        write_filter_benchmark_explain(&setup)?;
    }

    ensure_filter_benchmark_qa(catalog, platform, &setup.tools)?;

    let sqlite_path = setup.bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = setup.bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::<BenchmarkRecord<FastqFilterMetrics>>::new();
    for tool in &setup.tools {
        let tool_plan = prepare_filter_tool_plan(catalog, platform, args, &setup, jobs, tool)?;
        if let Ok(Some(record)) = fetch_fastq_filter_v2(
            &conn,
            tool,
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
        let execution = execute_filter_tool(&tool_plan, setup.bench_inputs.runner, jobs, tool)?;
        if let Some(failure) = filter_tool_failure(tool, execution.exit_code) {
            failures.push(failure);
            continue;
        }
        let record = build_filter_record(&FilterRecordInputs {
            catalog,
            platform,
            bench_inputs: &setup.bench_inputs,
            input_stats_r2: setup.input_stats_r2.as_ref(),
            tool,
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
        insert_fastq_filter_v2(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: setup.bench_inputs.bench_dir,
        explain: args.explain,
    })
}

struct FilterBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    bench_inputs: TrimBenchInputs,
    input_hash: String,
    input_stats_r2: Option<SeqkitMetrics>,
    options: FilterPlanOptions,
}

struct FilterToolPlan {
    tool_spec: ToolExecutionSpecV1,
    plan: StagePlanV1,
    params_hash: String,
    image_digest: String,
}

struct FilterRecordInputs<'a, S: ::std::hash::BuildHasher> {
    catalog: &'a HashMap<String, ToolImageSpec, S>,
    platform: &'a PlatformSpec,
    bench_inputs: &'a TrimBenchInputs,
    input_stats_r2: Option<&'a SeqkitMetrics>,
    tool: &'a str,
    tool_spec: &'a ToolExecutionSpecV1,
    input_hash: &'a str,
    params: &'a serde_json::Value,
    output_reads: &'a Path,
    output_reads_r2: Option<&'a Path>,
    execution: &'a StageResultV1,
}

struct FilterReadAccounting {
    reads_in: u64,
    reads_out: u64,
    reads_dropped: u64,
    bases_in: u64,
    bases_out: u64,
    pairs_in: Option<u64>,
    pairs_out: Option<u64>,
}

struct FilterObservedOutputs {
    r1: SeqkitMetrics,
    r2: Option<SeqkitMetrics>,
}

fn select_filter_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqFilterArgs,
) -> Result<Vec<String>> {
    let tools = select_filter_tools(&args.tools)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_FILTER_READS.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_FILTER_READS.as_str(), &header);
    Ok(tools)
}

fn prepare_filter_benchmark_setup<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqFilterArgs,
    selected_tools: &[String],
) -> Result<FilterBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools =
        filter_tools_by_role(STAGE_FILTER_READS.as_str(), selected_tools, &registry, false)?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_FILTER_READS,
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
    let options = FilterPlanOptions {
        threads: args.threads,
        max_n: args.max_n,
        max_n_fraction: args.max_n_fraction,
        max_n_count: args.max_n_count,
        low_complexity_threshold: args.low_complexity_threshold,
        entropy_threshold: args.entropy_threshold,
        kmer_ref: args.kmer_ref.clone(),
        redundant_filters: Vec::new(),
        polyx_policy: args.polyx_policy.clone(),
    };
    Ok(FilterBenchmarkSetup { registry, tools, bench_inputs, input_hash, input_stats_r2, options })
}

fn write_filter_benchmark_explain(setup: &FilterBenchmarkSetup) -> Result<()> {
    write_explain_md(
        &setup.bench_inputs.bench_dir,
        STAGE_FILTER_READS.as_str(),
        &setup.tools,
        &[],
        None,
    )?;
    write_explain_plan_json(
        &setup.bench_inputs.bench_dir,
        STAGE_FILTER_READS.as_str(),
        &setup.tools,
        &setup.registry,
        None,
    )
}

fn ensure_filter_benchmark_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(STAGE_FILTER_READS.as_str(), tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_FILTER_READS.as_str(), tools, platform, catalog)
}

fn prepare_filter_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqFilterArgs,
    setup: &FilterBenchmarkSetup,
    jobs: usize,
    tool: &str,
) -> Result<FilterToolPlan> {
    let out_dir = setup.bench_inputs.tools_root.join(tool);
    bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
    let tool_spec = build_tool_execution_spec(
        STAGE_FILTER_READS.as_str(),
        tool,
        &setup.registry,
        catalog,
        platform,
    )?;
    let tool_spec = apply_thread_override(&tool_spec, args.threads);
    let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
    let plan = plan_filter(&tool_spec, &args.r1, args.r2.as_deref(), &out_dir, &setup.options)?;
    let params_hash = stable_params_hash(&plan.params);
    let image_digest = tool_spec
        .image
        .digest
        .as_ref()
        .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
        .clone();
    Ok(FilterToolPlan { tool_spec, plan, params_hash, image_digest })
}

fn execute_filter_tool(
    tool_plan: &FilterToolPlan,
    runner: RuntimeKind,
    jobs: usize,
    tool: &str,
) -> Result<StageResultV1> {
    execute_plans_with_jobs(
        vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&tool_plan.plan)],
        runner,
        jobs,
    )?
    .into_iter()
    .next()
    .ok_or_else(|| anyhow!("missing execution result for {tool}"))
}

fn filter_tool_failure(tool: &str, exit_code: i32) -> Option<RawFailure> {
    (exit_code != 0).then(|| RawFailure {
        stage: STAGE_FILTER_READS.as_str().to_string(),
        tool: tool.to_string(),
        reason: format!("tool {tool} failed with status {exit_code}"),
        category: ErrorCategory::ToolError,
    })
}

#[allow(clippy::too_many_lines)]
fn build_filter_record<S: ::std::hash::BuildHasher>(
    inputs: &FilterRecordInputs<'_, S>,
) -> Result<BenchmarkRecord<FastqFilterMetrics>> {
    let platform = inputs.platform;
    let bench_inputs = inputs.bench_inputs;
    let input_stats_r2 = inputs.input_stats_r2;
    let tool = inputs.tool;
    let tool_spec = inputs.tool_spec;
    let input_hash = inputs.input_hash;
    let params = inputs.params;
    let output_reads = inputs.output_reads;
    let output_reads_r2 = inputs.output_reads_r2;
    let execution = inputs.execution;
    let output_stats = observe_filter_outputs(inputs)?;
    let output_stats_r1 = output_stats.r1;
    let output_stats_r2 = output_stats.r2;
    let accounting = filter_read_accounting(
        bench_inputs.input_stats,
        input_stats_r2,
        output_stats_r1,
        output_stats_r2.as_ref(),
    );
    let out_dir = output_reads.parent().ok_or_else(|| anyhow!("filter output has no parent"))?;
    let report_path = out_dir.join("filter_report.json");
    let raw_backend_report = params
        .get("raw_backend_report")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    let raw_backend_report_format = params
        .get("raw_backend_report_format")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    let backend_metrics = parse_filter_backend_metrics(
        raw_backend_report.as_deref().map(std::path::Path::new),
        raw_backend_report_format.as_deref(),
    );
    let removal_counts =
        derive_filter_removal_counts(backend_metrics.as_ref(), args_kmer_filter_requested(params));
    let report = FilterReadsReportV1 {
        schema_version: FILTER_READS_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_FILTER_READS.as_str().to_string(),
        stage_id: STAGE_FILTER_READS.as_str().to_string(),
        tool_id: tool.to_string(),
        paired_mode: if output_reads_r2.is_some() {
            bijux_dna_domain_fastq::params::PairedMode::PairedEnd
        } else {
            bijux_dna_domain_fastq::params::PairedMode::SingleEnd
        },
        threads: tool_spec.resources.threads,
        input_r1: params
            .get("input_r1")
            .and_then(serde_json::Value::as_str)
            .map_or_else(|| bench_inputs.r1.display().to_string(), ToString::to_string),
        input_r2: params
            .get("input_r2")
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string),
        output_r1: output_reads.display().to_string(),
        output_r2: output_reads_r2.map(|path| path.display().to_string()),
        report_json: report_path.display().to_string(),
        max_n: params
            .get("max_n")
            .and_then(serde_json::Value::as_u64)
            .and_then(|v| u32::try_from(v).ok()),
        max_n_fraction: params.get("max_n_fraction").and_then(serde_json::Value::as_f64),
        max_n_count: params
            .get("max_n_count")
            .and_then(serde_json::Value::as_u64)
            .and_then(|v| u32::try_from(v).ok()),
        low_complexity_threshold: params
            .get("low_complexity_threshold")
            .and_then(serde_json::Value::as_f64),
        entropy_threshold: params.get("entropy_threshold").and_then(serde_json::Value::as_f64),
        n_policy: Some("drop".to_string()),
        polyx_policy: params
            .get("polyx_policy")
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string),
        contaminant_db: params
            .get("kmer_ref")
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string),
        reads_in: accounting.reads_in,
        reads_out: accounting.reads_out,
        reads_dropped: accounting.reads_dropped,
        reads_removed_by_n: removal_counts.reads_removed_by_n,
        reads_removed_by_entropy: removal_counts.reads_removed_by_entropy,
        reads_removed_low_complexity: removal_counts.reads_removed_low_complexity,
        reads_removed_by_kmer: removal_counts.reads_removed_by_kmer,
        reads_removed_contaminant_kmer: removal_counts.reads_removed_contaminant_kmer,
        reads_removed_by_length: removal_counts.reads_removed_by_length,
        bases_in: accounting.bases_in,
        bases_out: accounting.bases_out,
        pairs_in: accounting.pairs_in,
        pairs_out: accounting.pairs_out,
        mean_q_before: bench_inputs.input_stats.mean_q,
        mean_q_after: output_stats_r1.mean_q,
        runtime_s: Some(execution.runtime_s),
        memory_mb: Some(execution.memory_mb),
        exit_code: Some(execution.exit_code),
        raw_backend_report,
        raw_backend_report_format,
        backend_metrics,
    };
    let metrics = FastqFilterMetrics {
        reads_in: report.reads_in,
        reads_out: report.reads_out,
        reads_dropped: report.reads_dropped,
        reads_removed_by_n: report.reads_removed_by_n,
        reads_removed_by_entropy: report.reads_removed_by_entropy,
        reads_removed_low_complexity: report.reads_removed_low_complexity,
        reads_removed_by_kmer: report.reads_removed_by_kmer,
        reads_removed_contaminant_kmer: report.reads_removed_contaminant_kmer,
        reads_removed_by_length: report.reads_removed_by_length,
        bases_in: report.bases_in,
        bases_out: report.bases_out,
        pairs_in: report.pairs_in,
        pairs_out: report.pairs_out,
        mean_q_before: report.mean_q_before,
        mean_q_after: report.mean_q_after,
        delta_metrics: derive_trim_delta(&bench_inputs.input_stats, &output_stats_r1),
    };
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    bijux_dna_infra::atomic_write_json(&report_path, &report).context("write filter report")?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write filter metrics")?;

    let context = build_benchmark_context(
        tool,
        tool_spec.tool_version.clone(),
        benchmark_image_identity(tool_spec),
        bench_inputs.runner,
        platform,
        input_hash.to_string(),
        params.clone(),
    );
    let record = BenchmarkRecord {
        context,
        execution: ExecutionMetrics {
            runtime_s: execution.runtime_s,
            memory_mb: execution.memory_mb,
            exit_code: execution.exit_code,
        },
        metrics: metric_set,
    };
    record.validate()?;
    Ok(record)
}

fn observe_filter_outputs<S: ::std::hash::BuildHasher>(
    inputs: &FilterRecordInputs<'_, S>,
) -> Result<FilterObservedOutputs> {
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

    Ok(FilterObservedOutputs { r1: output_stats_r1, r2: output_stats_r2 })
}

fn filter_read_accounting(
    input_stats_r1: SeqkitMetrics,
    input_stats_r2: Option<&SeqkitMetrics>,
    output_stats_r1: SeqkitMetrics,
    output_stats_r2: Option<&SeqkitMetrics>,
) -> FilterReadAccounting {
    let reads_in = input_stats_r1.reads + input_stats_r2.map_or(0, |stats| stats.reads);
    let reads_out = output_stats_r1.reads + output_stats_r2.map_or(0, |stats| stats.reads);
    let bases_in = input_stats_r1.bases + input_stats_r2.map_or(0, |stats| stats.bases);
    let bases_out = output_stats_r1.bases + output_stats_r2.map_or(0, |stats| stats.bases);
    let pairs_in = input_stats_r2.map(|stats| input_stats_r1.reads.min(stats.reads));
    let pairs_out = output_stats_r2.map(|stats| output_stats_r1.reads.min(stats.reads));

    FilterReadAccounting {
        reads_in,
        reads_out,
        reads_dropped: reads_in.saturating_sub(reads_out),
        bases_in,
        bases_out,
        pairs_in,
        pairs_out,
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(clippy::struct_field_names)]
struct FilterRemovalCounts {
    reads_removed_by_n: u64,
    reads_removed_by_entropy: u64,
    reads_removed_low_complexity: u64,
    reads_removed_by_kmer: u64,
    reads_removed_contaminant_kmer: u64,
    reads_removed_by_length: u64,
}

fn args_kmer_filter_requested(params: &serde_json::Value) -> bool {
    params
        .get("kmer_ref")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| !value.is_empty())
}

fn parse_filter_backend_metrics(
    raw_backend_report: Option<&std::path::Path>,
    raw_backend_report_format: Option<&str>,
) -> Option<serde_json::Value> {
    match (raw_backend_report, raw_backend_report_format) {
        (Some(path), Some("fastp_json")) => std::fs::read_to_string(path)
            .ok()
            .and_then(|raw| bijux_dna_domain_fastq::observer::parse_fastp_metrics(&raw).ok())
            .and_then(|metrics| serde_json::to_value(metrics).ok()),
        (Some(path), Some("bbduk_stats")) => std::fs::read_to_string(path)
            .ok()
            .and_then(|raw| bijux_dna_domain_fastq::observer::parse_bbduk_reads_removed(&raw).ok())
            .map(|reads_removed| {
                serde_json::json!({
                    "schema_version": "bijux.bbduk.filter.metrics.v1",
                    "reads_removed": reads_removed
                })
            }),
        _ => None,
    }
}

fn derive_filter_removal_counts(
    backend_metrics: Option<&serde_json::Value>,
    kmer_filter_requested: bool,
) -> FilterRemovalCounts {
    let mut counts = FilterRemovalCounts::default();
    let Some(metrics) = backend_metrics.and_then(serde_json::Value::as_object) else {
        return counts;
    };
    counts.reads_removed_by_n =
        metrics.get("too_many_n_reads").and_then(serde_json::Value::as_u64).unwrap_or(0);
    counts.reads_removed_by_length =
        metrics.get("too_short_reads").and_then(serde_json::Value::as_u64).unwrap_or(0);
    if kmer_filter_requested {
        let removed = metrics.get("reads_removed").and_then(serde_json::Value::as_u64).unwrap_or(0);
        counts.reads_removed_by_kmer = removed;
        counts.reads_removed_contaminant_kmer = removed;
    }
    counts
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::{derive_filter_removal_counts, parse_filter_backend_metrics};

    #[test]
    fn parse_filter_backend_metrics_reads_fastp_json() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("fastp.filter.json");
        bijux_dna_infra::write_bytes(
            &report_path,
            serde_json::json!({
                "filtering_result": {
                    "passed_filter_reads": 95_u64,
                    "low_quality_reads": 4_u64,
                    "too_many_N_reads": 2_u64,
                    "too_short_reads": 3_u64
                }
            })
            .to_string(),
        )
        .expect("write fastp report");

        let parsed =
            parse_filter_backend_metrics(Some(&report_path), Some("fastp_json")).expect("metrics");
        assert_eq!(parsed["passed_filter_reads"], serde_json::json!(95_u64));
        assert_eq!(parsed["too_many_n_reads"], serde_json::json!(2_u64));
        assert_eq!(parsed["too_short_reads"], serde_json::json!(3_u64));
    }

    #[test]
    fn derive_filter_removal_counts_maps_backend_specific_fields() {
        let counts = derive_filter_removal_counts(
            Some(&serde_json::json!({
                "too_many_n_reads": 2_u64,
                "too_short_reads": 3_u64,
                "reads_removed": 11_u64
            })),
            true,
        );
        assert_eq!(counts.reads_removed_by_n, 2);
        assert_eq!(counts.reads_removed_by_length, 3);
        assert_eq!(counts.reads_removed_by_kmer, 11);
        assert_eq!(counts.reads_removed_contaminant_kmer, 11);
    }
}
