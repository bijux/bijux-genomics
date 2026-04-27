use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::internal::fastq::stages::trim_bench_common::benchmark_image_identity;
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{fetch_fastq_chimeras_v1, insert_fastq_chimeras_v1};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqChimeraMetrics};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::{
    params::edna::ChimeraDetectionEffectiveParams, PairedMode, RemoveChimerasReportV1,
    REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION,
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
    build_benchmark_context, observe_fastq_stats,
};
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};

const STAGE_ID: &str = "fastq.remove_chimeras";

/// Benchmark FASTQ chimera-removal tools under governed contracts.
///
/// # Errors
/// Returns an error if planning, execution, report parsing, or persistence fails.
pub fn bench_fastq_remove_chimeras<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqRemoveChimerasArgs,
) -> Result<BenchOutcome<FastqChimeraMetrics>> {
    let selected_tools = select_remove_chimeras_benchmark_tools(args)?;
    let setup =
        prepare_remove_chimeras_setup(catalog, platform, runner_override, args, &selected_tools)?;

    if args.explain {
        write_remove_chimeras_explain(&setup)?;
    }

    ensure_remove_chimeras_qa(catalog, platform, &setup.tools)?;

    let store = RemoveChimerasBenchmarkStore::from_setup(&setup);
    let conn = bijux_dna_analyze::open_sqlite(&store.sqlite_path)?;
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::new();

    for tool in &setup.tools {
        let tool_plan = prepare_remove_chimeras_tool_plan(catalog, platform, args, &setup, tool)?;
        let cache_identity =
            RemoveChimerasCacheIdentity::from_plan(platform, &setup, tool, &tool_plan);
        if let Ok(Some(record)) = fetch_fastq_chimeras_v1(
            &conn,
            cache_identity.tool,
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
        let execution = execute_remove_chimeras_tool(&tool_plan, setup.runner, jobs, tool)?;
        if let Some(failure) = remove_chimeras_tool_failure(tool, &execution) {
            failures.push(failure);
            continue;
        }
        let outputs = resolve_remove_chimeras_outputs(&tool_plan.plan)?;
        let observation =
            observe_remove_chimeras_outputs(catalog, platform, args, &setup, &outputs)?;
        let measurements = remove_chimeras_measurements(&setup, &observation);
        let effective_params: ChimeraDetectionEffectiveParams =
            serde_json::from_value(tool_plan.plan.effective_params.clone())
                .map_err(|error| anyhow!("parse remove_chimeras effective params: {error}"))?;
        let report_inputs = RemoveChimerasReportInputs {
            tool_id: tool,
            paired_mode: if args.r2.is_some() {
                PairedMode::PairedEnd
            } else {
                PairedMode::SingleEnd
            },
            effective_params: &effective_params,
            input_reads: &args.r1,
            output_reads: &outputs.filtered_reads,
            chimera_metrics_json: &outputs.metrics_json,
            chimeras_fasta: outputs.chimeras_fasta.as_deref(),
            uchime_report_tsv: outputs.uchime_report_tsv.as_deref(),
            reads_in: measurements.reads_in,
            reads_out: measurements.reads_out,
            chimeras_removed: measurements.chimeras_removed,
            chimera_fraction: measurements.chimera_fraction,
            used_fallback: observation.used_fallback,
            runtime_s: execution.result.runtime_s,
            memory_mb: execution.result.memory_mb,
            exit_code: execution.result.exit_code,
        };
        let report = build_remove_chimeras_report(&report_inputs);
        let metrics = measurements.metrics();
        let metric_set = metric_set(metrics);
        validate_remove_chimeras_report_identity(tool, &report)?;
        write_remove_chimeras_artifacts(&tool_plan.out_dir, &outputs, &report, &metric_set)?;
        let record = build_remove_chimeras_record(
            &RemoveChimerasRecordInputs {
                platform,
                setup: &setup,
                tool,
                tool_plan: &tool_plan,
                execution: &execution,
            },
            metric_set,
        )?;
        persist_remove_chimeras_record(&store, &record, |record| {
            insert_fastq_chimeras_v1(&conn, record).context("insert bench sqlite")
        })?;
        records.push(record);
    }

    Ok(BenchOutcome { records, failures, bench_dir: setup.bench_dir, explain: args.explain })
}

struct RemoveChimerasBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    runner: RuntimeKind,
    input_stats_r1: SeqkitMetrics,
    input_stats_r2: Option<SeqkitMetrics>,
    input_hash: String,
    bench_dir: PathBuf,
    tools_root: PathBuf,
}

struct RemoveChimerasBenchmarkStore {
    sqlite_path: PathBuf,
    jsonl_path: PathBuf,
}

struct RemoveChimerasToolPlan {
    out_dir: PathBuf,
    tool_spec: ToolExecutionSpecV1,
    plan: StagePlanV1,
    params_hash: String,
    image_digest: String,
}

struct RemoveChimerasToolExecution {
    result: StageResultV1,
}

struct RemoveChimerasOutputs {
    filtered_reads: PathBuf,
    metrics_json: PathBuf,
    report_json: PathBuf,
    chimeras_fasta: Option<PathBuf>,
    uchime_report_tsv: Option<PathBuf>,
}

struct RemoveChimerasObservation {
    output_stats_r1: SeqkitMetrics,
    used_fallback: bool,
}

struct RemoveChimerasMeasurements {
    reads_in: u64,
    reads_out: u64,
    chimeras_removed: u64,
    chimera_fraction: f64,
}

struct RemoveChimerasRecordInputs<'a> {
    platform: &'a PlatformSpec,
    setup: &'a RemoveChimerasBenchmarkSetup,
    tool: &'a str,
    tool_plan: &'a RemoveChimerasToolPlan,
    execution: &'a RemoveChimerasToolExecution,
}

impl RemoveChimerasMeasurements {
    fn metrics(&self) -> FastqChimeraMetrics {
        FastqChimeraMetrics {
            reads_in: self.reads_in,
            reads_out: self.reads_out,
            chimeras_removed: self.chimeras_removed,
            chimera_fraction: self.chimera_fraction,
        }
    }
}

fn build_remove_chimeras_record(
    inputs: &RemoveChimerasRecordInputs<'_>,
    metric_set: bijux_dna_analyze::MetricSet<FastqChimeraMetrics>,
) -> Result<BenchmarkRecord<FastqChimeraMetrics>> {
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
        execution: remove_chimeras_execution_metrics(inputs.execution),
        metrics: metric_set,
    };
    record.validate()?;
    Ok(record)
}

struct RemoveChimerasCacheIdentity<'a> {
    tool: &'a str,
    tool_version: String,
    image_digest: String,
    runner: String,
    platform: String,
    input_hash: String,
    params_hash: String,
}

impl<'a> RemoveChimerasCacheIdentity<'a> {
    fn from_plan(
        platform: &PlatformSpec,
        setup: &RemoveChimerasBenchmarkSetup,
        tool: &'a str,
        tool_plan: &RemoveChimerasToolPlan,
    ) -> Self {
        Self {
            tool,
            tool_version: tool_plan.tool_spec.tool_version.clone(),
            image_digest: tool_plan.image_digest.clone(),
            runner: setup.runner.to_string(),
            platform: platform.name.clone(),
            input_hash: setup.input_hash.clone(),
            params_hash: tool_plan.params_hash.clone(),
        }
    }
}

impl RemoveChimerasBenchmarkStore {
    fn from_setup(setup: &RemoveChimerasBenchmarkSetup) -> Self {
        Self {
            sqlite_path: setup.bench_dir.join("bench.sqlite"),
            jsonl_path: setup.bench_dir.join("bench.jsonl"),
        }
    }
}

fn prepare_remove_chimeras_setup<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqRemoveChimerasArgs,
    selected_tools: &[String],
) -> Result<RemoveChimerasBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_ID, selected_tools, &registry, false)?;
    let runner = ensure_bench_runner(platform, runner_override)?;
    let input_stats_r1 = observe_fastq_stats(catalog, platform, runner, &args.r1)?;
    let input_stats_r2 = if let Some(r2) = args.r2.as_deref() {
        Some(observe_fastq_stats(catalog, platform, runner, r2)?)
    } else {
        None
    };
    let input_hash = remove_chimeras_input_hash(args)?;
    let bench_dir_name =
        bench_dir_name(&bijux_dna_domain_fastq::stages::ids::STAGE_REMOVE_CHIMERAS)
            .ok_or_else(|| anyhow!("bench dir missing for {STAGE_ID}"))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir)?;
    bijux_dna_infra::ensure_dir(&tools_root)?;

    Ok(RemoveChimerasBenchmarkSetup {
        registry,
        tools,
        runner,
        input_stats_r1,
        input_stats_r2,
        input_hash,
        bench_dir,
        tools_root,
    })
}

fn remove_chimeras_input_hash(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqRemoveChimerasArgs,
) -> Result<String> {
    if let Some(r2) = args.r2.as_deref() {
        return Ok(format!("{}+{}", hash_file_sha256(&args.r1)?, hash_file_sha256(r2)?));
    }
    Ok(hash_file_sha256(&args.r1)?)
}

fn write_remove_chimeras_explain(setup: &RemoveChimerasBenchmarkSetup) -> Result<()> {
    write_explain_md(&setup.bench_dir, STAGE_ID, &setup.tools, &[], None)?;
    write_explain_plan_json(&setup.bench_dir, STAGE_ID, &setup.tools, &setup.registry, None)
}

fn ensure_remove_chimeras_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(STAGE_ID, tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_ID, tools, platform, catalog)
}

fn prepare_remove_chimeras_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqRemoveChimerasArgs,
    setup: &RemoveChimerasBenchmarkSetup,
    tool: &str,
) -> Result<RemoveChimerasToolPlan> {
    let out_dir = setup.tools_root.join(tool);
    bijux_dna_infra::ensure_dir(&out_dir)?;
    let tool_spec = build_tool_execution_spec(STAGE_ID, tool, &setup.registry, catalog, platform)?;
    let effective_params =
        governed_chimera_params(args.threads.unwrap_or(tool_spec.resources.threads).max(1));
    let plan =
        bijux_dna_planner_fastq::tool_adapters::fastq::remove_chimeras::plan_with_effective_params(
            &tool_spec,
            &args.r1,
            args.r2.as_deref(),
            &out_dir,
            &effective_params,
        )?;
    let params_hash = stable_params_hash(&plan.params);
    let image_digest = benchmark_image_identity(&tool_spec);
    Ok(RemoveChimerasToolPlan { out_dir, tool_spec, plan, params_hash, image_digest })
}

fn execute_remove_chimeras_tool(
    tool_plan: &RemoveChimerasToolPlan,
    runner: RuntimeKind,
    jobs: usize,
    tool: &str,
) -> Result<RemoveChimerasToolExecution> {
    let result = execute_plans_with_jobs(
        vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&tool_plan.plan)],
        runner,
        jobs,
    )?
    .into_iter()
    .next()
    .ok_or_else(|| anyhow!("missing execution result for {tool}"))?;
    Ok(RemoveChimerasToolExecution { result })
}

fn remove_chimeras_execution_metrics(execution: &RemoveChimerasToolExecution) -> ExecutionMetrics {
    ExecutionMetrics {
        runtime_s: execution.result.runtime_s,
        memory_mb: execution.result.memory_mb,
        exit_code: execution.result.exit_code,
    }
}

fn persist_remove_chimeras_record(
    store: &RemoveChimerasBenchmarkStore,
    record: &BenchmarkRecord<FastqChimeraMetrics>,
    insert_record: impl FnOnce(&BenchmarkRecord<FastqChimeraMetrics>) -> Result<()>,
) -> Result<()> {
    append_jsonl(&store.jsonl_path, record).context("write bench.jsonl")?;
    insert_record(record)
}

fn remove_chimeras_tool_failure(
    tool: &str,
    execution: &RemoveChimerasToolExecution,
) -> Option<RawFailure> {
    let exit_code = execution.result.exit_code;
    if exit_code == 0 {
        return None;
    }
    let stderr = execution.result.stderr.trim();
    let reason = if stderr.is_empty() {
        format!("tool {tool} failed with status {exit_code}")
    } else {
        format!("tool {tool} failed with status {exit_code}: {stderr}")
    };
    Some(RawFailure {
        stage: STAGE_ID.to_string(),
        tool: tool.to_string(),
        reason,
        category: ErrorCategory::ToolError,
    })
}

fn resolve_remove_chimeras_outputs(plan: &StagePlanV1) -> Result<RemoveChimerasOutputs> {
    let outputs = RemoveChimerasOutputs {
        filtered_reads: required_remove_chimeras_output(plan, "chimera_filtered_reads")?,
        metrics_json: required_remove_chimeras_output(plan, "chimera_metrics_json")?,
        report_json: required_remove_chimeras_output(plan, "report_json")?,
        chimeras_fasta: optional_remove_chimeras_output(plan, "chimeras_fasta"),
        uchime_report_tsv: optional_remove_chimeras_output(plan, "uchime_report_tsv"),
    };
    validate_remove_chimeras_output_paths(&outputs)?;
    Ok(outputs)
}

fn validate_remove_chimeras_output_paths(outputs: &RemoveChimerasOutputs) -> Result<()> {
    let mut paths = BTreeSet::new();
    for path in [
        Some(outputs.filtered_reads.as_path()),
        Some(outputs.metrics_json.as_path()),
        Some(outputs.report_json.as_path()),
        outputs.chimeras_fasta.as_deref(),
        outputs.uchime_report_tsv.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        if !paths.insert(path) {
            return Err(anyhow!(
                "remove_chimeras output path reused by multiple artifacts: {}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn observe_remove_chimeras_outputs<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqRemoveChimerasArgs,
    setup: &RemoveChimerasBenchmarkSetup,
    outputs: &RemoveChimerasOutputs,
) -> Result<RemoveChimerasObservation> {
    let used_fallback = !outputs.filtered_reads.exists();
    if used_fallback {
        std::fs::copy(&args.r1, &outputs.filtered_reads)?;
    }
    let output_stats_r1 =
        observe_fastq_stats(catalog, platform, setup.runner, &outputs.filtered_reads)?;
    Ok(RemoveChimerasObservation { output_stats_r1, used_fallback })
}

fn remove_chimeras_measurements(
    setup: &RemoveChimerasBenchmarkSetup,
    observation: &RemoveChimerasObservation,
) -> RemoveChimerasMeasurements {
    let reads_in =
        setup.input_stats_r1.reads + setup.input_stats_r2.as_ref().map_or(0, |stats| stats.reads);
    let reads_out = observation.output_stats_r1.reads;
    let chimeras_removed = reads_in.saturating_sub(reads_out);
    let chimera_fraction =
        if reads_in == 0 { 0.0 } else { u64_to_f64(chimeras_removed) / u64_to_f64(reads_in) };
    RemoveChimerasMeasurements { reads_in, reads_out, chimeras_removed, chimera_fraction }
}

fn required_remove_chimeras_output(plan: &StagePlanV1, name: &str) -> Result<PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == name)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| anyhow!("remove_chimeras plan missing {name}"))
}

fn optional_remove_chimeras_output(plan: &StagePlanV1, name: &str) -> Option<PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == name)
        .map(|artifact| artifact.path.clone())
}

fn select_remove_chimeras_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqRemoveChimerasArgs,
) -> Result<Vec<String>> {
    let tools = bijux_dna_planner_fastq::select_remove_chimeras_tools(&args.tools)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_ID, artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_ID, &header);
    Ok(tools)
}

fn parse_uchime_summary(path: Option<&std::path::Path>) -> Option<serde_json::Value> {
    let path = path?;
    let raw = std::fs::read_to_string(path).ok()?;
    let parsed_records = raw.lines().filter(|line| !line.trim().is_empty()).count() as u64;
    let flagged_records = raw
        .lines()
        .filter(|line| line.split('\t').next_back().is_some_and(|flag| flag == "Y"))
        .count() as u64;
    Some(serde_json::json!({
        "parsed_records": parsed_records,
        "flagged_records": flagged_records,
    }))
}

struct RemoveChimerasReportInputs<'a> {
    tool_id: &'a str,
    paired_mode: PairedMode,
    effective_params: &'a ChimeraDetectionEffectiveParams,
    input_reads: &'a std::path::Path,
    output_reads: &'a std::path::Path,
    chimera_metrics_json: &'a std::path::Path,
    chimeras_fasta: Option<&'a std::path::Path>,
    uchime_report_tsv: Option<&'a std::path::Path>,
    reads_in: u64,
    reads_out: u64,
    chimeras_removed: u64,
    chimera_fraction: f64,
    used_fallback: bool,
    runtime_s: f64,
    memory_mb: f64,
    exit_code: i32,
}

fn build_remove_chimeras_report(inputs: &RemoveChimerasReportInputs<'_>) -> RemoveChimerasReportV1 {
    RemoveChimerasReportV1 {
        schema_version: REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_ID.to_string(),
        stage_id: STAGE_ID.to_string(),
        tool_id: inputs.tool_id.to_string(),
        paired_mode: inputs.paired_mode,
        threads: inputs.effective_params.threads,
        method: inputs.effective_params.method.clone(),
        detection_scope: inputs.effective_params.detection_scope.clone(),
        chimera_removed_definition: inputs.effective_params.chimera_removed_definition.clone(),
        input_reads: inputs.input_reads.display().to_string(),
        output_reads: inputs.output_reads.display().to_string(),
        chimera_metrics_json: inputs.chimera_metrics_json.display().to_string(),
        chimeras_fasta: inputs.chimeras_fasta.map(|path| path.display().to_string()),
        uchime_report_tsv: inputs.uchime_report_tsv.map(|path| path.display().to_string()),
        reads_in: Some(inputs.reads_in),
        reads_out: Some(inputs.reads_out),
        chimeras_removed: Some(inputs.chimeras_removed),
        chimera_fraction: Some(inputs.chimera_fraction),
        used_fallback: inputs.used_fallback,
        raw_backend_report: inputs.uchime_report_tsv.map(|path| path.display().to_string()),
        raw_backend_report_format: inputs
            .uchime_report_tsv
            .map(|_| inputs.effective_params.raw_backend_report_format.clone()),
        runtime_s: Some(inputs.runtime_s),
        memory_mb: Some(inputs.memory_mb),
        exit_code: Some(inputs.exit_code),
        backend_metrics: parse_uchime_summary(inputs.uchime_report_tsv),
    }
}

fn write_remove_chimeras_artifacts(
    out_dir: &std::path::Path,
    outputs: &RemoveChimerasOutputs,
    report: &RemoveChimerasReportV1,
    metric_set: &bijux_dna_analyze::MetricSet<FastqChimeraMetrics>,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(&outputs.report_json, report)?;
    bijux_dna_infra::atomic_write_json(
        &outputs.metrics_json,
        &compatibility_metrics_from_report(report),
    )?;
    bijux_dna_infra::atomic_write_json(
        &out_dir.join("metrics.json"),
        &serde_json::to_value(metric_set)?,
    )?;
    Ok(())
}

fn validate_remove_chimeras_report_identity(
    tool: &str,
    report: &RemoveChimerasReportV1,
) -> Result<()> {
    if report.schema_version != REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION {
        return Err(anyhow!(
            "remove_chimeras report schema mismatch: expected {}, observed {}",
            REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION,
            report.schema_version
        ));
    }
    if report.stage != STAGE_ID || report.stage_id != STAGE_ID {
        return Err(anyhow!(
            "remove_chimeras report stage mismatch: observed stage={} stage_id={}",
            report.stage,
            report.stage_id
        ));
    }
    if report.tool_id != tool {
        return Err(anyhow!(
            "remove_chimeras report tool mismatch: expected {}, observed {}",
            tool,
            report.tool_id
        ));
    }
    Ok(())
}

fn compatibility_metrics_from_report(report: &RemoveChimerasReportV1) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "bijux.fastq.remove_chimeras.v2",
        "chimera_fraction": report.chimera_fraction.unwrap_or(0.0),
        "chimeras_removed": report.chimeras_removed.unwrap_or(0),
        "non_chimera_reads": report.reads_out.unwrap_or(0),
        "tool": report.tool_id,
        "used_fallback": report.used_fallback,
    })
}

fn governed_chimera_params(threads: u32) -> ChimeraDetectionEffectiveParams {
    ChimeraDetectionEffectiveParams {
        method: "vsearch_uchime_denovo".to_string(),
        detection_scope: "denovo".to_string(),
        input_layout: "single_stream".to_string(),
        threads,
        report_artifact: "report_json".to_string(),
        metrics_artifact: "chimera_metrics_json".to_string(),
        chimera_sequence_artifact: "chimeras_fasta".to_string(),
        raw_backend_report_artifact: "uchime_report_tsv".to_string(),
        raw_backend_report_format: "vsearch_uchime_tsv".to_string(),
        chimera_removed_definition:
            "reads flagged as de_novo chimeras are excluded from downstream abundance tables"
                .to_string(),
        fallback_behavior: "copy_input_reads_and_mark_report".to_string(),
    }
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}
