use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::query_shared::{fetch_fastq_trim_v2, insert_fastq_trim_v2};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqTrimMetrics};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::params::trim::TrimEffectiveParams;
use bijux_dna_domain_fastq::TrimReadsReportV1;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::select_trim_tools;
use bijux_dna_planner_fastq::stage_api::fastq::trim_reads::{
    plan_with_options, validate_trim_toolset_support, TrimPlanOptions,
};
use bijux_dna_planner_fastq::stage_api::{
    adapter_bank_context, contaminant_bank_context, inspect_headers, log_header_warnings,
    polyx_bank_context, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use bijux_dna_stage_contract::StagePlanV1;

use super::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context, derive_trim_delta, json_string,
    observe_fastq_stats, prepare_trim_bench, TrimBenchInputs,
};
use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_TRIM_READS,
};
use serde::Serialize;

mod policy;

use self::policy::{
    adapter_bank_requested, adapter_policy_uses_bank, benchmark_query_context,
    contaminant_policy_uses_bank, normalized_adapter_policy, normalized_contaminant_policy,
    normalized_polyx_policy, polyx_policy_uses_bank,
};

const LOCAL_TRIM_READS_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.trim_reads.local_smoke.report.v1";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
enum LocalTrimReadsSmokeLayout {
    SingleEnd,
    PairedEnd,
}

#[derive(Debug, Clone, Serialize)]
struct LocalTrimReadsSmokeCaseReport {
    sample_id: String,
    layout: LocalTrimReadsSmokeLayout,
    input_r1: String,
    input_r2: Option<String>,
    input_read_count_total: u64,
    input_read_count_r1: u64,
    input_read_count_r2: Option<u64>,
    input_pair_count: Option<u64>,
    output_read_count_total: u64,
    output_read_count_r1: u64,
    output_read_count_r2: Option<u64>,
    output_pair_count: Option<u64>,
    reads_retained: u64,
    reads_dropped: u64,
    read_count_not_greater_than_input: bool,
    min_length: u32,
    quality_cutoff: Option<u32>,
    bases_removed: u64,
    trimmed_reads_r1: String,
    trimmed_reads_r2: Option<String>,
    report_json: String,
    raw_backend_report: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct LocalTrimReadsSmokeReport {
    schema_version: String,
    stage_id: String,
    case_count: u64,
    all_cases_passed: bool,
    cases: Vec<LocalTrimReadsSmokeCaseReport>,
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

fn load_governed_trim_report(report_path: &std::path::Path) -> Result<TrimReadsReportV1> {
    let raw = std::fs::read_to_string(report_path)
        .with_context(|| format!("read governed trim report {}", report_path.display()))?;
    bijux_dna_domain_fastq::observer::parse_trim_reads_report(&raw)
        .with_context(|| format!("parse governed trim report {}", report_path.display()))
}

fn write_governed_trim_report(
    report_path: &std::path::Path,
    report: &TrimReadsReportV1,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(report_path, report)
        .with_context(|| format!("write governed trim report {}", report_path.display()))
}

/// Materialize the governed local-smoke `fastq.trim_reads` report bundle.
///
/// The written summary artifact lives at `target/local-smoke/fastq.trim_reads/report.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_trim_reads_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_fastq::stage_api::local_trim_reads_smoke_plans(&repo_root)?;
    let output_root = repo_root.join("target/local-smoke/fastq.trim_reads");
    bijux_dna_infra::ensure_dir(&output_root)?;

    let case_reports = cases
        .iter()
        .map(|case| materialize_local_trim_reads_smoke_case(&repo_root, case))
        .collect::<Result<Vec<_>>>()?;

    let summary = LocalTrimReadsSmokeReport {
        schema_version: LOCAL_TRIM_READS_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: STAGE_TRIM_READS.as_str().to_string(),
        case_count: case_reports.len() as u64,
        all_cases_passed: case_reports.iter().all(|case| case.read_count_not_greater_than_input),
        cases: case_reports,
    };

    let report_path = output_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    Ok(report_path)
}

fn materialize_local_trim_reads_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_fastq::LocalTrimReadsSmokeCasePlan,
) -> Result<LocalTrimReadsSmokeCaseReport> {
    let effective_params =
        serde_json::from_value::<TrimEffectiveParams>(case.plan.effective_params.clone())
            .context("decode trim reads local-smoke effective params")?;
    let input_r1 = repo_root.join(&case.r1);
    let input_r2 = case.r2.as_ref().map(|path| repo_root.join(path));
    let output_r1 = resolve_plan_output_path(
        repo_root,
        &required_plan_output_path(&case.plan, "trimmed_reads_r1")?,
    );
    let output_r2 = case
        .r2
        .as_ref()
        .map(|_| required_plan_output_path(&case.plan, "trimmed_reads_r2"))
        .transpose()?
        .map(|path| resolve_plan_output_path(repo_root, &path));
    let report_path =
        resolve_plan_output_path(repo_root, &required_plan_output_path(&case.plan, "report_json")?);
    let raw_backend_report = optional_plan_output_path(&case.plan, "raw_backend_report_json")
        .or_else(|| optional_plan_output_path(&case.plan, "raw_backend_report_txt"))
        .map(|path| resolve_plan_output_path(repo_root, &path));

    for path in [&output_r1, &report_path] {
        if let Some(parent) = path.parent() {
            bijux_dna_infra::ensure_dir(parent)?;
        }
    }
    if let Some(output_r2) = output_r2.as_ref() {
        if let Some(parent) = output_r2.parent() {
            bijux_dna_infra::ensure_dir(parent)?;
        }
    }
    if let Some(raw_backend_report) = raw_backend_report.as_ref() {
        if let Some(parent) = raw_backend_report.parent() {
            bijux_dna_infra::ensure_dir(parent)?;
        }
    }

    let mut report = bijux_dna_domain_fastq::stages::trim_reads(
        &input_r1,
        input_r2.as_deref(),
        &effective_params,
        case.plan.tool_id.as_str(),
        &output_r1,
        output_r2.as_deref(),
        raw_backend_report.as_deref(),
    )?;
    if let Some(raw_backend_report) = raw_backend_report.as_ref() {
        write_local_trim_backend_report(raw_backend_report, case.plan.tool_id.as_str(), &report)?;
    }

    report.input_r1 = case.r1.display().to_string();
    report.input_r2 = case.r2.as_ref().map(|path| path.display().to_string());
    report.output_r1 = path_relative_to_repo(repo_root, &output_r1);
    report.output_r2 = output_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path));
    report.raw_backend_report =
        raw_backend_report.as_ref().map(|path| path_relative_to_repo(repo_root, path));
    write_governed_trim_report(&report_path, &report)?;

    let input_read_count_total = report.reads_in.unwrap_or(0);
    let output_read_count_total = report.reads_out.unwrap_or(0);
    let input_pair_count = report.pairs_in;
    let output_pair_count = report.pairs_out;
    let reads_retained = output_read_count_total;
    let reads_dropped = input_read_count_total.saturating_sub(output_read_count_total);
    let bases_removed = report.bases_in.unwrap_or(0).saturating_sub(report.bases_out.unwrap_or(0));
    let input_read_count_r2 = input_pair_count;
    let output_read_count_r2 = output_pair_count;
    let input_read_count_r1 =
        input_read_count_total.saturating_sub(input_read_count_r2.unwrap_or(0));
    let output_read_count_r1 =
        output_read_count_total.saturating_sub(output_read_count_r2.unwrap_or(0));

    Ok(LocalTrimReadsSmokeCaseReport {
        sample_id: case.sample_id.clone(),
        layout: if case.r2.is_some() {
            LocalTrimReadsSmokeLayout::PairedEnd
        } else {
            LocalTrimReadsSmokeLayout::SingleEnd
        },
        input_r1: case.r1.display().to_string(),
        input_r2: case.r2.as_ref().map(|path| path.display().to_string()),
        input_read_count_total,
        input_read_count_r1,
        input_read_count_r2,
        input_pair_count,
        output_read_count_total,
        output_read_count_r1,
        output_read_count_r2,
        output_pair_count,
        reads_retained,
        reads_dropped,
        read_count_not_greater_than_input: output_read_count_total <= input_read_count_total,
        min_length: effective_params.min_len,
        quality_cutoff: effective_params.q_cutoff,
        bases_removed,
        trimmed_reads_r1: path_relative_to_repo(repo_root, &output_r1),
        trimmed_reads_r2: output_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        report_json: path_relative_to_repo(repo_root, &report_path),
        raw_backend_report: raw_backend_report
            .as_ref()
            .map(|path| path_relative_to_repo(repo_root, path)),
    })
}

fn write_local_trim_backend_report(
    path: &Path,
    tool_id: &str,
    report: &TrimReadsReportV1,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    match tool_id {
        "fastp" => bijux_dna_infra::write_bytes(
            path,
            serde_json::json!({
                "summary": {
                    "before_filtering": {
                        "total_reads": report.reads_in,
                        "total_bases": report.bases_in,
                    },
                    "after_filtering": {
                        "total_reads": report.reads_out,
                        "total_bases": report.bases_out,
                    }
                },
                "filtering_result": {
                    "passed_filter_reads": report.reads_out,
                    "too_short_reads": report
                        .reads_in
                        .zip(report.reads_out)
                        .map(|(before, after)| before.saturating_sub(after)),
                },
                "adapter_cutting": {
                    "adapter_trimmed_reads": report.reads_out,
                },
                "quality_cutoff": report.quality_cutoff,
                "min_length": report.min_length,
            })
            .to_string(),
        )
        .with_context(|| format!("write local trim backend report {}", path.display())),
        _ => Err(anyhow!(
            "local-smoke fastq.trim_reads does not support backend report materialization for tool `{tool_id}`"
        )),
    }
}

fn resolve_plan_output_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

fn required_plan_output_path(plan: &StagePlanV1, output_id: &str) -> Result<PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| {
            anyhow!(
                "trim_reads plan is missing governed output `{output_id}` for tool {}",
                plan.tool_id.as_str()
            )
        })
}

fn optional_plan_output_path(plan: &StagePlanV1, output_id: &str) -> Option<PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
}

/// # Errors
/// Returns an error if planning, execution, metric derivation, or persistence fails.
pub fn bench_fastq_trim<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimArgs,
) -> Result<BenchOutcome<FastqTrimMetrics>> {
    let tools = select_trim_benchmark_tools(args)?;
    let setup = prepare_trim_benchmark_setup(catalog, platform, runner_override, args, &tools)?;

    if args.explain {
        write_trim_benchmark_explain(&setup)?;
    }
    ensure_trim_benchmark_qa(catalog, platform, &setup.tools)?;

    let policy = resolve_trim_policy_context(args)?;
    validate_trim_toolset_support(&setup.tools, args.r2.is_some(), &policy.trim_options)?;

    let sqlite_path = setup.bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = setup.bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut records = Vec::<BenchmarkRecord<FastqTrimMetrics>>::new();
    let mut failures = Vec::<RawFailure>::new();

    for tool in setup.tools.clone() {
        let tool_plan =
            prepare_trim_tool_plan(catalog, platform, args, &setup, &policy, jobs, tool)?;
        if let Ok(Some(record)) = fetch_fastq_trim_v2(
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

        let execution = execute_trim_tool(&tool_plan, setup.bench_inputs.runner, jobs)?;

        if let Some(failure) = trim_tool_failure(&tool_plan, execution.exit_code) {
            failures.push(failure);
            continue;
        }

        let observed_stats = observe_trim_tool_stats(catalog, platform, args, &setup, &tool_plan)?;
        let report_path = tool_plan.plan.out_dir.join("trim_report.json");
        let governed_report =
            enrich_governed_trim_report(&report_path, &observed_stats, &execution)?;
        write_governed_trim_report(&report_path, &governed_report)?;
        let metrics = trim_metrics_from_report(&observed_stats, &governed_report);
        let metric_set = metric_set(metrics.clone());
        bijux_dna_analyze::validate_metric_set(&metric_set)?;
        let metrics_json = serde_json::to_value(&metric_set)?;
        let metrics_path = tool_plan.plan.out_dir.join("metrics.json");
        bijux_dna_infra::atomic_write_json(&metrics_path, &metrics_json)
            .context("write trim metrics")?;
        prune_trim_tool_payload(
            &tool_plan.plan.out_dir,
            &report_path,
            &metrics_path,
            &governed_report,
        )?;

        let record = build_trim_record(platform, &setup, &tool_plan, &execution, metric_set)?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_trim_v2(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: setup.bench_inputs.bench_dir,
        explain: args.explain,
    })
}

fn select_trim_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimArgs,
) -> Result<Vec<String>> {
    let tools = select_trim_tools(&args.tools, false)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_TRIM_READS.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_TRIM_READS.as_str(), &header);
    Ok(tools)
}

struct TrimBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    excluded_tools: Vec<String>,
    bench_inputs: TrimBenchInputs,
    input_hash: String,
    input_stats_r2: Option<SeqkitMetrics>,
}

struct TrimPolicyContext {
    trim_options: TrimPlanOptions,
    adapter_context: Option<serde_json::Value>,
    polyx_context: Option<serde_json::Value>,
    contaminant_context: Option<serde_json::Value>,
}

struct TrimToolPlan {
    tool: String,
    tool_spec: ToolExecutionSpecV1,
    plan: StagePlanV1,
    bench_params: serde_json::Value,
    params_hash: String,
    image_digest: String,
}

struct TrimObservedStats {
    before: SeqkitMetrics,
    after: SeqkitMetrics,
    pairs_in: Option<u64>,
    pairs_out: Option<u64>,
}

fn observe_trim_tool_stats<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimArgs,
    setup: &TrimBenchmarkSetup,
    tool_plan: &TrimToolPlan,
) -> Result<TrimObservedStats> {
    let output_r1 = tool_plan.plan.io.outputs[0].path.clone();
    let output_stats_r1 =
        observe_fastq_stats(catalog, platform, setup.bench_inputs.runner, &output_r1)?;
    let output_stats_r2 = if args.r2.is_some() {
        Some(observe_fastq_stats(
            catalog,
            platform,
            setup.bench_inputs.runner,
            &tool_plan.plan.io.outputs[1].path,
        )?)
    } else {
        None
    };
    let before =
        combine_seqkit_metrics(&setup.bench_inputs.input_stats, setup.input_stats_r2.as_ref());
    let after = combine_seqkit_metrics(&output_stats_r1, output_stats_r2.as_ref());
    let pairs_in = setup
        .input_stats_r2
        .as_ref()
        .map(|stats| setup.bench_inputs.input_stats.reads.min(stats.reads));
    let pairs_out = output_stats_r2.as_ref().map(|stats| output_stats_r1.reads.min(stats.reads));
    Ok(TrimObservedStats { before, after, pairs_in, pairs_out })
}

fn enrich_governed_trim_report(
    report_path: &Path,
    observed_stats: &TrimObservedStats,
    execution: &StageResultV1,
) -> Result<TrimReadsReportV1> {
    let mut report = load_governed_trim_report(report_path)?;
    report.reads_in = Some(observed_stats.before.reads);
    report.reads_out = Some(observed_stats.after.reads);
    report.bases_in = Some(observed_stats.before.bases);
    report.bases_out = Some(observed_stats.after.bases);
    report.pairs_in = observed_stats.pairs_in;
    report.pairs_out = observed_stats.pairs_out;
    report.mean_q_before = Some(observed_stats.before.mean_q);
    report.mean_q_after = Some(observed_stats.after.mean_q);
    report.runtime_s = Some(execution.runtime_s);
    report.memory_mb = Some(execution.memory_mb);
    Ok(report)
}

fn trim_metrics_from_report(
    observed_stats: &TrimObservedStats,
    report: &TrimReadsReportV1,
) -> FastqTrimMetrics {
    FastqTrimMetrics {
        reads_in: observed_stats.before.reads,
        reads_out: observed_stats.after.reads,
        bases_in: observed_stats.before.bases,
        bases_out: observed_stats.after.bases,
        pairs_in: observed_stats.pairs_in,
        pairs_out: observed_stats.pairs_out,
        mean_q_before: observed_stats.before.mean_q,
        mean_q_after: observed_stats.after.mean_q,
        delta_metrics: derive_trim_delta(&observed_stats.before, &observed_stats.after),
        paired_mode: Some(paired_mode_label(report).to_string()),
        adapter_policy: Some(report.adapter_policy.clone()),
        polyx_policy: report.polyx_policy.clone(),
        n_policy: report.n_policy.clone(),
        contaminant_policy: report.contaminant_policy.clone(),
        raw_backend_report_format: report.raw_backend_report_format.clone(),
        adapter_preset: report.adapter_preset.clone(),
        adapter_bank_id: report.adapter_bank_id.clone(),
        adapter_bank_hash: report.adapter_bank_hash.clone(),
        adapter_overrides: report.adapter_overrides.clone().map(Into::into),
    }
}

fn paired_mode_label(report: &TrimReadsReportV1) -> &'static str {
    match report.paired_mode {
        bijux_dna_domain_fastq::PairedMode::SingleEnd => "single_end",
        bijux_dna_domain_fastq::PairedMode::PairedEnd => "paired_end",
        bijux_dna_domain_fastq::PairedMode::Unknown => "not_declared",
    }
}

fn build_trim_record(
    platform: &PlatformSpec,
    setup: &TrimBenchmarkSetup,
    tool_plan: &TrimToolPlan,
    execution: &StageResultV1,
    metrics: bijux_dna_analyze::MetricSet<FastqTrimMetrics>,
) -> Result<BenchmarkRecord<FastqTrimMetrics>> {
    let context = build_benchmark_context(
        &tool_plan.tool,
        tool_plan.tool_spec.tool_version.clone(),
        tool_plan.image_digest.clone(),
        setup.bench_inputs.runner,
        platform,
        setup.input_hash.clone(),
        tool_plan.bench_params.clone(),
    );
    let record = BenchmarkRecord {
        context,
        execution: ExecutionMetrics {
            runtime_s: execution.runtime_s,
            memory_mb: execution.memory_mb,
            exit_code: execution.exit_code,
        },
        metrics,
    };
    record.validate()?;
    Ok(record)
}

fn prepare_trim_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimArgs,
    setup: &TrimBenchmarkSetup,
    policy: &TrimPolicyContext,
    jobs: usize,
    tool: String,
) -> Result<TrimToolPlan> {
    let out_dir = setup.bench_inputs.tools_root.join(&tool);
    bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
    let tool_spec = build_tool_execution_spec(
        STAGE_TRIM_READS.as_str(),
        &tool,
        &setup.registry,
        catalog,
        platform,
    )?;
    let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
    let tool_spec = apply_thread_override(&tool_spec, args.threads);
    let plan = plan_with_options(
        &tool_spec,
        &setup.bench_inputs.r1,
        args.r2.as_deref(),
        &out_dir,
        policy.adapter_context.as_ref(),
        policy.polyx_context.as_ref(),
        policy.contaminant_context.as_ref(),
        &policy.trim_options,
    )?;
    let bench_params = benchmark_query_context(
        policy.adapter_context.as_ref(),
        policy.polyx_context.as_ref(),
        policy.contaminant_context.as_ref(),
    )?
    .embed_in_parameters(&plan.params);
    let params_hash = stable_params_hash(&bench_params);
    let image_digest = benchmark_image_identity(&tool_spec);
    Ok(TrimToolPlan { tool, tool_spec, plan, bench_params, params_hash, image_digest })
}

fn execute_trim_tool(
    tool_plan: &TrimToolPlan,
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

fn trim_tool_failure(tool_plan: &TrimToolPlan, exit_code: i32) -> Option<RawFailure> {
    if exit_code == 0 {
        return None;
    }
    Some(RawFailure {
        stage: STAGE_TRIM_READS.as_str().to_string(),
        tool: tool_plan.tool.clone(),
        reason: format!("tool `{}` failed with status {exit_code}", tool_plan.tool),
        category: ErrorCategory::ToolError,
    })
}

fn resolve_trim_policy_context(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimArgs,
) -> Result<TrimPolicyContext> {
    let adapter_policy =
        normalized_adapter_policy(args.adapter_policy.as_deref(), adapter_bank_requested(args))?;
    let adapter_context = if adapter_policy_uses_bank(adapter_policy.as_deref()) {
        adapter_bank_context(
            args.adapter_bank_preset.as_deref(),
            args.adapter_bank.as_deref(),
            args.adapter_bank_file.as_deref(),
            &args.enable_adapters,
            &args.disable_adapters,
        )?
    } else {
        None
    };
    let polyx_policy =
        normalized_polyx_policy(args.polyx_policy.as_deref(), args.polyx_preset.is_some())?;
    let polyx_context = if polyx_policy_uses_bank(polyx_policy.as_deref()) {
        polyx_bank_context(args.polyx_preset.as_deref())?
    } else {
        None
    };
    let contaminant_policy = normalized_contaminant_policy(
        args.contaminant_policy.as_deref(),
        args.contaminant_preset.is_some(),
    )?;
    let contaminant_context = if contaminant_policy_uses_bank(contaminant_policy.as_deref()) {
        contaminant_bank_context(args.contaminant_preset.as_deref())?
    } else {
        None
    };
    let trim_options = TrimPlanOptions {
        threads: args.threads,
        min_length: args.min_length,
        quality_cutoff: args.quality_cutoff,
        n_policy: args.n_policy.clone(),
        adapter_policy,
        polyx_policy,
        contaminant_policy,
    };
    Ok(TrimPolicyContext { trim_options, adapter_context, polyx_context, contaminant_context })
}

fn prepare_trim_benchmark_setup<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimArgs,
    tools: &[String],
) -> Result<TrimBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_TRIM_READS.as_str(), tools, &registry, false)?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_TRIM_READS,
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
    let excluded_tools = excluded_trim_tools(&registry, &tools);
    Ok(TrimBenchmarkSetup {
        registry,
        tools,
        excluded_tools,
        bench_inputs,
        input_hash,
        input_stats_r2,
    })
}

fn excluded_trim_tools(registry: &ToolRegistry, selected_tools: &[String]) -> Vec<String> {
    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_TRIM_READS.as_str());
    registry
        .tools_for_stage(&stage_id)
        .iter()
        .map(|tool| tool.tool_id.to_string())
        .filter(|tool| !selected_tools.contains(tool))
        .collect()
}

fn write_trim_benchmark_explain(setup: &TrimBenchmarkSetup) -> Result<()> {
    write_explain_md(
        &setup.bench_inputs.bench_dir,
        STAGE_TRIM_READS.as_str(),
        &setup.tools,
        &setup.excluded_tools,
        None,
    )?;
    write_explain_plan_json(
        &setup.bench_inputs.bench_dir,
        STAGE_TRIM_READS.as_str(),
        &setup.tools,
        &setup.registry,
        None,
    )
}

fn ensure_trim_benchmark_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(STAGE_TRIM_READS.as_str(), tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_TRIM_READS.as_str(), tools, platform, catalog)
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

fn prune_trim_tool_payload(
    out_dir: &Path,
    report_path: &Path,
    metrics_path: &Path,
    report: &TrimReadsReportV1,
) -> Result<()> {
    let run_artifacts_dir = out_dir.join("run_artifacts");
    let mut keep = HashSet::new();
    keep.insert(report_path.to_path_buf());
    keep.insert(metrics_path.to_path_buf());
    if let Some(raw_backend_report) = report.raw_backend_report.as_ref() {
        keep.insert(Path::new(raw_backend_report).to_path_buf());
    }

    let mut dirs = vec![out_dir.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        for entry in
            fs::read_dir(&dir).with_context(|| format!("read trim tool dir {}", dir.display()))?
        {
            let path = entry.with_context(|| format!("read entry in {}", dir.display()))?.path();
            if path == run_artifacts_dir || path.starts_with(&run_artifacts_dir) {
                continue;
            }
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            if keep.contains(&path) {
                continue;
            }
            fs::remove_file(&path)
                .with_context(|| format!("prune trim payload {}", path.display()))?;
        }
    }

    Ok(())
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        adapter_bank_requested, adapter_policy_uses_bank, apply_thread_override,
        benchmark_query_context, contaminant_policy_uses_bank, normalized_adapter_policy,
        normalized_contaminant_policy, normalized_polyx_policy, polyx_policy_uses_bank,
        prune_trim_tool_payload, write_governed_trim_report,
    };
    use bijux_dna_core::prelude::{
        CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
    };
    use bijux_dna_domain_fastq::{PairedMode, TrimReadsReportV1, TRIM_READS_REPORT_SCHEMA_VERSION};
    use std::fs;

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
    fn benchmark_query_context_captures_governed_trim_bank_hashes() {
        let adapter_context = serde_json::json!({"bank_hash": "adapter-hash"});
        let polyx_context = serde_json::json!({"bank_hash": "polyx-hash"});
        let contaminant_context = serde_json::json!({"bank_hash": "contaminant-hash"});

        let context = benchmark_query_context(
            Some(&adapter_context),
            Some(&polyx_context),
            Some(&contaminant_context),
        )
        .unwrap_or_else(|err| panic!("query context: {err}"));

        assert!(context.stage_contract_hash.is_some());
        assert_eq!(
            context.bank_hashes.get("adapter_bank").map(String::as_str),
            Some("adapter-hash")
        );
        assert_eq!(context.bank_hashes.get("polyx_bank").map(String::as_str), Some("polyx-hash"));
        assert_eq!(
            context.bank_hashes.get("contaminant_bank").map(String::as_str),
            Some("contaminant-hash")
        );
    }

    #[test]
    fn implicit_trim_banks_stay_disabled_without_policy_or_explicit_selection() {
        assert_eq!(
            normalized_adapter_policy(None, false).unwrap_or_else(|err| panic!("{err}")),
            None
        );
        assert_eq!(
            normalized_polyx_policy(None, false).unwrap_or_else(|err| panic!("{err}")),
            None
        );
        assert_eq!(
            normalized_contaminant_policy(None, false).unwrap_or_else(|err| panic!("{err}")),
            None
        );
        assert!(!adapter_policy_uses_bank(None));
        assert!(!polyx_policy_uses_bank(None));
        assert!(!contaminant_policy_uses_bank(None));
    }

    #[test]
    fn explicit_trim_bank_selection_promotes_missing_policy_to_bank() {
        assert_eq!(
            normalized_adapter_policy(None, true).unwrap_or_else(|err| panic!("{err}")).as_deref(),
            Some("bank")
        );
        assert_eq!(
            normalized_polyx_policy(None, true).unwrap_or_else(|err| panic!("{err}")).as_deref(),
            Some("bank")
        );
        assert_eq!(
            normalized_contaminant_policy(None, true)
                .unwrap_or_else(|err| panic!("{err}"))
                .as_deref(),
            Some("bank")
        );
    }

    #[test]
    fn adapter_policy_supports_ancient_strict_without_forcing_explicit_flags() {
        assert_eq!(
            normalized_adapter_policy(Some("ancient_strict"), false)
                .unwrap_or_else(|err| panic!("{err}"))
                .as_deref(),
            Some("ancient_strict")
        );
        assert!(adapter_policy_uses_bank(Some("ancient_strict")));
    }

    #[test]
    fn adapter_bank_requested_detects_any_explicit_adapter_selection() {
        let args = bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimArgs {
            sample_id: "sample".to_string(),
            r1: "reads_R1.fastq.gz".into(),
            r2: None,
            out: "out".into(),
            tools: vec!["fastp".to_string()],
            explain: false,
            replicates: 1,
            jobs: 1,
            ci_bootstrap: None,
            threads: None,
            adapter_bank_preset: Some("illumina-default".to_string()),
            adapter_bank: None,
            adapter_bank_file: None,
            enable_adapters: Vec::new(),
            disable_adapters: Vec::new(),
            polyx_preset: None,
            contaminant_preset: None,
            min_length: None,
            quality_cutoff: None,
            n_policy: None,
            adapter_policy: None,
            polyx_policy: None,
            contaminant_policy: None,
        };

        assert!(adapter_bank_requested(&args));
    }

    #[test]
    fn write_governed_trim_report_preserves_contract_shape() {
        let temp = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let report_path = temp.path().join("trim_report.json");
        let report = TrimReadsReportV1 {
            schema_version: TRIM_READS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.trim_reads".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            tool_id: "fastp".to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 4,
            trimming_backend: "fastp".to_string(),
            backend_mode: "enforced".to_string(),
            input_r1: "reads.fastq.gz".to_string(),
            input_r2: None,
            output_r1: "trimmed.fastq.gz".to_string(),
            output_r2: None,
            min_length: 30,
            quality_cutoff: None,
            adapter_policy: "none".to_string(),
            polyx_policy: Some("none".to_string()),
            n_policy: Some("retain".to_string()),
            contaminant_policy: Some("none".to_string()),
            adapter_bank_id: None,
            adapter_bank_hash: None,
            adapter_preset: None,
            detected_adapter_source: None,
            adapter_overrides: Some(serde_json::json!({
                "enable": ["AGATCGGAAGAGC"],
                "disable": ["polyA"],
            })),
            prepared_adapter_bank: None,
            polyx_bank_id: None,
            polyx_bank_hash: None,
            polyx_preset: None,
            contaminant_bank_id: None,
            contaminant_bank_hash: None,
            contaminant_preset: None,
            reads_in: Some(100),
            reads_out: Some(95),
            bases_in: Some(1000),
            bases_out: Some(900),
            pairs_in: None,
            pairs_out: None,
            mean_q_before: Some(28.0),
            mean_q_after: Some(30.0),
            effective_trim_params: serde_json::json!({
                "min_length": 30,
                "adapter_policy": "none",
            }),
            runtime_s: Some(1.5),
            memory_mb: Some(64.0),
            raw_backend_report: Some("trim.fastp.json".to_string()),
            raw_backend_report_format: Some("fastp_json".to_string()),
        };

        write_governed_trim_report(&report_path, &report)
            .unwrap_or_else(|err| panic!("write report: {err}"));
        let raw = std::fs::read_to_string(&report_path)
            .unwrap_or_else(|err| panic!("read report: {err}"));
        let decoded: TrimReadsReportV1 =
            serde_json::from_str(&raw).unwrap_or_else(|err| panic!("parse report: {err}"));
        assert_eq!(decoded.tool_id, "fastp");
        assert_eq!(decoded.threads, 4);
        assert_eq!(decoded.raw_backend_report_format.as_deref(), Some("fastp_json"));
        assert_eq!(
            decoded.adapter_overrides,
            Some(serde_json::json!({
                "enable": ["AGATCGGAAGAGC"],
                "disable": ["polyA"],
            }))
        );
    }

    #[test]
    fn thread_override_replaces_governed_trim_threads() {
        let tool = dummy_tool("fastp", 2);
        let overridden = apply_thread_override(&tool, Some(8));
        assert_eq!(overridden.resources.threads, 8);
    }

    #[test]
    fn prune_trim_tool_payload_keeps_reports_and_run_artifacts() {
        let temp = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let out_dir = temp.path().join("fastp");
        let run_artifacts = out_dir.join("run_artifacts");
        fs::create_dir_all(&run_artifacts).expect("mkdir");

        let report_path = out_dir.join("trim_report.json");
        let metrics_path = out_dir.join("metrics.json");
        let raw_backend_report = out_dir.join("trim.fastp.json");
        let trimmed_r1 = out_dir.join("reads_R1.fastq.gz");
        let trimmed_r2 = out_dir.join("reads_R2.fastq.gz");
        let stage_report = run_artifacts.join("stage_report.json");

        fs::write(&report_path, "{}").expect("write report");
        fs::write(&metrics_path, "{}").expect("write metrics");
        fs::write(&raw_backend_report, "{}").expect("write backend report");
        fs::write(&trimmed_r1, "trimmed").expect("write r1");
        fs::write(&trimmed_r2, "trimmed").expect("write r2");
        fs::write(&stage_report, "{}").expect("write run artifact");

        let report = TrimReadsReportV1 {
            schema_version: TRIM_READS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.trim_reads".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            tool_id: "fastp".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 4,
            trimming_backend: "fastp".to_string(),
            backend_mode: "enforced".to_string(),
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            output_r1: trimmed_r1.display().to_string(),
            output_r2: Some(trimmed_r2.display().to_string()),
            min_length: 30,
            quality_cutoff: None,
            adapter_policy: "none".to_string(),
            polyx_policy: Some("none".to_string()),
            n_policy: Some("retain".to_string()),
            contaminant_policy: Some("none".to_string()),
            adapter_bank_id: None,
            adapter_bank_hash: None,
            adapter_preset: None,
            detected_adapter_source: None,
            adapter_overrides: None,
            prepared_adapter_bank: None,
            polyx_bank_id: None,
            polyx_bank_hash: None,
            polyx_preset: None,
            contaminant_bank_id: None,
            contaminant_bank_hash: None,
            contaminant_preset: None,
            reads_in: Some(100),
            reads_out: Some(90),
            bases_in: Some(1000),
            bases_out: Some(900),
            pairs_in: Some(50),
            pairs_out: Some(45),
            mean_q_before: Some(28.0),
            mean_q_after: Some(30.0),
            effective_trim_params: serde_json::json!({
                "min_length": 30,
                "adapter_policy": "none",
            }),
            runtime_s: Some(1.0),
            memory_mb: Some(64.0),
            raw_backend_report: Some(raw_backend_report.display().to_string()),
            raw_backend_report_format: Some("fastp_json".to_string()),
        };

        prune_trim_tool_payload(&out_dir, &report_path, &metrics_path, &report)
            .unwrap_or_else(|err| panic!("prune payload: {err}"));

        assert!(report_path.is_file());
        assert!(metrics_path.is_file());
        assert!(raw_backend_report.is_file());
        assert!(stage_report.is_file());
        assert!(!trimmed_r1.exists());
        assert!(!trimmed_r2.exists());
    }
}
