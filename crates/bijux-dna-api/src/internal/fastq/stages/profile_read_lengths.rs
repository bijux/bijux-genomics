use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_read_lengths_v1, insert_fastq_read_lengths_v1,
};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkRecord, FastqReadLengthMetrics, MetricSet, StageMetricSchema,
};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::{
    PairedMode, ProfileReadLengthBinV1, ProfileReadLengthsReportV1,
    PROFILE_READ_LENGTHS_REPORT_SCHEMA_VERSION,
};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::stage_api::{
    bench_dir_name, inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind,
    RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use bijux_dna_stage_contract::StagePlanV1;

use crate::internal::fastq::stages::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context,
};
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};

const STAGE_ID: &str = "fastq.profile_read_lengths";

/// Benchmark FASTQ read-length profiling tools under governed contracts.
///
/// # Errors
/// Returns an error if planning, execution, profile parsing, or persistence fails.
pub fn bench_fastq_profile_read_lengths<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileReadLengthsArgs,
) -> Result<BenchOutcome<FastqReadLengthMetrics>> {
    let selected_tools = select_read_lengths_benchmark_tools(args)?;
    let setup =
        prepare_read_lengths_benchmark_setup(platform, runner_override, args, &selected_tools)?;

    if args.explain {
        write_read_lengths_benchmark_explain(&setup)?;
    }

    ensure_read_lengths_benchmark_qa(catalog, platform, &setup.tools)?;

    let store = ReadLengthsBenchmarkStore::from_setup(&setup);
    let conn = bijux_dna_analyze::open_sqlite(&store.sqlite_path)?;
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::new();

    for tool in &setup.tools {
        let tool_plan = prepare_read_lengths_tool_plan(catalog, platform, args, &setup, tool)?;
        let cache_identity = ReadLengthsCacheIdentity::from_plan(platform, &setup, &tool_plan);
        if let Ok(Some(record)) = fetch_fastq_read_lengths_v1(
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

        let execution = execute_read_lengths_tool(&tool_plan, setup.runner, jobs)?;
        if let Some(failure) = read_lengths_tool_failure(&tool_plan, &execution) {
            failures.push(failure);
            continue;
        }

        let observation = observe_read_lengths_tool(args, &tool_plan.plan)?;
        let metric_set = build_read_lengths_metric_set(&observation)?;
        let histogram = project_read_lengths_histogram(&observation);
        let report = build_read_lengths_report(ReadLengthsReportInputs {
            tool,
            args,
            artifacts: &observation.artifacts,
            metrics: &metric_set.metrics,
            histogram,
            threads: tool_plan.plan.resources.threads,
            execution_metrics: read_lengths_execution_metrics(&execution),
        });
        write_read_lengths_artifacts(&tool_plan, &observation, &report, &metric_set)?;
        let record = build_read_lengths_record(
            &ReadLengthsRecordInputs {
                platform,
                setup: &setup,
                tool,
                tool_plan: &tool_plan,
                execution: &execution,
            },
            metric_set,
        )?;
        append_jsonl(&store.jsonl_path, &record)?;
        insert_fastq_read_lengths_v1(&conn, &record)?;
        records.push(record);
    }

    Ok(BenchOutcome { records, failures, bench_dir: setup.bench_dir, explain: args.explain })
}

struct ReadLengthsBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    runner: RuntimeKind,
    bench_dir: PathBuf,
    tools_root: PathBuf,
    input_hash: String,
}

struct ReadLengthsBenchmarkStore {
    sqlite_path: PathBuf,
    jsonl_path: PathBuf,
}

impl ReadLengthsBenchmarkStore {
    fn from_setup(setup: &ReadLengthsBenchmarkSetup) -> Self {
        Self {
            sqlite_path: setup.bench_dir.join("bench.sqlite"),
            jsonl_path: setup.bench_dir.join("bench.jsonl"),
        }
    }
}

struct ReadLengthsToolPlan {
    tool: String,
    out_dir: PathBuf,
    tool_spec: ToolExecutionSpecV1,
    plan: StagePlanV1,
    params_hash: String,
    image_digest: String,
}

struct ReadLengthsToolExecution {
    result: StageResultV1,
}

struct ReadLengthsCacheIdentity {
    tool: String,
    tool_version: String,
    image_digest: String,
    runner: String,
    platform: String,
    input_hash: String,
    params_hash: String,
}

impl ReadLengthsCacheIdentity {
    fn from_plan(
        platform: &PlatformSpec,
        setup: &ReadLengthsBenchmarkSetup,
        tool_plan: &ReadLengthsToolPlan,
    ) -> Self {
        Self {
            tool: tool_plan.tool.clone(),
            tool_version: tool_plan.tool_spec.tool_version.clone(),
            image_digest: tool_plan.image_digest.clone(),
            runner: setup.runner.to_string(),
            platform: platform.name.clone(),
            input_hash: setup.input_hash.clone(),
            params_hash: tool_plan.params_hash.clone(),
        }
    }
}

struct ReadLengthsArtifacts {
    report_json: PathBuf,
    length_tsv: PathBuf,
    length_json: PathBuf,
    histogram_bins: u32,
}

struct ReadLengthsObservation {
    lengths: Vec<usize>,
    artifacts: ReadLengthsArtifacts,
}

struct ReadLengthsReportInputs<'a> {
    tool: &'a str,
    args: &'a bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileReadLengthsArgs,
    artifacts: &'a ReadLengthsArtifacts,
    metrics: &'a FastqReadLengthMetrics,
    histogram: Vec<ProfileReadLengthBinV1>,
    threads: u32,
    execution_metrics: ExecutionMetrics,
}

struct ReadLengthsRecordInputs<'a> {
    platform: &'a PlatformSpec,
    setup: &'a ReadLengthsBenchmarkSetup,
    tool: &'a str,
    tool_plan: &'a ReadLengthsToolPlan,
    execution: &'a ReadLengthsToolExecution,
}

fn select_read_lengths_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileReadLengthsArgs,
) -> Result<Vec<String>> {
    let tools = bijux_dna_planner_fastq::select_profile_read_lengths_tools(&args.tools)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_ID, artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_ID, &header);
    Ok(tools)
}

fn read_lengths_input_hash(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileReadLengthsArgs,
) -> Result<String> {
    if let Some(r2) = args.r2.as_deref() {
        return Ok(format!(
            "{}+{}",
            hash_file_sha256(&args.r1).context("hash read-length input r1")?,
            hash_file_sha256(r2).context("hash read-length input r2")?
        ));
    }
    hash_file_sha256(&args.r1).context("hash read-length input")
}

fn prepare_read_lengths_benchmark_setup(
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileReadLengthsArgs,
    selected_tools: &[String],
) -> Result<ReadLengthsBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_ID, selected_tools, &registry, false)?;
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir_name =
        bench_dir_name(&bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_READ_LENGTHS)
            .ok_or_else(|| anyhow!("bench dir missing for {STAGE_ID}"))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir)?;
    bijux_dna_infra::ensure_dir(&tools_root)?;
    let input_hash = read_lengths_input_hash(args)?;
    Ok(ReadLengthsBenchmarkSetup { registry, tools, runner, bench_dir, tools_root, input_hash })
}

fn write_read_lengths_benchmark_explain(setup: &ReadLengthsBenchmarkSetup) -> Result<()> {
    write_explain_md(&setup.bench_dir, STAGE_ID, &setup.tools, &[], None)?;
    write_explain_plan_json(&setup.bench_dir, STAGE_ID, &setup.tools, &setup.registry, None)
}

fn ensure_read_lengths_benchmark_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(STAGE_ID, tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_ID, tools, platform, catalog)
}

fn prepare_read_lengths_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileReadLengthsArgs,
    setup: &ReadLengthsBenchmarkSetup,
    tool: &str,
) -> Result<ReadLengthsToolPlan> {
    let out_dir = setup.tools_root.join(tool);
    bijux_dna_infra::ensure_dir(&out_dir)?;
    let tool_spec = build_tool_execution_spec(STAGE_ID, tool, &setup.registry, catalog, platform)?;
    let plan =
        bijux_dna_planner_fastq::tool_adapters::fastq::profile_read_lengths::plan_with_options(
            &tool_spec,
            &args.r1,
            args.r2.as_deref(),
            &out_dir,
            args.threads,
            args.histogram_bins,
        )?;
    let params_hash = stable_params_hash(&plan.params);
    let image_digest = benchmark_image_identity(&tool_spec);
    Ok(ReadLengthsToolPlan {
        tool: tool.to_string(),
        out_dir,
        tool_spec,
        plan,
        params_hash,
        image_digest,
    })
}

fn execute_read_lengths_tool(
    tool_plan: &ReadLengthsToolPlan,
    runner: RuntimeKind,
    jobs: usize,
) -> Result<ReadLengthsToolExecution> {
    let result = execute_plans_with_jobs(
        vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&tool_plan.plan)],
        runner,
        jobs,
    )?
    .into_iter()
    .next()
    .ok_or_else(|| anyhow!("missing execution result for {}", tool_plan.tool))?;
    Ok(ReadLengthsToolExecution { result })
}

fn read_lengths_tool_failure(
    tool_plan: &ReadLengthsToolPlan,
    execution: &ReadLengthsToolExecution,
) -> Option<RawFailure> {
    let exit_code = execution.result.exit_code;
    if exit_code == 0 {
        return None;
    }
    let stderr = execution.result.stderr.trim();
    let reason = if stderr.is_empty() {
        format!("tool {} failed with status {exit_code}", tool_plan.tool)
    } else {
        format!("tool {} failed with status {exit_code}: {stderr}", tool_plan.tool)
    };
    Some(RawFailure {
        stage: STAGE_ID.to_string(),
        tool: tool_plan.tool.clone(),
        reason,
        category: ErrorCategory::ToolError,
    })
}

fn read_lengths_execution_metrics(execution: &ReadLengthsToolExecution) -> ExecutionMetrics {
    ExecutionMetrics {
        runtime_s: execution.result.runtime_s,
        memory_mb: execution.result.memory_mb,
        exit_code: execution.result.exit_code,
    }
}

fn observe_read_lengths(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileReadLengthsArgs,
) -> Result<Vec<usize>> {
    let mut lengths = read_fastq_lengths(&args.r1)?;
    if let Some(r2) = args.r2.as_deref() {
        lengths.extend(read_fastq_lengths(r2)?);
    }
    Ok(lengths)
}

fn prepare_read_lengths_artifacts(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileReadLengthsArgs,
    plan: &StagePlanV1,
    lengths: &[usize],
) -> Result<ReadLengthsArtifacts> {
    let report_json = required_output_path(plan, "report_json")?.to_path_buf();
    let length_tsv = required_output_path(plan, "length_distribution_tsv")?.to_path_buf();
    let length_json = required_output_path(plan, "length_distribution_json")?.to_path_buf();
    let histogram_bins = args.histogram_bins.unwrap_or(100).max(1);
    if !length_tsv.exists() || !length_json.exists() {
        write_length_outputs(&length_tsv, &length_json, lengths, histogram_bins)?;
    }
    Ok(ReadLengthsArtifacts { report_json, length_tsv, length_json, histogram_bins })
}

fn observe_read_lengths_tool(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileReadLengthsArgs,
    plan: &StagePlanV1,
) -> Result<ReadLengthsObservation> {
    let lengths = observe_read_lengths(args)?;
    let artifacts = prepare_read_lengths_artifacts(args, plan, &lengths)?;
    Ok(ReadLengthsObservation { lengths, artifacts })
}

fn project_read_lengths_histogram(
    observation: &ReadLengthsObservation,
) -> Vec<ProfileReadLengthBinV1> {
    rebin_lengths(&observation.lengths, observation.artifacts.histogram_bins)
        .into_iter()
        .map(|(read_length, count)| ProfileReadLengthBinV1 {
            read_length: read_length as u64,
            count,
        })
        .collect()
}

fn build_read_lengths_metric_set(
    observation: &ReadLengthsObservation,
) -> Result<MetricSet<FastqReadLengthMetrics>> {
    let metrics = metrics_from_lengths(&observation.lengths)?;
    let metric_set = metric_set(metrics);
    bijux_dna_analyze::validate_metric_set(&metric_set)?;
    Ok(metric_set)
}

fn build_read_lengths_report(inputs: ReadLengthsReportInputs<'_>) -> ProfileReadLengthsReportV1 {
    ProfileReadLengthsReportV1 {
        schema_version: PROFILE_READ_LENGTHS_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_ID.to_string(),
        stage_id: STAGE_ID.to_string(),
        tool_id: inputs.tool.to_string(),
        paired_mode: if inputs.args.r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        threads: inputs.threads,
        histogram_bins: inputs.artifacts.histogram_bins,
        input_r1: inputs.args.r1.display().to_string(),
        input_r2: inputs.args.r2.as_ref().map(|path| path.display().to_string()),
        length_distribution_tsv: inputs.artifacts.length_tsv.display().to_string(),
        length_distribution_json: inputs.artifacts.length_json.display().to_string(),
        report_json: inputs.artifacts.report_json.display().to_string(),
        read_count: inputs.metrics.read_count,
        mean_read_length: inputs.metrics.mean_read_length,
        max_read_length: inputs.metrics.max_read_length,
        distinct_lengths: inputs.metrics.distinct_lengths,
        histogram: inputs.histogram,
        runtime_s: Some(inputs.execution_metrics.runtime_s),
        memory_mb: Some(inputs.execution_metrics.memory_mb),
        exit_code: Some(inputs.execution_metrics.exit_code),
        raw_backend_report: Some(inputs.artifacts.length_tsv.display().to_string()),
        raw_backend_report_format: Some("seqkit_fx2tab_tsv".to_string()),
    }
}

fn write_read_lengths_report(
    report_json: &Path,
    report: &ProfileReadLengthsReportV1,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(report_json, report).context("write read-lengths report")
}

fn write_read_lengths_metrics(
    out_dir: &Path,
    metric_set: &MetricSet<FastqReadLengthMetrics>,
) -> Result<()> {
    let metrics_json = serde_json::to_value(metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write read-lengths metrics")
}

fn write_read_lengths_artifacts(
    tool_plan: &ReadLengthsToolPlan,
    observation: &ReadLengthsObservation,
    report: &ProfileReadLengthsReportV1,
    metric_set: &MetricSet<FastqReadLengthMetrics>,
) -> Result<()> {
    write_read_lengths_report(&observation.artifacts.report_json, report)?;
    write_read_lengths_metrics(&tool_plan.out_dir, metric_set)
}

fn build_read_lengths_record(
    inputs: &ReadLengthsRecordInputs<'_>,
    metric_set: MetricSet<FastqReadLengthMetrics>,
) -> Result<BenchmarkRecord<FastqReadLengthMetrics>> {
    let record = BenchmarkRecord {
        context: build_benchmark_context(
            inputs.tool,
            inputs.tool_plan.tool_spec.tool_version.clone(),
            inputs.tool_plan.image_digest.clone(),
            inputs.setup.runner,
            inputs.platform,
            inputs.setup.input_hash.clone(),
            inputs.tool_plan.plan.params.clone(),
        ),
        execution: read_lengths_execution_metrics(inputs.execution),
        metrics: metric_set,
    };
    record.validate()?;
    Ok(record)
}

fn read_fastq_lengths(path: &Path) -> Result<Vec<usize>> {
    let raw = if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
    {
        let output = bijux_dna_runner::command_runner::run_command(
            "gzip",
            &["-cd".to_string(), path.to_string_lossy().into_owned()],
        )
        .with_context(|| format!("gzip -cd {}", path.display()))?;
        if output.exit_code != 0 {
            return Err(anyhow!("failed to decompress {}", path.display()));
        }
        output.stdout
    } else {
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?
    };

    let mut lengths = Vec::new();
    let mut lines = raw.lines();
    loop {
        let header = lines.next();
        let seq = lines.next();
        let plus = lines.next();
        let qual = lines.next();
        match (header, seq, plus, qual) {
            (None, None, None, None) => break,
            (Some(header), Some(seq), Some(plus), Some(qual)) => {
                if !header.starts_with('@') || !plus.starts_with('+') {
                    return Err(anyhow!("invalid FASTQ framing in {}", path.display()));
                }
                if seq.len() != qual.len() {
                    return Err(anyhow!(
                        "FASTQ sequence/quality length mismatch in {}",
                        path.display()
                    ));
                }
                lengths.push(seq.len());
            }
            _ => return Err(anyhow!("truncated FASTQ record in {}", path.display())),
        }
    }
    if lengths.is_empty() {
        return Err(anyhow!("no reads detected in {}", path.display()));
    }
    Ok(lengths)
}

fn write_length_outputs(
    tsv: &Path,
    json: &Path,
    lengths: &[usize],
    histogram_bins: u32,
) -> Result<()> {
    let hist = rebin_lengths(lengths, histogram_bins.max(1));
    let mut tsv_body = String::from("sample_id\tread_length\tcount\n");
    for (len, count) in &hist {
        tsv_body.push_str("sample\t");
        tsv_body.push_str(&len.to_string());
        tsv_body.push('\t');
        tsv_body.push_str(&count.to_string());
        tsv_body.push('\n');
    }
    bijux_dna_infra::atomic_write_bytes(tsv, tsv_body.as_bytes())?;
    let json_body = serde_json::json!({
        "schema_version": "bijux.fastq.profile_read_lengths.v1",
        "histogram": hist.iter().map(|(len, count)| serde_json::json!({"read_length": len, "count": count})).collect::<Vec<_>>(),
    });
    bijux_dna_infra::atomic_write_json(json, &json_body)?;
    Ok(())
}

fn required_output_path<'a>(
    plan: &'a bijux_dna_stage_contract::StagePlanV1,
    artifact_name: &str,
) -> Result<&'a Path> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_name)
        .map(|artifact| artifact.path.as_path())
        .ok_or_else(|| anyhow!("profile_read_lengths plan missing `{artifact_name}` output"))
}

fn rebin_lengths(lengths: &[usize], histogram_bins: u32) -> BTreeMap<usize, u64> {
    let mut exact = BTreeMap::<usize, u64>::new();
    for &len in lengths {
        *exact.entry(len).or_insert(0) += 1;
    }
    let target_bins = histogram_bins.max(1) as usize;
    if exact.len() <= target_bins {
        return exact;
    }

    let min_len = *exact.keys().next().unwrap_or(&0);
    let max_len = *exact.keys().last().unwrap_or(&min_len);
    if min_len == max_len {
        return exact;
    }
    let span = max_len - min_len + 1;
    let bin_width = span.div_ceil(target_bins).max(1);
    let mut rebinned = BTreeMap::<usize, u64>::new();
    for (len, count) in exact {
        let offset = len.saturating_sub(min_len);
        let bucket_start = min_len + ((offset / bin_width) * bin_width);
        *rebinned.entry(bucket_start).or_insert(0) += count;
    }
    rebinned
}

fn metrics_from_lengths(lengths: &[usize]) -> Result<FastqReadLengthMetrics> {
    let read_count = usize_to_u64(lengths.len());
    let total: usize = lengths.iter().sum();
    let max_read_length = usize_to_u64(lengths.iter().copied().max().unwrap_or(0));
    let distinct_lengths = usize_to_u64(lengths.iter().copied().collect::<BTreeSet<_>>().len());
    let metrics = FastqReadLengthMetrics {
        read_count,
        mean_read_length: usize_to_f64(total) / u64_to_f64(read_count),
        max_read_length,
        distinct_lengths,
    };
    metrics.validate()?;
    Ok(metrics)
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

fn usize_to_f64(value: usize) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

fn usize_to_u64(value: usize) -> u64 {
    value.try_into().unwrap_or(u64::MAX)
}
