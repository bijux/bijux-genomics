use std::collections::HashMap;
use std::path::PathBuf;

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::internal::fastq::stages::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context, observe_fastq_stats, prepare_trim_bench,
    TrimBenchInputs,
};
use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_DETECT_ADAPTERS,
};
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_detect_adapters_v1, insert_fastq_detect_adapters_v1,
};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkRecord, FastqDetectAdaptersMetrics, MetricSet,
};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::{
    params::{
        detect_adapters::{AdapterEvidenceFormat, AdapterEvidenceScope, AdapterInspectionMode},
        PairedMode,
    },
    DetectAdaptersReportV1, DETECT_ADAPTERS_REPORT_SCHEMA_VERSION,
};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::select_detect_adapters_tools;
use bijux_dna_planner_fastq::stage_api::fastq::detect_adapters::plan;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use bijux_dna_stage_contract::StagePlanV1;

/// # Errors
/// Returns an error if planning, execution, report parsing, or persistence fails.
pub fn bench_fastq_detect_adapters<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDetectAdaptersArgs,
) -> Result<BenchOutcome<FastqDetectAdaptersMetrics>> {
    let tools = select_detect_adapters_benchmark_tools(args)?;
    let setup =
        prepare_detect_adapters_benchmark_setup(catalog, platform, runner_override, args, &tools)?;

    if args.explain {
        write_detect_adapters_benchmark_explain(&setup)?;
    }

    ensure_detect_adapters_benchmark_qa(catalog, platform, &setup.tools)?;

    let store = DetectAdaptersBenchmarkStore::from_bench_inputs(&setup.bench_inputs);
    let conn = bijux_dna_analyze::open_sqlite(&store.sqlite_path).context("open bench sqlite")?;
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::<BenchmarkRecord<FastqDetectAdaptersMetrics>>::new();
    for tool in &setup.tools {
        let tool_plan =
            prepare_detect_adapters_tool_plan(catalog, platform, args, &setup, jobs, tool)?;
        let cache_identity =
            DetectAdaptersCacheIdentity::from_tool_plan(&setup, platform, &tool_plan);
        if let Ok(Some(record)) = fetch_fastq_detect_adapters_v1(
            &conn,
            &cache_identity.tool,
            &cache_identity.tool_version,
            &cache_identity.image_digest,
            &cache_identity.runner,
            &cache_identity.platform,
            &cache_identity.input_hash,
            &cache_identity.params_hash,
        ) {
            records.push(record);
            continue;
        }
        let execution = execute_detect_adapters_tool(&tool_plan, setup.bench_inputs.runner, jobs)?;
        if let Some(failure) = detect_adapters_tool_failure(&tool_plan, execution.result.exit_code)
        {
            failures.push(failure);
            continue;
        }
        let record = build_detect_record(&DetectRecordInputs {
            platform,
            bench_inputs: &setup.bench_inputs,
            input_r2_path: args.r2.as_deref(),
            input_stats_r2: setup.input_stats_r2.as_ref(),
            input_hash: &setup.input_hash,
            tool_plan: &tool_plan,
            execution: &execution,
        })?;
        append_jsonl(&store.jsonl_path, &record).context("write bench.jsonl")?;
        insert_fastq_detect_adapters_v1(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: setup.bench_inputs.bench_dir,
        explain: args.explain,
    })
}

fn select_detect_adapters_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDetectAdaptersArgs,
) -> Result<Vec<String>> {
    let tools = select_detect_adapters_tools(&args.tools)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_DETECT_ADAPTERS.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_DETECT_ADAPTERS.as_str(), &header);
    Ok(tools)
}

struct DetectAdaptersBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    bench_inputs: TrimBenchInputs,
    input_hash: String,
    input_stats_r2: Option<SeqkitMetrics>,
}

struct DetectAdaptersBenchmarkStore {
    sqlite_path: PathBuf,
    jsonl_path: PathBuf,
}

impl DetectAdaptersBenchmarkStore {
    fn from_bench_inputs(bench_inputs: &TrimBenchInputs) -> Self {
        Self {
            sqlite_path: bench_inputs.bench_dir.join("bench.sqlite"),
            jsonl_path: bench_inputs.bench_dir.join("bench.jsonl"),
        }
    }
}

struct DetectAdaptersToolPlan {
    tool: String,
    tool_spec: ToolExecutionSpecV1,
    plan: StagePlanV1,
    params_hash: String,
    image_digest: String,
}

struct DetectAdaptersToolExecution {
    result: StageResultV1,
}

struct DetectAdaptersCacheIdentity {
    tool: String,
    tool_version: String,
    image_digest: String,
    runner: String,
    platform: String,
    input_hash: String,
    params_hash: String,
}

impl DetectAdaptersCacheIdentity {
    fn from_tool_plan(
        setup: &DetectAdaptersBenchmarkSetup,
        platform: &PlatformSpec,
        tool_plan: &DetectAdaptersToolPlan,
    ) -> Self {
        Self {
            tool: tool_plan.tool.clone(),
            tool_version: tool_plan.tool_spec.tool_version.clone(),
            image_digest: tool_plan.image_digest.clone(),
            runner: setup.bench_inputs.runner.to_string(),
            platform: platform.name.clone(),
            input_hash: setup.input_hash.clone(),
            params_hash: tool_plan.params_hash.clone(),
        }
    }
}

struct DetectRecordInputs<'a> {
    platform: &'a PlatformSpec,
    bench_inputs: &'a TrimBenchInputs,
    input_r2_path: Option<&'a std::path::Path>,
    input_stats_r2: Option<&'a SeqkitMetrics>,
    input_hash: &'a str,
    tool_plan: &'a DetectAdaptersToolPlan,
    execution: &'a DetectAdaptersToolExecution,
}

fn prepare_detect_adapters_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDetectAdaptersArgs,
    setup: &DetectAdaptersBenchmarkSetup,
    jobs: usize,
    tool: &str,
) -> Result<DetectAdaptersToolPlan> {
    let out_dir = setup.bench_inputs.tools_root.join(tool);
    bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
    let mut tool_spec = build_tool_execution_spec(
        STAGE_DETECT_ADAPTERS.as_str(),
        tool,
        &setup.registry,
        catalog,
        platform,
    )?;
    if let Some(threads) = args.threads {
        tool_spec.resources.threads = threads;
    }
    let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
    let plan = plan(&tool_spec, &setup.bench_inputs.r1, args.r2.as_deref(), &out_dir)?;
    let params_hash = stable_params_hash(&plan.params);
    let image_digest = benchmark_image_identity(&tool_spec);
    Ok(DetectAdaptersToolPlan {
        tool: tool.to_string(),
        tool_spec,
        plan,
        params_hash,
        image_digest,
    })
}

fn execute_detect_adapters_tool(
    tool_plan: &DetectAdaptersToolPlan,
    runner: RuntimeKind,
    jobs: usize,
) -> Result<DetectAdaptersToolExecution> {
    let result = execute_plans_with_jobs(
        vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&tool_plan.plan)],
        runner,
        jobs,
    )?
    .into_iter()
    .next()
    .ok_or_else(|| anyhow!("missing execution result for {}", tool_plan.tool))?;
    Ok(DetectAdaptersToolExecution { result })
}

fn detect_adapters_tool_failure(
    tool_plan: &DetectAdaptersToolPlan,
    exit_code: i32,
) -> Option<RawFailure> {
    if exit_code == 0 {
        return None;
    }
    Some(RawFailure {
        stage: STAGE_DETECT_ADAPTERS.as_str().to_string(),
        tool: tool_plan.tool.clone(),
        reason: format!("tool {} failed with status {exit_code}", tool_plan.tool),
        category: ErrorCategory::ToolError,
    })
}

fn prepare_detect_adapters_benchmark_setup<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDetectAdaptersArgs,
    tools: &[String],
) -> Result<DetectAdaptersBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_DETECT_ADAPTERS.as_str(), tools, &registry, false)?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_DETECT_ADAPTERS,
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
    Ok(DetectAdaptersBenchmarkSetup { registry, tools, bench_inputs, input_hash, input_stats_r2 })
}

fn write_detect_adapters_benchmark_explain(setup: &DetectAdaptersBenchmarkSetup) -> Result<()> {
    write_explain_md(
        &setup.bench_inputs.bench_dir,
        STAGE_DETECT_ADAPTERS.as_str(),
        &setup.tools,
        &[],
        None,
    )?;
    write_explain_plan_json(
        &setup.bench_inputs.bench_dir,
        STAGE_DETECT_ADAPTERS.as_str(),
        &setup.tools,
        &setup.registry,
        None,
    )
}

fn ensure_detect_adapters_benchmark_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(STAGE_DETECT_ADAPTERS.as_str(), tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_DETECT_ADAPTERS.as_str(), tools, platform, catalog)
}

fn build_detect_record(
    inputs: &DetectRecordInputs<'_>,
) -> Result<BenchmarkRecord<FastqDetectAdaptersMetrics>> {
    let report = build_detect_report(
        inputs.bench_inputs,
        inputs.input_r2_path,
        inputs.input_stats_r2,
        &inputs.tool_plan.tool,
        &inputs.tool_plan.tool_spec,
        &inputs.tool_plan.plan.out_dir,
        &inputs.execution.result,
    )?;
    let metric_set = build_detect_adapters_metric_set(&report)?;

    bijux_dna_infra::atomic_write_json(
        &inputs.tool_plan.plan.out_dir.join("adapter_report.json"),
        &report,
    )
    .context("write adapter report")?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(
        &inputs.tool_plan.plan.out_dir.join("metrics.json"),
        &metrics_json,
    )
    .context("write adapter metrics")?;

    let context = build_benchmark_context(
        &inputs.tool_plan.tool,
        inputs.tool_plan.tool_spec.tool_version.clone(),
        inputs.tool_plan.image_digest.clone(),
        inputs.bench_inputs.runner,
        inputs.platform,
        inputs.input_hash.to_string(),
        inputs.tool_plan.plan.params.clone(),
    );
    let record = BenchmarkRecord {
        context,
        execution: ExecutionMetrics {
            runtime_s: inputs.execution.result.runtime_s,
            memory_mb: inputs.execution.result.memory_mb,
            exit_code: inputs.execution.result.exit_code,
        },
        metrics: metric_set,
    };
    record.validate()?;
    Ok(record)
}

fn build_detect_adapters_metric_set(
    report: &DetectAdaptersReportV1,
) -> Result<MetricSet<FastqDetectAdaptersMetrics>> {
    let metrics = FastqDetectAdaptersMetrics {
        reads_in: report.reads_in,
        reads_out: report.reads_out,
        bases_in: report.bases_in,
        bases_out: report.bases_out,
        mean_q: report.mean_q,
        candidate_adapter_count: report.candidate_adapter_count,
        adapter_trimmed_fraction: report.adapter_trimmed_fraction,
    };
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;
    Ok(metric_set)
}

fn build_detect_report(
    bench_inputs: &crate::internal::fastq::stages::trim_bench_common::TrimBenchInputs,
    input_r2_path: Option<&std::path::Path>,
    input_stats_r2: Option<&SeqkitMetrics>,
    tool: &str,
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    out_dir: &std::path::Path,
    execution: &StageResultV1,
) -> Result<DetectAdaptersReportV1> {
    let (candidate_adapter_count, adapter_trimmed_fraction) = detect_adapter_summary(out_dir)?;
    let reads_in = bench_inputs.input_stats.reads + input_stats_r2.map_or(0, |stats| stats.reads);
    let bases_in = bench_inputs.input_stats.bases + input_stats_r2.map_or(0, |stats| stats.bases);
    let mean_q = if bases_in == 0 {
        0.0
    } else {
        ((bench_inputs.input_stats.mean_q * u64_to_f64(bench_inputs.input_stats.bases))
            + input_stats_r2.map_or(0.0, |stats| stats.mean_q * u64_to_f64(stats.bases)))
            / u64_to_f64(bases_in)
    };
    let pairs_in = input_stats_r2.map(|stats| bench_inputs.input_stats.reads.min(stats.reads));
    let pairs_out = pairs_in;
    Ok(DetectAdaptersReportV1 {
        schema_version: DETECT_ADAPTERS_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_DETECT_ADAPTERS.as_str().to_string(),
        stage_id: STAGE_DETECT_ADAPTERS.as_str().to_string(),
        tool_id: tool.to_string(),
        paired_mode: if input_stats_r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        threads: tool_spec.resources.threads,
        inspection_mode: AdapterInspectionMode::EvidenceOnly,
        report_only: true,
        evidence_engine: tool.to_string(),
        evidence_scope: AdapterEvidenceScope::FullInput,
        evidence_format: AdapterEvidenceFormat::FastqcSummary,
        evidence_artifact_id: "report_json".to_string(),
        input_r1: bench_inputs.r1.display().to_string(),
        input_r2: input_r2_path.map(|path| path.display().to_string()),
        report_json: out_dir.join("adapter_report.json").display().to_string(),
        adapter_evidence_dir: out_dir.join("fastqc").display().to_string(),
        reads_in,
        reads_out: reads_in,
        bases_in,
        bases_out: bases_in,
        pairs_in,
        pairs_out,
        mean_q,
        candidate_adapter_count,
        adapter_trimmed_fraction,
        adapter_content_max: None,
        adapter_content_mean: None,
        duplication_rate: None,
        n_rate: None,
        kmer_warning_count: None,
        overrepresented_sequence_count: None,
        runtime_s: Some(execution.runtime_s),
        memory_mb: Some(execution.memory_mb),
        exit_code: Some(execution.exit_code),
        raw_backend_report: None,
        raw_backend_report_format: None,
    })
}

fn detect_adapter_summary(out_dir: &std::path::Path) -> Result<(u64, Option<f64>)> {
    let fastp_json = out_dir.join("fastp.json");
    if fastp_json.exists() {
        let raw = std::fs::read_to_string(&fastp_json)
            .with_context(|| format!("read {}", fastp_json.display()))?;
        let parsed: serde_json::Value = serde_json::from_str(&raw)
            .with_context(|| format!("parse {}", fastp_json.display()))?;
        let adapter_trimmed_reads = parsed
            .pointer("/adapter_cutting/adapter_trimmed_reads")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let total_reads = parsed
            .pointer("/summary/before_filtering/total_reads")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let fraction = if total_reads > 0 {
            Some(u64_to_f64(adapter_trimmed_reads) / u64_to_f64(total_reads))
        } else {
            None
        };
        let count = u64::from(adapter_trimmed_reads > 0);
        return Ok((count, fraction));
    }
    Ok((u64::from(out_dir.join("fastqc").exists()), None))
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}
