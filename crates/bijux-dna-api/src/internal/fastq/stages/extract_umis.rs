use std::collections::HashMap;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{fetch_fastq_umi_v1, insert_fastq_umi_v1};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqUmiMetrics};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::measure::SeqkitMetrics;
use bijux_dna_core::prelude::params_hash;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::{ExtractUmisReportV1, PairedMode, EXTRACT_UMIS_REPORT_SCHEMA_VERSION};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::select_umi_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::extract_umis::plan_umi_with_options;
use bijux_dna_planner_fastq::stage_api::{
    ensure_umi_headers, inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind,
    RawFailure,
};
use bijux_dna_planner_fastq::ExtractUmisStageParams;
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use bijux_dna_stage_contract::StagePlanV1;
use uuid::Uuid;

use crate::internal::fastq::stages::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context, observe_fastq_stats,
};
use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_EXTRACT_UMIS,
};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;

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
pub fn bench_fastq_umi<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqUmiArgs,
) -> Result<BenchOutcome<bijux_dna_analyze::FastqUmiMetrics>> {
    let tools = select_umi_benchmark_tools(args)?;
    let r2 = args.r2.as_path();
    let setup = prepare_umi_benchmark_setup(catalog, platform, runner_override, args, &tools)?;

    if args.explain {
        write_umi_benchmark_explain(&setup)?;
    }

    ensure_umi_benchmark_qa(catalog, platform, args, &setup.tools)?;

    let sqlite_path = setup.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = setup.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::<BenchmarkRecord<FastqUmiMetrics>>::new();
    for tool in &setup.tools {
        let tool_plan = prepare_umi_tool_plan(catalog, platform, args, &setup, jobs, tool)?;
        if let Ok(Some(record)) = fetch_fastq_umi_v1(
            &conn,
            &tool_plan.tool,
            &tool_plan.tool_spec.tool_version,
            &tool_plan.image_digest,
            &setup.runner.to_string(),
            &platform.name,
            &setup.input_hash,
            &tool_plan.params_hash,
        ) {
            records.push(record);
            continue;
        }
        let execution = execute_umi_tool(&tool_plan, setup.runner, jobs)?;
        if let Some(failure) = umi_tool_failure(&tool_plan, execution.exit_code) {
            failures.push(failure);
            continue;
        }
        let record = build_umi_record(&UmiRecordInputs {
            catalog,
            platform,
            input_hash: &setup.input_hash,
            r1: &args.r1,
            r2,
            input_stats_r1: &setup.input_stats_r1,
            input_stats_r2: &setup.input_stats_r2,
            tool: &tool_plan.tool,
            tool_spec: &tool_plan.tool_spec,
            params: &tool_plan.plan.params,
            out_dir: &tool_plan.plan.out_dir,
            execution: &execution,
        })?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_umi_v1(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome { records, failures, bench_dir: setup.bench_dir, explain: args.explain })
}

fn select_umi_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqUmiArgs,
) -> Result<Vec<String>> {
    let tools = select_umi_tools(&args.tools)?;
    let r2 = args.r2.as_path();
    preflight_stage(STAGE_EXTRACT_UMIS.as_str(), FastqArtifactKind::PairedEnd)?;
    let header = inspect_headers(&args.r1, Some(r2), false)?;
    log_header_warnings(STAGE_EXTRACT_UMIS.as_str(), &header);
    Ok(tools)
}

struct UmiBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    bench_dir: std::path::PathBuf,
    tools_root: std::path::PathBuf,
    input_hash: String,
    runner: RuntimeKind,
    input_stats_r1: SeqkitMetrics,
    input_stats_r2: SeqkitMetrics,
}

struct UmiToolPlan {
    tool: String,
    tool_spec: ToolExecutionSpecV1,
    plan: StagePlanV1,
    params_hash: String,
    image_digest: String,
}

struct UmiRecordInputs<'a, S: ::std::hash::BuildHasher> {
    catalog: &'a HashMap<String, ToolImageSpec, S>,
    platform: &'a PlatformSpec,
    input_hash: &'a str,
    r1: &'a std::path::Path,
    r2: &'a std::path::Path,
    input_stats_r1: &'a SeqkitMetrics,
    input_stats_r2: &'a SeqkitMetrics,
    tool: &'a str,
    tool_spec: &'a ToolExecutionSpecV1,
    params: &'a serde_json::Value,
    out_dir: &'a std::path::Path,
    execution: &'a StageResultV1,
}

struct UmiReportInputs<'a> {
    tool: &'a str,
    threads: u32,
    params: &'a serde_json::Value,
    r1: &'a std::path::Path,
    r2: &'a std::path::Path,
    output_r1: &'a std::path::Path,
    output_r2: &'a std::path::Path,
    input_stats_r1: &'a SeqkitMetrics,
    input_stats_r2: &'a SeqkitMetrics,
    output_stats_r1: &'a SeqkitMetrics,
    output_stats_r2: &'a SeqkitMetrics,
    execution: &'a StageResultV1,
}

fn prepare_umi_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqUmiArgs,
    setup: &UmiBenchmarkSetup,
    jobs: usize,
    tool: &str,
) -> Result<UmiToolPlan> {
    let out_dir = setup.tools_root.join(tool);
    bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
    let tool_spec = build_tool_execution_spec(
        STAGE_EXTRACT_UMIS.as_str(),
        tool,
        &setup.registry,
        catalog,
        platform,
    )?;
    let tool_spec = apply_thread_override(&tool_spec, args.threads);
    let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
    let plan = plan_umi_with_options(
        &tool_spec,
        &args.r1,
        args.r2.as_path(),
        &out_dir,
        &ExtractUmisStageParams {
            threads: args.threads,
            umi_pattern: Some(args.umi_pattern.clone()),
        },
    )?;
    let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
    let image_digest = benchmark_image_identity(&tool_spec);
    Ok(UmiToolPlan { tool: tool.to_string(), tool_spec, plan, params_hash, image_digest })
}

fn execute_umi_tool(
    tool_plan: &UmiToolPlan,
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

fn umi_tool_failure(tool_plan: &UmiToolPlan, exit_code: i32) -> Option<RawFailure> {
    if exit_code == 0 {
        return None;
    }
    Some(RawFailure {
        stage: STAGE_EXTRACT_UMIS.as_str().to_string(),
        tool: tool_plan.tool.clone(),
        reason: format!("tool {} failed with status {exit_code}", tool_plan.tool),
        category: ErrorCategory::ToolError,
    })
}

fn prepare_umi_benchmark_setup<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqUmiArgs,
    tools: &[String],
) -> Result<UmiBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_EXTRACT_UMIS.as_str(), tools, &registry, false)?;
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir_name = bench_dir_name(&STAGE_EXTRACT_UMIS)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_EXTRACT_UMIS.as_str()))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_dna_infra::ensure_dir(&tools_root).context("create tools output dir")?;
    let r2 = args.r2.as_path();
    let input_hash = format!(
        "{}+{}",
        hash_file_sha256(&args.r1).context("hash umi input r1")?,
        hash_file_sha256(r2).context("hash umi input r2")?
    );
    let input_stats_r1 = observe_fastq_stats(catalog, platform, runner, &args.r1)?;
    let input_stats_r2 = observe_fastq_stats(catalog, platform, runner, r2)?;
    Ok(UmiBenchmarkSetup {
        registry,
        tools,
        bench_dir,
        tools_root,
        input_hash,
        runner,
        input_stats_r1,
        input_stats_r2,
    })
}

fn write_umi_benchmark_explain(setup: &UmiBenchmarkSetup) -> Result<()> {
    write_explain_md(&setup.bench_dir, STAGE_EXTRACT_UMIS.as_str(), &setup.tools, &[], None)?;
    write_explain_plan_json(
        &setup.bench_dir,
        STAGE_EXTRACT_UMIS.as_str(),
        &setup.tools,
        &setup.registry,
        None,
    )
}

fn ensure_umi_benchmark_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqUmiArgs,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(STAGE_EXTRACT_UMIS.as_str(), tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_EXTRACT_UMIS.as_str(), tools, platform, catalog)?;
    ensure_umi_headers(&args.r1, Some(args.r2.as_path()))
}

fn build_umi_record<S: ::std::hash::BuildHasher>(
    inputs: &UmiRecordInputs<'_, S>,
) -> Result<BenchmarkRecord<FastqUmiMetrics>> {
    let output_r1 = inputs.out_dir.join("umi_tools.r1.fastq.gz");
    let output_r2 = inputs.out_dir.join("umi_tools.r2.fastq.gz");
    let output_stats_r1 = if inputs.execution.exit_code == 0 && output_r1.exists() {
        observe_fastq_stats(inputs.catalog, inputs.platform, inputs.platform.runner, &output_r1)?
    } else {
        *inputs.input_stats_r1
    };
    let output_stats_r2 = if inputs.execution.exit_code == 0 && output_r2.exists() {
        observe_fastq_stats(inputs.catalog, inputs.platform, inputs.platform.runner, &output_r2)?
    } else {
        *inputs.input_stats_r2
    };
    let report = build_umi_report(&UmiReportInputs {
        tool: inputs.tool,
        threads: inputs.tool_spec.resources.threads,
        params: inputs.params,
        r1: inputs.r1,
        r2: inputs.r2,
        output_r1: &output_r1,
        output_r2: &output_r2,
        input_stats_r1: inputs.input_stats_r1,
        input_stats_r2: inputs.input_stats_r2,
        output_stats_r1: &output_stats_r1,
        output_stats_r2: &output_stats_r2,
        execution: inputs.execution,
    });
    let metrics = umi_metrics_from_report(&report);
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    bijux_dna_infra::atomic_write_json(&inputs.out_dir.join("umi_report.json"), &report)
        .context("write umi report")?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&inputs.out_dir.join("metrics.json"), &metrics_json)
        .context("write umi metrics")?;

    let context = build_benchmark_context(
        inputs.tool,
        inputs.tool_spec.tool_version.clone(),
        benchmark_image_identity(inputs.tool_spec),
        inputs.platform.runner,
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

fn build_umi_report(inputs: &UmiReportInputs<'_>) -> ExtractUmisReportV1 {
    let reads_in = inputs.input_stats_r1.reads + inputs.input_stats_r2.reads;
    let reads_out = inputs.output_stats_r1.reads + inputs.output_stats_r2.reads;
    let bases_in = inputs.input_stats_r1.bases + inputs.input_stats_r2.bases;
    let bases_out = inputs.output_stats_r1.bases + inputs.output_stats_r2.bases;
    let pairs_in = Some(inputs.input_stats_r1.reads.min(inputs.input_stats_r2.reads));
    let pairs_out = Some(inputs.output_stats_r1.reads.min(inputs.output_stats_r2.reads));
    let reads_with_umi = reads_out;
    let raw_backend_report = inputs
        .params
        .get("raw_backend_report")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    ExtractUmisReportV1 {
        schema_version: EXTRACT_UMIS_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_EXTRACT_UMIS.as_str().to_string(),
        stage_id: STAGE_EXTRACT_UMIS.as_str().to_string(),
        tool_id: inputs.tool.to_string(),
        paired_mode: PairedMode::PairedEnd,
        threads: inputs.threads,
        umi_pattern: inputs
            .params
            .get("umi_pattern")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("NNNNNNNN")
            .to_string(),
        input_r1: inputs.r1.display().to_string(),
        input_r2: Some(inputs.r2.display().to_string()),
        output_r1: inputs.output_r1.display().to_string(),
        output_r2: Some(inputs.output_r2.display().to_string()),
        report_json: inputs
            .params
            .get("report_json")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("umi_report.json")
            .to_string(),
        reads_in,
        reads_out,
        bases_in,
        bases_out,
        pairs_in,
        pairs_out,
        reads_with_umi,
        mean_q_before: weighted_mean_q(inputs.input_stats_r1, inputs.input_stats_r2),
        mean_q_after: weighted_mean_q(inputs.output_stats_r1, inputs.output_stats_r2),
        runtime_s: Some(inputs.execution.runtime_s),
        memory_mb: Some(inputs.execution.memory_mb),
        exit_code: Some(inputs.execution.exit_code),
        raw_backend_report: raw_backend_report.clone(),
        raw_backend_report_format: inputs
            .params
            .get("raw_backend_report_format")
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string),
        backend_metrics: Some(serde_json::json!({
            "reads_with_umi_fraction": if reads_in == 0 { 0.0 } else { u64_to_f64(reads_with_umi) / u64_to_f64(reads_in) },
            "raw_backend_report_present": raw_backend_report.is_some(),
        })),
    }
}

fn umi_metrics_from_report(report: &ExtractUmisReportV1) -> FastqUmiMetrics {
    FastqUmiMetrics {
        reads_in: report.reads_in,
        reads_out: report.reads_out,
        bases_in: report.bases_in,
        bases_out: report.bases_out,
        pairs_in: report.pairs_in,
        pairs_out: report.pairs_out,
        reads_with_umi: report.reads_with_umi,
    }
}

fn weighted_mean_q(
    r1: &bijux_dna_core::prelude::measure::SeqkitMetrics,
    r2: &bijux_dna_core::prelude::measure::SeqkitMetrics,
) -> f64 {
    let total_bases = r1.bases + r2.bases;
    if total_bases == 0 {
        0.0
    } else {
        ((r1.mean_q * u64_to_f64(r1.bases)) + (r2.mean_q * u64_to_f64(r2.bases)))
            / u64_to_f64(total_bases)
    }
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}
