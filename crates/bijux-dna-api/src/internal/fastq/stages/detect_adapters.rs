use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
        detect_adapters::{
            AdapterEvidenceFormat, AdapterEvidenceScope, AdapterInspectionMode,
            DetectAdaptersEffectiveParams, DETECT_ADAPTERS_SCHEMA_VERSION,
        },
        PairedMode,
    },
    DetectAdaptersReportV1,
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
use serde::{Deserialize, Serialize};

const LOCAL_DETECT_ADAPTERS_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.detect_adapters.local_smoke.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LocalDetectAdaptersSmokeStatus {
    AdapterDetected,
    BelowThreshold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalDetectAdaptersSmokeCaseReport {
    sample_id: String,
    layout: PairedMode,
    input_r1: String,
    input_r2: Option<String>,
    adapter_status: LocalDetectAdaptersSmokeStatus,
    adapter_report: String,
    candidate_adapter_count: u64,
    detected_adapter_ids: Vec<String>,
    detection_confidence: Option<f64>,
    detection_threshold: Option<f64>,
    adapter_trimmed_fraction: Option<f64>,
    recommended_adapter_preset: Option<String>,
    adapter_evidence_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalDetectAdaptersSmokeReport {
    schema_version: String,
    stage_id: String,
    case_count: u64,
    detected_case_count: u64,
    below_threshold_case_count: u64,
    cases: Vec<LocalDetectAdaptersSmokeCaseReport>,
}

/// Materialize the governed local-smoke `fastq.detect_adapters` report bundle.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_detect_adapters_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_fastq::stage_api::local_detect_adapters_smoke_plans(&repo_root)?;
    let output_root = repo_root.join("target/local-smoke/fastq.detect_adapters");
    bijux_dna_infra::ensure_dir(&output_root)?;

    let case_reports = cases
        .iter()
        .map(|case| materialize_local_detect_adapters_smoke_case(&repo_root, case))
        .collect::<Result<Vec<_>>>()?;

    let summary = LocalDetectAdaptersSmokeReport {
        schema_version: LOCAL_DETECT_ADAPTERS_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: STAGE_DETECT_ADAPTERS.as_str().to_string(),
        case_count: case_reports.len() as u64,
        detected_case_count: case_reports
            .iter()
            .filter(|case| case.adapter_status == LocalDetectAdaptersSmokeStatus::AdapterDetected)
            .count() as u64,
        below_threshold_case_count: case_reports
            .iter()
            .filter(|case| case.adapter_status == LocalDetectAdaptersSmokeStatus::BelowThreshold)
            .count() as u64,
        cases: case_reports,
    };

    let report_path = output_root.join("adapters.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    Ok(report_path)
}

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

fn materialize_local_detect_adapters_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_fastq::LocalDetectAdaptersSmokeCasePlan,
) -> Result<LocalDetectAdaptersSmokeCaseReport> {
    let case_out_dir = resolve_plan_dir(repo_root, &case.plan.out_dir);
    let report_json = case_out_dir.join("adapter_report.json");
    let adapter_evidence_dir = case_out_dir.join("fastqc");
    let raw_backend_report = adapter_evidence_dir.join("normalized_adapter_evidence.json");
    bijux_dna_infra::ensure_dir(&adapter_evidence_dir)?;

    let effective_params = DetectAdaptersEffectiveParams {
        schema_version: DETECT_ADAPTERS_SCHEMA_VERSION.to_string(),
        paired_mode: if case.r2.is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads: case.plan.resources.threads,
        sample_reads: None,
        inspection_mode: AdapterInspectionMode::EvidenceOnly,
        report_only: true,
        evidence_engine: case.plan.tool_id.as_str().to_string(),
        evidence_scope: AdapterEvidenceScope::FullInput,
        evidence_format: AdapterEvidenceFormat::FastqcSummary,
        evidence_artifact_id: "report_json".to_string(),
    };

    let r1 = repo_root.join(&case.r1);
    let r2 = case.r2.as_ref().map(|path| repo_root.join(path));
    let report = bijux_dna_domain_fastq::stages::detect_adapters(
        &r1,
        r2.as_deref(),
        &effective_params,
        &report_json,
        &adapter_evidence_dir,
        Some(&raw_backend_report),
    )?;
    bijux_dna_infra::atomic_write_json(&report_json, &report)?;
    write_local_detect_adapters_evidence(&raw_backend_report, &report)?;

    Ok(LocalDetectAdaptersSmokeCaseReport {
        sample_id: case.sample_id.clone(),
        layout: if case.r2.is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        input_r1: case.r1.display().to_string(),
        input_r2: case.r2.as_ref().map(|path| path.display().to_string()),
        adapter_status: if report.candidate_adapter_count > 0 {
            LocalDetectAdaptersSmokeStatus::AdapterDetected
        } else {
            LocalDetectAdaptersSmokeStatus::BelowThreshold
        },
        adapter_report: path_relative_to_repo(repo_root, &report_json),
        candidate_adapter_count: report.candidate_adapter_count,
        detected_adapter_ids: report.detected_adapter_ids.clone(),
        detection_confidence: report.detection_confidence,
        detection_threshold: report.detection_threshold,
        adapter_trimmed_fraction: report.adapter_trimmed_fraction,
        recommended_adapter_preset: report.recommended_adapter_preset.clone(),
        adapter_evidence_dir: path_relative_to_repo(repo_root, &adapter_evidence_dir),
    })
}

fn write_local_detect_adapters_evidence(
    evidence_path: &Path,
    report: &DetectAdaptersReportV1,
) -> Result<()> {
    let evidence = serde_json::json!({
        "schema_version": "bijux.fastq.detect_adapters.evidence.v2",
        "candidate_adapter_count": report.candidate_adapter_count,
        "detected_adapter_ids": report.detected_adapter_ids,
        "detection_confidence": report.detection_confidence,
        "detection_threshold": report.detection_threshold,
        "adapter_trimmed_fraction": report.adapter_trimmed_fraction,
        "recommended_adapter_preset": report.recommended_adapter_preset,
        "detected_adapter_source": report.detected_adapter_source,
    });
    Ok(bijux_dna_infra::atomic_write_json(evidence_path, &evidence)?)
}

fn resolve_plan_dir(repo_root: &Path, out_dir: &Path) -> PathBuf {
    if out_dir.is_absolute() {
        out_dir.to_path_buf()
    } else {
        repo_root.join(out_dir)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
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

    write_detect_adapters_artifacts(&inputs.tool_plan.plan.out_dir, &report, &metric_set)?;

    let context = build_detect_adapters_context(inputs);
    let record = BenchmarkRecord {
        context,
        execution: detect_adapters_execution_metrics(inputs.execution),
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
        adapter_report: Some(report.report_json.clone()),
        candidate_adapter_count: report.candidate_adapter_count,
        detected_adapter_ids: report.detected_adapter_ids.clone(),
        detection_confidence: report.detection_confidence,
        detection_threshold: report.detection_threshold,
        adapter_trimmed_fraction: report.adapter_trimmed_fraction,
    };
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;
    Ok(metric_set)
}

fn write_detect_adapters_artifacts(
    out_dir: &std::path::Path,
    report: &DetectAdaptersReportV1,
    metric_set: &MetricSet<FastqDetectAdaptersMetrics>,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(&out_dir.join("adapter_report.json"), report)
        .context("write adapter report")?;
    let metrics_json = serde_json::to_value(metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write adapter metrics")
}

fn build_detect_adapters_context(
    inputs: &DetectRecordInputs<'_>,
) -> bijux_dna_analyze::BenchmarkContext {
    build_benchmark_context(
        &inputs.tool_plan.tool,
        inputs.tool_plan.tool_spec.tool_version.clone(),
        inputs.tool_plan.image_digest.clone(),
        inputs.bench_inputs.runner,
        inputs.platform,
        inputs.input_hash.to_string(),
        inputs.tool_plan.plan.params.clone(),
    )
}

fn detect_adapters_execution_metrics(execution: &DetectAdaptersToolExecution) -> ExecutionMetrics {
    ExecutionMetrics {
        runtime_s: execution.result.runtime_s,
        memory_mb: execution.result.memory_mb,
        exit_code: execution.result.exit_code,
    }
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
    let report_json = out_dir.join("adapter_report.json");
    let adapter_evidence_dir = out_dir.join("fastqc");
    let raw_backend_report = out_dir.join("fastqc").join("fastqc_data.txt");
    let effective_params = DetectAdaptersEffectiveParams {
        schema_version: DETECT_ADAPTERS_SCHEMA_VERSION.to_string(),
        paired_mode: if input_stats_r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        threads: tool_spec.resources.threads,
        sample_reads: None,
        inspection_mode: AdapterInspectionMode::EvidenceOnly,
        report_only: true,
        evidence_engine: tool.to_string(),
        evidence_scope: AdapterEvidenceScope::FullInput,
        evidence_format: AdapterEvidenceFormat::FastqcSummary,
        evidence_artifact_id: "report_json".to_string(),
    };
    let mut report = bijux_dna_domain_fastq::stages::detect_adapters(
        &bench_inputs.r1,
        input_r2_path,
        &effective_params,
        &report_json,
        &adapter_evidence_dir,
        raw_backend_report.exists().then_some(raw_backend_report.as_path()),
    )?;
    report.runtime_s = Some(execution.runtime_s);
    report.memory_mb = Some(execution.memory_mb);
    report.exit_code = Some(execution.exit_code);
    Ok(report)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{build_detect_adapters_metric_set, build_detect_report, TrimBenchInputs};
    use bijux_dna_core::prelude::measure::SeqkitMetrics;
    use bijux_dna_core::prelude::{
        CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
    };
    use bijux_dna_domain_fastq::{
        DetectAdaptersReportV1, PairedMode, DETECT_ADAPTERS_REPORT_SCHEMA_VERSION,
    };
    use bijux_dna_environment::api::RuntimeKind;
    use bijux_dna_runner::step_runner::StageResultV1;

    fn dummy_tool(tool_id: &'static str, threads: u32) -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::from_static(tool_id),
            tool_version: "1.0.0".to_string(),
            image: ContainerImageRefV1 { image: format!("{tool_id}:latest"), digest: None },
            command: CommandSpecV1 { template: vec![tool_id.to_string()] },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads,
            },
        }
    }

    #[test]
    fn build_detect_adapters_metric_set_retains_adapter_contract_fields() {
        let report = DetectAdaptersReportV1 {
            schema_version: DETECT_ADAPTERS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.detect_adapters".to_string(),
            stage_id: "fastq.detect_adapters".to_string(),
            tool_id: "fastqc".to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 4,
            inspection_mode: bijux_dna_domain_fastq::params::detect_adapters::AdapterInspectionMode::EvidenceOnly,
            report_only: true,
            evidence_engine: "fastqc".to_string(),
            evidence_scope: bijux_dna_domain_fastq::params::detect_adapters::AdapterEvidenceScope::FullInput,
            evidence_format: bijux_dna_domain_fastq::params::detect_adapters::AdapterEvidenceFormat::FastqcSummary,
            evidence_artifact_id: "report_json".to_string(),
            detected_adapter_source: "normalized_fastqc_evidence".to_string(),
            detected_adapter_ids: vec!["truseq_universal".to_string()],
            detection_confidence: Some(0.25),
            detection_threshold: Some(0.01),
            input_r1: "reads.fastq.gz".to_string(),
            input_r2: None,
            report_json: "out/adapter_report.json".to_string(),
            adapter_evidence_dir: "out/fastqc".to_string(),
            recommended_adapter_bank_id: Some("bijux-dna-fastq-adapter-bank".to_string()),
            recommended_adapter_bank_hash: Some("sha256:test".to_string()),
            recommended_adapter_preset: Some("illumina-default".to_string()),
            reads_in: 100,
            reads_out: 100,
            bases_in: 10_000,
            bases_out: 10_000,
            pairs_in: None,
            pairs_out: None,
            mean_q: 31.0,
            candidate_adapter_count: 1,
            adapter_trimmed_fraction: Some(0.25),
            adapter_content_max: Some(0.25),
            adapter_content_mean: Some(0.10),
            duplication_rate: Some(0.0),
            n_rate: Some(0.0),
            kmer_warning_count: Some(0),
            overrepresented_sequence_count: Some(0),
            runtime_s: Some(2.0),
            memory_mb: Some(64.0),
            exit_code: Some(0),
            raw_backend_report: Some("out/fastqc/fastqc_data.txt".to_string()),
            raw_backend_report_format: Some("fastqc_normalized".to_string()),
        };

        let metric_set =
            build_detect_adapters_metric_set(&report).expect("metric set should validate");
        assert_eq!(metric_set.metrics.adapter_report.as_deref(), Some("out/adapter_report.json"));
        assert_eq!(metric_set.metrics.detected_adapter_ids, vec!["truseq_universal".to_string()]);
        assert_eq!(metric_set.metrics.detection_confidence, Some(0.25));
        assert_eq!(metric_set.metrics.detection_threshold, Some(0.01));
    }

    #[test]
    fn build_detect_report_uses_governed_adapter_identity_fields() {
        let temp = bijux_dna_infra::temp_dir("bijux-api-detect-adapters").expect("temp dir");
        let r1 = temp.path().join("reads.fastq");
        std::fs::write(
            &r1,
            "@r1\nACGTAGATCGGAAGAGCTTT\n+\nIIIIIIIIIIIIIIIIIIII\n@r2\nACGTACGT\n+\nIIIIIIII\n",
        )
        .expect("write fastq");

        let out_dir = temp.path().join("out");
        std::fs::create_dir_all(out_dir.join("fastqc")).expect("create fastqc dir");
        std::fs::write(out_dir.join("fastqc/fastqc_data.txt"), "adapter content")
            .expect("write raw report");

        let bench_inputs = TrimBenchInputs {
            runner: RuntimeKind::Docker,
            r1: r1.clone(),
            input_hash: "hash".to_string(),
            input_stats: SeqkitMetrics { reads: 2, bases: 28, mean_q: 31.0, gc_percent: 0.5 },
            bench_dir: temp.path().join("bench"),
            tools_root: temp.path().join("tools"),
        };
        let report = build_detect_report(
            &bench_inputs,
            None,
            None,
            "fastqc",
            &dummy_tool("fastqc", 4),
            &out_dir,
            &StageResultV1 {
                run_id: "run-1".to_string(),
                exit_code: 0,
                runtime_s: 3.5,
                memory_mb: 72.0,
                outputs: vec![out_dir.join("adapter_report.json")],
                metrics_path: None,
                stdout: String::new(),
                stderr: String::new(),
                command: "fastqc".to_string(),
            },
        )
        .expect("build report");

        assert_eq!(report.report_json, out_dir.join("adapter_report.json").display().to_string());
        assert_eq!(report.detected_adapter_ids, vec!["truseq_universal".to_string()]);
        assert_eq!(report.detection_confidence, Some(0.5));
        assert_eq!(report.detection_threshold, Some(0.5));
        assert_eq!(report.raw_backend_report_format.as_deref(), Some("fastqc_normalized"));
    }
}
