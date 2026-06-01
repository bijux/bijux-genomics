use std::collections::{BTreeSet, HashMap};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{fetch_fastq_umi_v1, insert_fastq_umi_v1};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqUmiMetrics, MetricSet};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::measure::SeqkitMetrics;
use bijux_dna_core::prelude::params_hash;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::params::umi::{
    UmiDedupPolicy, UmiDownstreamPropagation, UmiExtractionLocation, UmiFailedExtractionPolicy,
    UmiGroupingPolicy, UmiReadNameTransform,
};
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
use serde::Serialize;

use crate::internal::fastq::stages::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context, observe_fastq_stats,
};
use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_EXTRACT_UMIS,
};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;

const LOCAL_EXTRACT_UMIS_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.extract_umis.local_smoke.report.v1";

#[derive(Debug, Clone, Serialize)]
struct LocalExtractUmisSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    planned_tool_id: String,
    report_tool_id: String,
    umi_pattern: String,
    extracted_umi_count: u64,
    invalid_umi_count: u64,
    tag_header_format: String,
    umi_extracted_fastq_gz: String,
    umi_extracted_r2_fastq_gz: String,
    case_report_json: String,
    raw_backend_report: String,
}

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

    let store = UmiBenchmarkStore::from_setup(&setup);
    let conn = bijux_dna_analyze::open_sqlite(&store.sqlite_path).context("open bench sqlite")?;
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::<BenchmarkRecord<FastqUmiMetrics>>::new();
    for tool in &setup.tools {
        let tool_plan = prepare_umi_tool_plan(catalog, platform, args, &setup, jobs, tool)?;
        let cache_identity = UmiCacheIdentity::from_plan(platform, &setup, &tool_plan);
        if let Ok(Some(record)) = fetch_fastq_umi_v1(
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
        let execution = execute_umi_tool(&tool_plan, setup.runner, jobs)?;
        if let Some(failure) = umi_tool_failure(&tool_plan, &execution) {
            failures.push(failure);
            continue;
        }
        let record = build_umi_record(&UmiRecordInputs {
            catalog,
            platform,
            runner: setup.runner,
            input_hash: &setup.input_hash,
            r1: &args.r1,
            r2,
            input_stats_r1: &setup.input_stats_r1,
            input_stats_r2: &setup.input_stats_r2,
            tool: &tool_plan.tool,
            tool_spec: &tool_plan.tool_spec,
            image_digest: &tool_plan.image_digest,
            params: &tool_plan.plan.params,
            plan: &tool_plan.plan,
            out_dir: &tool_plan.out_dir,
            execution: &execution,
        })?;
        persist_umi_record(&store, &record, |record| {
            insert_fastq_umi_v1(&conn, record).context("insert bench sqlite")
        })?;
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

struct UmiBenchmarkStore {
    sqlite_path: PathBuf,
    jsonl_path: PathBuf,
}

impl UmiBenchmarkStore {
    fn from_setup(setup: &UmiBenchmarkSetup) -> Self {
        Self {
            sqlite_path: setup.bench_dir.join("bench.sqlite"),
            jsonl_path: setup.bench_dir.join("bench.jsonl"),
        }
    }
}

struct UmiToolPlan {
    tool: String,
    out_dir: PathBuf,
    tool_spec: ToolExecutionSpecV1,
    plan: StagePlanV1,
    params_hash: String,
    image_digest: String,
}

struct UmiToolExecution {
    result: StageResultV1,
}

struct UmiCacheIdentity {
    tool: String,
    tool_version: String,
    image_digest: String,
    runner: String,
    platform: String,
    input_hash: String,
    params_hash: String,
}

impl UmiCacheIdentity {
    fn from_plan(
        platform: &PlatformSpec,
        setup: &UmiBenchmarkSetup,
        tool_plan: &UmiToolPlan,
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

struct UmiArtifacts {
    output_r1: PathBuf,
    output_r2: PathBuf,
    report_json: PathBuf,
}

struct UmiRecordInputs<'a, S: ::std::hash::BuildHasher> {
    catalog: &'a HashMap<String, ToolImageSpec, S>,
    platform: &'a PlatformSpec,
    runner: RuntimeKind,
    input_hash: &'a str,
    r1: &'a std::path::Path,
    r2: &'a std::path::Path,
    input_stats_r1: &'a SeqkitMetrics,
    input_stats_r2: &'a SeqkitMetrics,
    tool: &'a str,
    tool_spec: &'a ToolExecutionSpecV1,
    image_digest: &'a str,
    params: &'a serde_json::Value,
    plan: &'a StagePlanV1,
    out_dir: &'a std::path::Path,
    execution: &'a UmiToolExecution,
}

struct UmiReportInputs<'a> {
    tool: &'a str,
    threads: u32,
    params: &'a serde_json::Value,
    r1: &'a std::path::Path,
    r2: &'a std::path::Path,
    output_r1: &'a std::path::Path,
    output_r2: &'a std::path::Path,
    report_json: &'a std::path::Path,
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
            extraction_location: None,
            read_name_transform: None,
            failed_extraction_policy: None,
            downstream_propagation: None,
            grouping_policy: None,
            downstream_dedup_policy: None,
        },
    )?;
    let params_hash = stable_params_hash(&plan.params);
    let image_digest = benchmark_image_identity(&tool_spec);
    Ok(UmiToolPlan { tool: tool.to_string(), out_dir, tool_spec, plan, params_hash, image_digest })
}

fn execute_umi_tool(
    tool_plan: &UmiToolPlan,
    runner: RuntimeKind,
    jobs: usize,
) -> Result<UmiToolExecution> {
    let result = execute_plans_with_jobs(
        vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&tool_plan.plan)],
        runner,
        jobs,
    )?
    .into_iter()
    .next()
    .ok_or_else(|| anyhow!("missing execution result for {}", tool_plan.tool))?;
    Ok(UmiToolExecution { result })
}

fn umi_tool_failure(tool_plan: &UmiToolPlan, execution: &UmiToolExecution) -> Option<RawFailure> {
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
        stage: STAGE_EXTRACT_UMIS.as_str().to_string(),
        tool: tool_plan.tool.clone(),
        reason,
        category: ErrorCategory::ToolError,
    })
}

fn umi_execution_metrics(execution: &UmiToolExecution) -> ExecutionMetrics {
    ExecutionMetrics {
        runtime_s: execution.result.runtime_s,
        memory_mb: execution.result.memory_mb,
        exit_code: execution.result.exit_code,
    }
}

fn persist_umi_record(
    store: &UmiBenchmarkStore,
    record: &BenchmarkRecord<FastqUmiMetrics>,
    insert_record: impl FnOnce(&BenchmarkRecord<FastqUmiMetrics>) -> Result<()>,
) -> Result<()> {
    append_jsonl(&store.jsonl_path, record).context("write bench.jsonl")?;
    insert_record(record)
}

fn prepare_umi_artifacts(plan: &StagePlanV1) -> Result<UmiArtifacts> {
    let output_r1 = required_output_path(plan, "umi_reads_r1")?.to_path_buf();
    let output_r2 = required_output_path(plan, "umi_reads_r2")?.to_path_buf();
    let report_json = required_output_path(plan, "report_json")?.to_path_buf();
    validate_umi_artifact_paths(&output_r1, &output_r2, &report_json)?;
    Ok(UmiArtifacts { output_r1, output_r2, report_json })
}

fn validate_umi_artifact_paths(
    output_r1: &Path,
    output_r2: &Path,
    report_json: &Path,
) -> Result<()> {
    let mut paths = BTreeSet::new();
    for path in [output_r1, output_r2, report_json] {
        if !paths.insert(path) {
            return Err(anyhow!(
                "extract_umis output path reused by multiple artifacts: {}",
                path.display()
            ));
        }
    }
    Ok(())
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
    let input_hash = umi_input_hash(&args.r1, r2)?;
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
    let artifacts = prepare_umi_artifacts(inputs.plan)?;
    let output_stats_r1 =
        observe_fastq_stats(inputs.catalog, inputs.platform, inputs.runner, &artifacts.output_r1)
            .with_context(|| format!("observe umi output r1 {}", artifacts.output_r1.display()))?;
    let output_stats_r2 =
        observe_fastq_stats(inputs.catalog, inputs.platform, inputs.runner, &artifacts.output_r2)
            .with_context(|| format!("observe umi output r2 {}", artifacts.output_r2.display()))?;
    let report = build_umi_report(&UmiReportInputs {
        tool: inputs.tool,
        threads: inputs.tool_spec.resources.threads,
        params: inputs.params,
        r1: inputs.r1,
        r2: inputs.r2,
        output_r1: &artifacts.output_r1,
        output_r2: &artifacts.output_r2,
        report_json: &artifacts.report_json,
        input_stats_r1: inputs.input_stats_r1,
        input_stats_r2: inputs.input_stats_r2,
        output_stats_r1: &output_stats_r1,
        output_stats_r2: &output_stats_r2,
        execution: &inputs.execution.result,
    });
    let metrics = umi_metrics_from_report(&report);
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    validate_umi_report_identity(inputs.tool, &report)?;
    validate_umi_report_paired_contract(&report)?;
    validate_umi_report_execution(&report, &inputs.execution.result)?;
    validate_umi_report_paths(&report, inputs.r1, inputs.r2, &artifacts)?;
    validate_umi_report_observed_counts(
        &report,
        inputs.input_stats_r1,
        inputs.input_stats_r2,
        &output_stats_r1,
        &output_stats_r2,
    )?;
    validate_umi_report_read_semantics(&report)?;
    validate_umi_report_metrics(&report, &metric_set.metrics)?;
    write_umi_report(&artifacts.report_json, &report)?;
    write_umi_metrics(inputs.out_dir, &metric_set)?;
    validate_umi_written_artifacts(&artifacts, inputs.out_dir, &report)?;

    let context = build_benchmark_context(
        inputs.tool,
        inputs.tool_spec.tool_version.clone(),
        inputs.image_digest.to_string(),
        inputs.runner,
        inputs.platform,
        inputs.input_hash.to_string(),
        inputs.params.clone(),
    );
    let record = BenchmarkRecord {
        context,
        execution: umi_execution_metrics(inputs.execution),
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
        extraction_location: parse_umi_extraction_location(
            inputs.params.get("extraction_location").and_then(serde_json::Value::as_str),
        ),
        read_name_transform: parse_umi_read_name_transform(
            inputs.params.get("read_name_transform").and_then(serde_json::Value::as_str),
        ),
        failed_extraction_policy: parse_umi_failed_extraction_policy(
            inputs.params.get("failed_extraction_policy").and_then(serde_json::Value::as_str),
        ),
        downstream_propagation: parse_umi_downstream_propagation(
            inputs.params.get("downstream_propagation").and_then(serde_json::Value::as_str),
        ),
        grouping_policy: parse_umi_grouping_policy(
            inputs.params.get("grouping_policy").and_then(serde_json::Value::as_str),
        ),
        downstream_dedup_policy: parse_umi_downstream_dedup_policy(
            inputs.params.get("downstream_dedup_policy").and_then(serde_json::Value::as_str),
        ),
        input_r1: inputs.r1.display().to_string(),
        input_r2: Some(inputs.r2.display().to_string()),
        output_r1: inputs.output_r1.display().to_string(),
        output_r2: Some(inputs.output_r2.display().to_string()),
        report_json: inputs.report_json.display().to_string(),
        reads_in,
        reads_out,
        bases_in,
        bases_out,
        pairs_in,
        pairs_out,
        reads_with_umi,
        failed_extractions: Some(reads_in.saturating_sub(reads_with_umi)),
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

fn parse_umi_extraction_location(value: Option<&str>) -> UmiExtractionLocation {
    match value.unwrap_or("read1_prefix") {
        "read2_prefix" => UmiExtractionLocation::Read2Prefix,
        "index_read" => UmiExtractionLocation::IndexRead,
        "header_tag" => UmiExtractionLocation::HeaderTag,
        _ => UmiExtractionLocation::Read1Prefix,
    }
}

fn parse_umi_read_name_transform(value: Option<&str>) -> UmiReadNameTransform {
    match value.unwrap_or("append_to_header") {
        "replace_header" => UmiReadNameTransform::ReplaceHeader,
        "none" => UmiReadNameTransform::None,
        _ => UmiReadNameTransform::AppendToHeader,
    }
}

fn parse_umi_failed_extraction_policy(value: Option<&str>) -> UmiFailedExtractionPolicy {
    match value.unwrap_or("refuse_stage") {
        "retain_unmodified" => UmiFailedExtractionPolicy::RetainUnmodified,
        "route_to_rejected" => UmiFailedExtractionPolicy::RouteToRejected,
        _ => UmiFailedExtractionPolicy::RefuseStage,
    }
}

fn parse_umi_downstream_propagation(value: Option<&str>) -> UmiDownstreamPropagation {
    match value.unwrap_or("header_and_report") {
        "header_only" => UmiDownstreamPropagation::HeaderOnly,
        _ => UmiDownstreamPropagation::HeaderAndReport,
    }
}

fn parse_umi_grouping_policy(value: Option<&str>) -> UmiGroupingPolicy {
    match value.unwrap_or("pair_aware") {
        "exact_header_tag" => UmiGroupingPolicy::ExactHeaderTag,
        _ => UmiGroupingPolicy::PairAware,
    }
}

fn parse_umi_downstream_dedup_policy(value: Option<&str>) -> UmiDedupPolicy {
    match value.unwrap_or("sequence_identity_recommended") {
        "coordinate_aware_recommended" => UmiDedupPolicy::CoordinateAwareRecommended,
        _ => UmiDedupPolicy::SequenceIdentityRecommended,
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

fn validate_umi_report_identity(tool: &str, report: &ExtractUmisReportV1) -> Result<()> {
    if report.schema_version != EXTRACT_UMIS_REPORT_SCHEMA_VERSION {
        return Err(anyhow!(
            "extract_umis report schema mismatch: expected {}, observed {}",
            EXTRACT_UMIS_REPORT_SCHEMA_VERSION,
            report.schema_version
        ));
    }
    if report.stage != STAGE_EXTRACT_UMIS.as_str() || report.stage_id != STAGE_EXTRACT_UMIS.as_str()
    {
        return Err(anyhow!(
            "extract_umis report stage mismatch: observed stage={} stage_id={}",
            report.stage,
            report.stage_id
        ));
    }
    if report.tool_id != tool {
        return Err(anyhow!(
            "extract_umis report tool mismatch: expected {}, observed {}",
            tool,
            report.tool_id
        ));
    }
    Ok(())
}

fn validate_umi_report_paired_contract(report: &ExtractUmisReportV1) -> Result<()> {
    if report.paired_mode != PairedMode::PairedEnd {
        return Err(anyhow!("extract_umis report must be paired-end"));
    }
    if report.input_r2.is_none() {
        return Err(anyhow!("extract_umis paired report missing input_r2"));
    }
    if report.output_r2.is_none() {
        return Err(anyhow!("extract_umis paired report missing output_r2"));
    }
    if report.pairs_in.is_none() {
        return Err(anyhow!("extract_umis paired report missing pairs_in"));
    }
    if report.pairs_out.is_none() {
        return Err(anyhow!("extract_umis paired report missing pairs_out"));
    }
    Ok(())
}

fn validate_umi_report_execution(
    report: &ExtractUmisReportV1,
    execution: &StageResultV1,
) -> Result<()> {
    if report.runtime_s.is_none_or(|observed| (observed - execution.runtime_s).abs() > f64::EPSILON)
    {
        return Err(anyhow!(
            "extract_umis report runtime mismatch: expected {}, observed {:?}",
            execution.runtime_s,
            report.runtime_s
        ));
    }
    if report.memory_mb.is_none_or(|observed| (observed - execution.memory_mb).abs() > f64::EPSILON)
    {
        return Err(anyhow!(
            "extract_umis report memory mismatch: expected {}, observed {:?}",
            execution.memory_mb,
            report.memory_mb
        ));
    }
    if report.exit_code != Some(execution.exit_code) {
        return Err(anyhow!(
            "extract_umis report exit code mismatch: expected {}, observed {:?}",
            execution.exit_code,
            report.exit_code
        ));
    }
    Ok(())
}

fn validate_umi_report_observed_counts(
    report: &ExtractUmisReportV1,
    input_stats_r1: &SeqkitMetrics,
    input_stats_r2: &SeqkitMetrics,
    output_stats_r1: &SeqkitMetrics,
    output_stats_r2: &SeqkitMetrics,
) -> Result<()> {
    let reads_in = input_stats_r1.reads + input_stats_r2.reads;
    if report.reads_in != reads_in {
        return Err(anyhow!(
            "extract_umis report reads_in observed mismatch: expected {}, observed {}",
            reads_in,
            report.reads_in
        ));
    }
    let reads_out = output_stats_r1.reads + output_stats_r2.reads;
    if report.reads_out != reads_out {
        return Err(anyhow!(
            "extract_umis report reads_out observed mismatch: expected {}, observed {}",
            reads_out,
            report.reads_out
        ));
    }
    let bases_in = input_stats_r1.bases + input_stats_r2.bases;
    if report.bases_in != bases_in {
        return Err(anyhow!(
            "extract_umis report bases_in observed mismatch: expected {}, observed {}",
            bases_in,
            report.bases_in
        ));
    }
    let bases_out = output_stats_r1.bases + output_stats_r2.bases;
    if report.bases_out != bases_out {
        return Err(anyhow!(
            "extract_umis report bases_out observed mismatch: expected {}, observed {}",
            bases_out,
            report.bases_out
        ));
    }
    let pairs_in = Some(input_stats_r1.reads.min(input_stats_r2.reads));
    if report.pairs_in != pairs_in {
        return Err(anyhow!(
            "extract_umis report pairs_in observed mismatch: expected {:?}, observed {:?}",
            pairs_in,
            report.pairs_in
        ));
    }
    let pairs_out = Some(output_stats_r1.reads.min(output_stats_r2.reads));
    if report.pairs_out != pairs_out {
        return Err(anyhow!(
            "extract_umis report pairs_out observed mismatch: expected {:?}, observed {:?}",
            pairs_out,
            report.pairs_out
        ));
    }
    Ok(())
}

fn validate_umi_report_read_semantics(report: &ExtractUmisReportV1) -> Result<()> {
    if report.reads_with_umi != report.reads_out {
        return Err(anyhow!(
            "extract_umis report reads_with_umi mismatch: expected {}, observed {}",
            report.reads_out,
            report.reads_with_umi
        ));
    }
    if report.reads_out > report.reads_in {
        return Err(anyhow!(
            "extract_umis report reads_out exceeds reads_in: reads_in={} reads_out={}",
            report.reads_in,
            report.reads_out
        ));
    }
    Ok(())
}

fn validate_umi_report_metrics(
    report: &ExtractUmisReportV1,
    metrics: &FastqUmiMetrics,
) -> Result<()> {
    if report.reads_in != metrics.reads_in {
        return Err(anyhow!(
            "extract_umis report reads_in mismatch: expected {}, observed {}",
            metrics.reads_in,
            report.reads_in
        ));
    }
    if report.reads_out != metrics.reads_out {
        return Err(anyhow!(
            "extract_umis report reads_out mismatch: expected {}, observed {}",
            metrics.reads_out,
            report.reads_out
        ));
    }
    if report.bases_in != metrics.bases_in {
        return Err(anyhow!(
            "extract_umis report bases_in mismatch: expected {}, observed {}",
            metrics.bases_in,
            report.bases_in
        ));
    }
    if report.bases_out != metrics.bases_out {
        return Err(anyhow!(
            "extract_umis report bases_out mismatch: expected {}, observed {}",
            metrics.bases_out,
            report.bases_out
        ));
    }
    if report.pairs_in != metrics.pairs_in {
        return Err(anyhow!(
            "extract_umis report pairs_in mismatch: expected {:?}, observed {:?}",
            metrics.pairs_in,
            report.pairs_in
        ));
    }
    if report.pairs_out != metrics.pairs_out {
        return Err(anyhow!(
            "extract_umis report pairs_out mismatch: expected {:?}, observed {:?}",
            metrics.pairs_out,
            report.pairs_out
        ));
    }
    if report.reads_with_umi != metrics.reads_with_umi {
        return Err(anyhow!(
            "extract_umis report reads_with_umi mismatch: expected {}, observed {}",
            metrics.reads_with_umi,
            report.reads_with_umi
        ));
    }
    Ok(())
}

fn validate_umi_report_paths(
    report: &ExtractUmisReportV1,
    input_r1: &Path,
    input_r2: &Path,
    artifacts: &UmiArtifacts,
) -> Result<()> {
    validate_umi_path_field("input_r1", input_r1, Path::new(&report.input_r1))?;
    let report_input_r2 = report
        .input_r2
        .as_deref()
        .ok_or_else(|| anyhow!("extract_umis paired report missing input_r2"))?;
    validate_umi_path_field("input_r2", input_r2, Path::new(report_input_r2))?;
    validate_umi_path_field("output_r1", &artifacts.output_r1, Path::new(&report.output_r1))?;
    let report_output_r2 = report
        .output_r2
        .as_deref()
        .ok_or_else(|| anyhow!("extract_umis paired report missing output_r2"))?;
    validate_umi_path_field("output_r2", &artifacts.output_r2, Path::new(report_output_r2))?;
    validate_umi_path_field("report_json", &artifacts.report_json, Path::new(&report.report_json))
}

fn validate_umi_path_field(label: &str, expected: &Path, observed: &Path) -> Result<()> {
    if observed != expected {
        return Err(anyhow!(
            "extract_umis report {label} mismatch: expected {}, observed {}",
            expected.display(),
            observed.display()
        ));
    }
    Ok(())
}

fn validate_umi_written_artifacts(
    artifacts: &UmiArtifacts,
    out_dir: &Path,
    report: &ExtractUmisReportV1,
) -> Result<()> {
    let metrics_json = out_dir.join("metrics.json");
    for path in [
        artifacts.output_r1.as_path(),
        artifacts.output_r2.as_path(),
        artifacts.report_json.as_path(),
        metrics_json.as_path(),
    ] {
        validate_umi_nonempty_artifact(path)?;
    }
    if let Some(raw_backend_report) = report.raw_backend_report.as_ref() {
        validate_umi_nonempty_artifact(Path::new(raw_backend_report))?;
    }
    Ok(())
}

fn validate_umi_nonempty_artifact(path: &Path) -> Result<()> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("read extract_umis artifact {}", path.display()))?;
    if metadata.len() == 0 {
        return Err(anyhow!("extract_umis artifact is empty: {}", path.display()));
    }
    Ok(())
}

fn write_umi_report(report_json: &Path, report: &ExtractUmisReportV1) -> Result<()> {
    bijux_dna_infra::atomic_write_json(report_json, report).context("write umi report")
}

fn write_umi_metrics(
    out_dir: &std::path::Path,
    metric_set: &MetricSet<FastqUmiMetrics>,
) -> Result<()> {
    let metrics_json = serde_json::to_value(metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write umi metrics")
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

fn umi_input_hash(r1: &Path, r2: &Path) -> Result<String> {
    let r1_hash = hash_file_sha256(r1).context("hash umi input r1")?;
    let r2_hash = hash_file_sha256(r2).context("hash umi input r2")?;
    params_hash(&serde_json::json!({ "r1": r1_hash, "r2": r2_hash }))
        .context("combine paired umi input hashes")
}

fn required_output_path<'a>(plan: &'a StagePlanV1, artifact_name: &str) -> Result<&'a Path> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_name)
        .map(|artifact| artifact.path.as_path())
        .ok_or_else(|| anyhow!("extract_umis plan missing `{artifact_name}` output"))
}

/// Materialize the governed local-smoke `fastq.extract_umis` artifacts.
///
/// The written summary artifact lives at `target/local-smoke/fastq.extract_umis/report.json`
/// under the active repository root, alongside top-level UMI-tagged FASTQ outputs.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_extract_umis_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_fastq::stage_api::local_extract_umis_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "governed fastq.extract_umis local smoke must resolve exactly one case"
        ));
    };

    let output_root = repo_root.join("target/local-smoke/fastq.extract_umis");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let summary = materialize_local_extract_umis_smoke_case(&repo_root, case, &output_root)?;
    let report_path = output_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    Ok(report_path)
}

fn materialize_local_extract_umis_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_fastq::LocalExtractUmisSmokeCasePlan,
    output_root: &Path,
) -> Result<LocalExtractUmisSmokeReport> {
    let effective_params = serde_json::from_value::<bijux_dna_domain_fastq::FastqUmiParams>(
        case.plan.effective_params.clone(),
    )
    .map_err(|error| anyhow!("decode extract-umis local-smoke effective params: {error}"))?;

    let input_r1 = repo_root.join(&case.r1);
    let input_r2 = repo_root.join(&case.r2);
    let case_output_r1 =
        resolve_smoke_path(repo_root, required_output_path(&case.plan, "umi_reads_r1")?);
    let case_output_r2 =
        resolve_smoke_path(repo_root, required_output_path(&case.plan, "umi_reads_r2")?);
    let case_report_json =
        resolve_smoke_path(repo_root, required_output_path(&case.plan, "report_json")?);
    let raw_backend_report =
        resolve_smoke_path(repo_root, &required_param_path(&case.plan, "raw_backend_report")?);

    for path in [&case_output_r1, &case_output_r2, &case_report_json, &raw_backend_report] {
        if let Some(parent) = path.parent() {
            bijux_dna_infra::ensure_dir(parent)?;
        }
    }

    let mut report = bijux_dna_domain_fastq::stages::contract::extract_umis(
        &input_r1,
        Some(&input_r2),
        &effective_params,
        &case_output_r1,
        Some(&case_output_r2),
        &case_report_json,
        Some(&raw_backend_report),
    )?;

    write_governed_local_smoke_log(
        &raw_backend_report,
        case.plan.tool_id.as_str(),
        &effective_params,
        &report,
    )?;

    report.input_r1 = case.r1.display().to_string();
    report.input_r2 = Some(case.r2.display().to_string());
    report.output_r1 = path_relative_to_repo(repo_root, &case_output_r1);
    report.output_r2 = Some(path_relative_to_repo(repo_root, &case_output_r2));
    report.report_json = path_relative_to_repo(repo_root, &case_report_json);
    report.raw_backend_report = Some(path_relative_to_repo(repo_root, &raw_backend_report));
    report.raw_backend_report_format = Some("governed_local_smoke_log".to_string());
    bijux_dna_infra::atomic_write_json(&case_report_json, &report)?;

    let top_level_r1 = output_root.join("umi_extracted.fastq.gz");
    let top_level_r2 = output_root.join("umi_extracted_R2.fastq.gz");
    copy_smoke_artifact(&case_output_r1, &top_level_r1)?;
    copy_smoke_artifact(&case_output_r2, &top_level_r2)?;

    Ok(LocalExtractUmisSmokeReport {
        schema_version: LOCAL_EXTRACT_UMIS_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "fastq.extract_umis".to_string(),
        sample_id: case.sample_id.clone(),
        planned_tool_id: case.plan.tool_id.as_str().to_string(),
        report_tool_id: report.tool_id,
        umi_pattern: case.umi_pattern.clone(),
        extracted_umi_count: report.reads_with_umi,
        invalid_umi_count: report.failed_extractions.unwrap_or(0),
        tag_header_format: read_name_transform_literal(&case.read_name_transform).to_string(),
        umi_extracted_fastq_gz: path_relative_to_repo(repo_root, &top_level_r1),
        umi_extracted_r2_fastq_gz: path_relative_to_repo(repo_root, &top_level_r2),
        case_report_json: path_relative_to_repo(repo_root, &case_report_json),
        raw_backend_report: path_relative_to_repo(repo_root, &raw_backend_report),
    })
}

fn resolve_smoke_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn required_param_path(plan: &StagePlanV1, key: &str) -> Result<PathBuf> {
    let raw = plan
        .params
        .get(key)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("extract_umis plan missing `{key}` path parameter"))?;
    Ok(PathBuf::from(raw))
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn copy_smoke_artifact(source: &Path, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    std::fs::copy(source, destination).map(|_| ()).map_err(Into::into)
}

fn read_name_transform_literal(transform: &UmiReadNameTransform) -> &'static str {
    match transform {
        UmiReadNameTransform::AppendToHeader => "append_to_header",
        UmiReadNameTransform::ReplaceHeader => "replace_header",
        UmiReadNameTransform::None => "none",
    }
}

fn write_governed_local_smoke_log(
    path: &Path,
    planned_tool_id: &str,
    params: &bijux_dna_domain_fastq::FastqUmiParams,
    report: &ExtractUmisReportV1,
) -> Result<()> {
    let mut file = std::fs::File::create(path)?;
    writeln!(file, "governed_local_smoke_runtime=bijux")?;
    writeln!(file, "planned_tool_id={planned_tool_id}")?;
    writeln!(file, "umi_pattern={}", params.umi_pattern.as_deref().unwrap_or("NNNNNNNN"))?;
    writeln!(
        file,
        "read_name_transform={}",
        read_name_transform_literal(&params.read_name_transform)
    )?;
    writeln!(file, "reads_with_umi={}", report.reads_with_umi)?;
    writeln!(file, "failed_extractions={}", report.failed_extractions.unwrap_or(0))?;
    Ok(())
}
