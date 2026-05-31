use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::core_other::{fetch_fastq_validate_v1, insert_fastq_validate_v1};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkContext, BenchmarkRecord, FastqValidateMetrics,
};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::measure::SeqkitMetrics;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::observer::parse_validation_report;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::select_validate_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::validate_reads::{
    default_plan_options_for_layout, pair_sync_policy_from_literal,
    plan_with_options as plan_validate_reads, validation_mode_from_literal,
    ValidateReadsPlanOptions,
};
use bijux_dna_planner_fastq::stage_api::observer::{input_fastq_stats, parse_seqkit_stats};
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::backend::docker::executor::resolve_image_for_run;
use bijux_dna_runner::step_runner::{execute_observer_command, StageResultV1};
use serde::{Deserialize, Serialize};

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::internal::fastq::stages::trim_bench_common::{
    benchmark_image_identity, require_existing_benchmark_output,
};
use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_VALIDATE_READS,
};
use bijux_dna_stage_contract::StagePlanV1;

const LOCAL_VALIDATE_READS_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.validate.local_smoke.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LocalValidateReadsSmokeLayout {
    SingleEnd,
    PairedEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LocalValidateReadsSmokeStatus {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalValidateReadsSmokeCaseReport {
    sample_id: String,
    layout: LocalValidateReadsSmokeLayout,
    input_r1: String,
    input_r2: Option<String>,
    input_read_count_total: u64,
    input_read_count_r1: u64,
    input_read_count_r2: Option<u64>,
    input_pair_count: Option<u64>,
    validation_status: LocalValidateReadsSmokeStatus,
    missing_output_marker_present: bool,
    validation_report: String,
    validated_reads_manifest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalValidateReadsSmokeReport {
    schema_version: String,
    stage_id: String,
    case_count: u64,
    all_cases_passed: bool,
    missing_output_marker_present: bool,
    cases: Vec<LocalValidateReadsSmokeCaseReport>,
}

/// Materialize the governed local-smoke `fastq.validate_reads` artifacts and summary report.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke report artifacts cannot be written.
pub fn write_local_validate_reads_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_fastq::stage_api::local_validate_reads_smoke_plans(&repo_root)?;
    let output_root = repo_root.join("target/local-smoke/fastq.validate_reads");
    bijux_dna_infra::ensure_dir(&output_root)?;

    let case_reports = cases
        .iter()
        .map(|case| materialize_local_validate_reads_smoke_case(&repo_root, case))
        .collect::<Result<Vec<_>>>()?;

    let summary = LocalValidateReadsSmokeReport {
        schema_version: LOCAL_VALIDATE_READS_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: STAGE_VALIDATE_READS.as_str().to_string(),
        case_count: case_reports.len() as u64,
        all_cases_passed: case_reports
            .iter()
            .all(|case| matches!(case.validation_status, LocalValidateReadsSmokeStatus::Pass)),
        missing_output_marker_present: case_reports
            .iter()
            .any(|case| case.missing_output_marker_present),
        cases: case_reports,
    };

    let report_path = output_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    Ok(report_path)
}

fn materialize_local_validate_reads_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_fastq::LocalValidateReadsSmokeCasePlan,
) -> Result<LocalValidateReadsSmokeCaseReport> {
    let case_out_dir = resolve_plan_dir(repo_root, &case.plan.out_dir);
    let artifact_paths =
        bijux_dna_domain_fastq::validation_artifact_paths(&case_out_dir, case.r2.is_some());
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let r1 = repo_root.join(&case.r1);
    let r2 = case.r2.as_ref().map(|path| repo_root.join(path));
    let (report, manifest) = bijux_dna_domain_fastq::stages::validate_reads(
        &r1,
        r2.as_deref(),
        case.validation_mode.clone(),
        case.pair_sync_policy.clone(),
        &artifact_paths.validation_log_r1,
        artifact_paths.validation_log_r2.as_deref(),
        &artifact_paths.report_json,
    )?;

    bijux_dna_infra::atomic_write_json(&artifact_paths.report_json, &report)?;
    bijux_dna_infra::atomic_write_json(&artifact_paths.validated_reads_manifest, &manifest)?;

    let missing_output_marker_present = scan_missing_output_markers(&case_out_dir);
    let input_read_count_r2 = report.validated_reads_r2;
    let input_pair_count = report.validated_pairs;
    Ok(LocalValidateReadsSmokeCaseReport {
        sample_id: case.sample_id.clone(),
        layout: if case.r2.is_some() {
            LocalValidateReadsSmokeLayout::PairedEnd
        } else {
            LocalValidateReadsSmokeLayout::SingleEnd
        },
        input_r1: case.r1.display().to_string(),
        input_r2: case.r2.as_ref().map(|path| path.display().to_string()),
        input_read_count_total: report.validated_reads_r1 + input_read_count_r2.unwrap_or(0),
        input_read_count_r1: report.validated_reads_r1,
        input_read_count_r2,
        input_pair_count,
        validation_status: if report.strict_pass {
            LocalValidateReadsSmokeStatus::Pass
        } else {
            LocalValidateReadsSmokeStatus::Fail
        },
        missing_output_marker_present,
        validation_report: path_relative_to_repo(repo_root, &artifact_paths.report_json),
        validated_reads_manifest: path_relative_to_repo(
            repo_root,
            &artifact_paths.validated_reads_manifest,
        ),
    })
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

fn scan_missing_output_markers(root: &Path) -> bool {
    walkdir::WalkDir::new(root)
        .into_iter()
        .flatten()
        .filter(|entry| entry.file_type().is_file())
        .any(|entry| {
            let name = entry.file_name().to_string_lossy();
            name.contains("missing_output") || name.ends_with(".invalid")
        })
}

/// # Errors
/// Returns an error if planning, execution, metric derivation, or persistence fails.
pub fn bench_fastq_validate_reads<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqValidateArgs,
) -> Result<BenchOutcome<bijux_dna_analyze::FastqValidateMetrics>> {
    let tools = select_validate_tools(&args.tools)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_VALIDATE_READS.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_VALIDATE_READS.as_str(), &header);

    let setup = prepare_validate_benchmark_setup(catalog, platform, runner_override, args, &tools)?;

    if args.explain {
        write_validate_benchmark_explain(&setup)?;
    }
    ensure_validate_benchmark_qa(catalog, platform, &setup.tools)?;

    let sqlite_path = setup.bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = setup.bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut records = Vec::<BenchmarkRecord<FastqValidateMetrics>>::new();
    let mut failures = Vec::<RawFailure>::new();

    for tool in setup.tools.clone() {
        let tool_plan = prepare_validate_tool_plan(catalog, platform, args, &setup, jobs, tool)?;
        if let Ok(Some(record)) = fetch_fastq_validate_v1(
            &conn,
            &tool_plan.tool,
            &tool_plan.tool_spec.tool_version,
            &tool_plan.image_digest,
            &setup.bench_inputs.runner.to_string(),
            &platform.name,
            &setup.bench_inputs.input_hash,
            &tool_plan.params_hash,
        ) {
            records.push(record);
            continue;
        }

        let tool_execution =
            execute_validate_tool(platform, &setup.bench_inputs, &tool_plan, jobs)?;

        append_jsonl(&bench_path, &tool_execution.record).context("write bench.jsonl")?;
        insert_fastq_validate_v1(&conn, &tool_execution.record).context("insert bench sqlite")?;
        if let Some(failure) = validate_tool_failure(args, &tool_plan, tool_execution.exit_code) {
            failures.push(failure);
        }
        records.push(tool_execution.record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: setup.bench_inputs.bench_dir,
        explain: args.explain,
    })
}

struct ValidateBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    excluded_tools: Vec<String>,
    bench_inputs: ValidateBenchInputs,
}

struct ValidateToolPlan {
    tool: String,
    tool_spec: ToolExecutionSpecV1,
    plan_options: ValidateReadsPlanOptions,
    plan: StagePlanV1,
    bench_params: serde_json::Value,
    params_hash: String,
    image_digest: String,
}

struct ValidateToolExecution {
    record: BenchmarkRecord<FastqValidateMetrics>,
    exit_code: i32,
}

fn prepare_validate_benchmark_setup<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqValidateArgs,
    tools: &[String],
) -> Result<ValidateBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_VALIDATE_READS.as_str(), tools, &registry, false)?;
    let bench_inputs = prepare_validate_bench(catalog, platform, runner_override, args)?;
    let excluded_tools = excluded_validate_tools(&registry, &tools);
    Ok(ValidateBenchmarkSetup { registry, tools, excluded_tools, bench_inputs })
}

fn prepare_validate_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqValidateArgs,
    setup: &ValidateBenchmarkSetup,
    jobs: usize,
    tool: String,
) -> Result<ValidateToolPlan> {
    let out_dir = setup.bench_inputs.tools_root.join(&tool);
    bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
    let tool_spec = build_tool_execution_spec(
        STAGE_VALIDATE_READS.as_str(),
        &tool,
        &setup.registry,
        catalog,
        platform,
    )?;
    let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
    let plan_options = validate_plan_options(args)?;
    let plan = plan_validate_reads(
        &tool_spec,
        &setup.bench_inputs.r1,
        args.r2.as_deref(),
        &out_dir,
        &plan_options,
    )?;
    let bench_params = benchmark_query_context()?.embed_in_parameters(&plan.params);
    let params_hash = stable_params_hash(&bench_params);
    let image_digest =
        tool_spec.image.digest.clone().unwrap_or_else(|| tool_spec.image.image.clone());
    Ok(ValidateToolPlan {
        tool,
        tool_spec,
        plan_options,
        plan,
        bench_params,
        params_hash,
        image_digest,
    })
}

fn execute_validate_tool(
    platform: &PlatformSpec,
    bench_inputs: &ValidateBenchInputs,
    tool_plan: &ValidateToolPlan,
    jobs: usize,
) -> Result<ValidateToolExecution> {
    let execution = execute_plans_with_jobs(
        vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&tool_plan.plan)],
        bench_inputs.runner,
        jobs,
    )?
    .into_iter()
    .next()
    .ok_or_else(|| anyhow!("missing execution result for {}", tool_plan.tool))?;

    let record = build_validate_record(
        platform,
        bench_inputs,
        &tool_plan.tool,
        &tool_plan.tool_spec,
        &tool_plan.bench_params,
        &tool_plan.plan,
        &execution,
    )?;
    Ok(ValidateToolExecution { record, exit_code: execution.exit_code })
}

fn validate_tool_failure(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqValidateArgs,
    tool_plan: &ValidateToolPlan,
    exit_code: i32,
) -> Option<RawFailure> {
    if exit_code == 0 || !validation_failures_are_fatal(args, &tool_plan.plan_options) {
        return None;
    }
    Some(RawFailure {
        stage: STAGE_VALIDATE_READS.as_str().to_string(),
        tool: tool_plan.tool.clone(),
        reason: format!(
            "validator `{}` failed strict validation with status {exit_code}",
            tool_plan.tool
        ),
        category: ErrorCategory::ToolError,
    })
}

fn excluded_validate_tools(registry: &ToolRegistry, selected_tools: &[String]) -> Vec<String> {
    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_VALIDATE_READS.as_str());
    registry
        .tools_for_stage(&stage_id)
        .iter()
        .map(|tool| tool.tool_id.to_string())
        .filter(|tool| !selected_tools.contains(tool))
        .collect()
}

fn write_validate_benchmark_explain(setup: &ValidateBenchmarkSetup) -> Result<()> {
    write_explain_md(
        &setup.bench_inputs.bench_dir,
        STAGE_VALIDATE_READS.as_str(),
        &setup.tools,
        &setup.excluded_tools,
        None,
    )?;
    write_explain_plan_json(
        &setup.bench_inputs.bench_dir,
        STAGE_VALIDATE_READS.as_str(),
        &setup.tools,
        &setup.registry,
        None,
    )
}

fn ensure_validate_benchmark_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(STAGE_VALIDATE_READS.as_str(), tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_VALIDATE_READS.as_str(), tools, platform, catalog)
}

#[derive(Debug, Clone)]
struct ValidateBenchInputs {
    runner: RuntimeKind,
    r1: PathBuf,
    input_hash: String,
    input_stats: SeqkitMetrics,
    input_stats_r2: Option<SeqkitMetrics>,
    bench_dir: PathBuf,
    tools_root: PathBuf,
}

fn prepare_validate_bench<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqValidateArgs,
) -> Result<ValidateBenchInputs> {
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir_name = bench_dir_name(&STAGE_VALIDATE_READS)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_VALIDATE_READS.as_str()))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_dna_infra::ensure_dir(&tools_root).context("create tools output dir")?;

    let r1 = args.r1.canonicalize().context("resolve r1 path")?;
    let r1_dir = r1.parent().ok_or_else(|| anyhow!("r1 has no parent"))?.to_path_buf();

    let seqkit_tool = catalog
        .get(bijux_dna_planner_fastq::stage_api::TOOL_SEQKIT)
        .ok_or_else(|| anyhow!("seqkit missing from images catalog"))?;
    let seqkit_image = resolve_image_for_run(seqkit_tool, platform)?;
    let stats_spec = input_fastq_stats(&r1_dir, &r1)?;
    let stats_output = execute_observer_command(
        &seqkit_image.full_name,
        stats_spec.mount_dir.as_path(),
        &stats_spec.args,
        runner,
    )?;
    if stats_output.exit_code != 0 {
        return Err(anyhow!("seqkit validation observer failed: {}", stats_output.stderr));
    }

    let input_hash = if let Some(r2) = args.r2.as_deref() {
        format!(
            "{}+{}",
            hash_file_sha256(&r1).context("hash validation input r1")?,
            hash_file_sha256(r2).context("hash validation input r2")?
        )
    } else {
        hash_file_sha256(&r1).context("hash validation input")?
    };
    let input_stats_r2 = if let Some(r2) = args.r2.as_deref() {
        let r2 = r2.canonicalize().context("resolve r2 path")?;
        let r2_dir = r2.parent().ok_or_else(|| anyhow!("r2 has no parent"))?.to_path_buf();
        let stats_spec = input_fastq_stats(&r2_dir, &r2)?;
        let stats_output = execute_observer_command(
            &seqkit_image.full_name,
            stats_spec.mount_dir.as_path(),
            &stats_spec.args,
            runner,
        )?;
        if stats_output.exit_code != 0 {
            return Err(anyhow!(
                "seqkit validation observer failed for r2: {}",
                stats_output.stderr
            ));
        }
        Some(parse_seqkit_stats(&stats_output.stdout)?)
    } else {
        None
    };

    Ok(ValidateBenchInputs {
        runner,
        r1,
        input_hash,
        input_stats: parse_seqkit_stats(&stats_output.stdout)?,
        input_stats_r2,
        bench_dir,
        tools_root,
    })
}

fn build_validate_record(
    platform: &PlatformSpec,
    bench_inputs: &ValidateBenchInputs,
    tool: &str,
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    params: &serde_json::Value,
    plan: &StagePlanV1,
    execution: &StageResultV1,
) -> Result<BenchmarkRecord<FastqValidateMetrics>> {
    let out_dir = &plan.out_dir;
    let report_path = required_plan_output_path(plan, "validation_report")?;
    let report_path = require_existing_benchmark_output(&report_path, "validation_report")?;
    let metrics = derive_validate_metrics(
        &bench_inputs.input_stats,
        bench_inputs.input_stats_r2.as_ref(),
        report_path,
    );
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    let manifest_path = required_plan_output_path(plan, "validated_reads_manifest")?;
    let _manifest_path =
        require_existing_benchmark_output(&manifest_path, "validated_reads_manifest")?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write validation metrics")?;

    let context = BenchmarkContext {
        tool: tool.to_string(),
        tool_version: tool_spec.tool_version.clone(),
        image_digest: benchmark_image_identity(tool_spec),
        runner: bench_inputs.runner.to_string(),
        platform: platform.name.clone(),
        input_hash: bench_inputs.input_hash.clone(),
        parameters: params.clone().into(),
    };
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

fn required_plan_output_path(plan: &StagePlanV1, output_id: &str) -> Result<PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| {
            anyhow!(
                "validate_reads plan is missing governed output `{output_id}` for tool {}",
                plan.tool_id.as_str()
            )
        })
}

fn derive_validate_metrics(
    input_stats: &SeqkitMetrics,
    input_stats_r2: Option<&SeqkitMetrics>,
    report_path: &std::path::Path,
) -> FastqValidateMetrics {
    let reads_in = input_stats.reads + input_stats_r2.map_or(0, |stats| stats.reads);
    let bases_in = input_stats.bases + input_stats_r2.map_or(0, |stats| stats.bases);
    let parsed_report = std::fs::read_to_string(report_path)
        .ok()
        .and_then(|raw| parse_validation_report(&raw).ok());
    let reads_total = parsed_report.as_ref().map_or(reads_in, validate_report_reads_total);
    let reads_invalid =
        parsed_report.as_ref().map_or(0, validate_report_reads_invalid).min(reads_total);
    let reads_valid = reads_total.saturating_sub(reads_invalid);
    FastqValidateMetrics {
        reads_in,
        reads_out: reads_in,
        bases_in,
        bases_out: bases_in,
        pairs_in: input_stats_r2.map(|stats| input_stats.reads.min(stats.reads)),
        pairs_out: input_stats_r2.map(|stats| input_stats.reads.min(stats.reads)),
        reads_total,
        reads_valid,
        reads_invalid,
        mean_q: input_stats.mean_q,
        validated_inputs: parsed_report.as_ref().map(|report| report.validated_inputs),
        validated_pairs: parsed_report.as_ref().and_then(|report| report.validated_pairs),
        pair_sync_checked: parsed_report.as_ref().map(|report| report.pair_sync_checked),
        pair_sync_pass: parsed_report.as_ref().and_then(|report| report.pair_sync_pass),
        pair_count_match: parsed_report.as_ref().and_then(|report| report.pair_count_match),
        strict_pass: parsed_report.as_ref().map(|report| report.strict_pass),
        failure_class: parsed_report
            .as_ref()
            .and_then(|report| serde_json::to_value(&report.failure_class).ok())
            .and_then(|value| value.as_str().map(ToOwned::to_owned)),
    }
}

fn validate_report_reads_total(report: &bijux_dna_domain_fastq::ValidationReportV1) -> u64 {
    report.validated_reads_r1 + report.validated_reads_r2.unwrap_or(0)
}

fn validate_report_reads_invalid(report: &bijux_dna_domain_fastq::ValidationReportV1) -> u64 {
    match report.failure_class {
        bijux_dna_domain_fastq::ValidateFailureClass::None
        | bijux_dna_domain_fastq::ValidateFailureClass::HeaderSyncMismatch => 0,
        bijux_dna_domain_fastq::ValidateFailureClass::UnsupportedCompression
        | bijux_dna_domain_fastq::ValidateFailureClass::EmptyInput
        | bijux_dna_domain_fastq::ValidateFailureClass::MalformedRecord
        | bijux_dna_domain_fastq::ValidateFailureClass::InvalidQualityEncoding => {
            validate_report_reads_total(report)
        }
        bijux_dna_domain_fastq::ValidateFailureClass::PairCountMismatch => {
            report.validated_reads_r1.abs_diff(report.validated_reads_r2.unwrap_or(0))
        }
        bijux_dna_domain_fastq::ValidateFailureClass::ValidatorError => {
            let mut invalid = 0;
            if report.status_r1 != 0 {
                invalid += report.validated_reads_r1;
            }
            if report.status_r2 != 0 {
                invalid += report.validated_reads_r2.unwrap_or(0);
            }
            invalid
        }
    }
}

fn benchmark_query_context() -> Result<bijux_dna_domain_fastq::BenchQueryContext> {
    bijux_dna_domain_fastq::governed_stage_bench_query_context(STAGE_VALIDATE_READS.as_str())
}

fn validate_plan_options(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqValidateArgs,
) -> Result<ValidateReadsPlanOptions> {
    let default_options = default_plan_options_for_layout(args.r2.as_deref());
    Ok(ValidateReadsPlanOptions {
        threads: args.threads,
        validation_mode: args
            .validation_mode
            .as_deref()
            .map(validation_mode_from_literal)
            .transpose()?
            .unwrap_or(default_options.validation_mode),
        pair_sync_policy: args
            .pair_sync_policy
            .as_deref()
            .map(pair_sync_policy_from_literal)
            .transpose()?
            .unwrap_or(default_options.pair_sync_policy),
    })
}

fn validation_failures_are_fatal(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqValidateArgs,
    options: &ValidateReadsPlanOptions,
) -> bool {
    if args.validation_mode.is_some() {
        return options.validation_mode
            == bijux_dna_domain_fastq::params::validate::ValidationMode::Strict;
    }
    args.strict
}

#[cfg(test)]
mod tests {
    use super::{
        derive_validate_metrics, required_plan_output_path, validate_plan_options,
        validation_failures_are_fatal,
    };
    use bijux_dna_core::contract::{ArtifactRole, StageIO, ToolConstraints};
    use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, ToolId};
    use bijux_dna_core::prelude::measure::SeqkitMetrics;
    use bijux_dna_core::prelude::{ArtifactRef, CommandSpecV1, ContainerImageRefV1};
    use bijux_dna_stage_contract::{PlanDecisionReason, StagePlanV1};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    #[test]
    fn validate_record_paths_follow_plan_outputs() {
        let plan = StagePlanV1 {
            stage_id: StageId::from_static("fastq.validate_reads"),
            stage_instance_id: None,
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("fastqvalidator"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
            command: CommandSpecV1 { template: vec!["fastqvalidator".to_string()] },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: Vec::new(),
                outputs: vec![
                    ArtifactRef::required(
                        ArtifactId::from_static("validation_report"),
                        PathBuf::from("custom/validation.json"),
                        ArtifactRole::ReportJson,
                    ),
                    ArtifactRef::required(
                        ArtifactId::from_static("validated_reads_manifest"),
                        PathBuf::from("custom/validated_reads_manifest.json"),
                        ArtifactRole::StageReport,
                    ),
                ],
            },
            out_dir: PathBuf::from("custom"),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            aux_images: BTreeMap::new(),
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
            reason: PlanDecisionReason::default(),
        };

        assert_eq!(
            required_plan_output_path(&plan, "validation_report")
                .unwrap_or_else(|err| panic!("report path: {err}")),
            PathBuf::from("custom/validation.json")
        );
        assert_eq!(
            required_plan_output_path(&plan, "validated_reads_manifest")
                .unwrap_or_else(|err| panic!("manifest path: {err}")),
            PathBuf::from("custom/validated_reads_manifest.json")
        );
    }

    #[test]
    fn missing_validation_manifest_is_rejected_before_metrics() {
        let plan = StagePlanV1 {
            stage_id: StageId::from_static("fastq.validate_reads"),
            stage_instance_id: None,
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("fastqvalidator"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
            command: CommandSpecV1 { template: vec!["fastqvalidator".to_string()] },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: Vec::new(),
                outputs: vec![ArtifactRef::required(
                    ArtifactId::from_static("validation_report"),
                    PathBuf::from("custom/validation.json"),
                    ArtifactRole::ReportJson,
                )],
            },
            out_dir: PathBuf::from("custom"),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            aux_images: BTreeMap::new(),
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
            reason: PlanDecisionReason::default(),
        };

        let error = match required_plan_output_path(&plan, "validated_reads_manifest") {
            Ok(path) => panic!("missing manifest must be rejected: {}", path.display()),
            Err(err) => err,
        };
        assert!(error.to_string().contains("missing governed output `validated_reads_manifest`"));
    }

    #[test]
    fn validate_plan_options_parse_governed_policy_overrides() {
        let args = bijux_dna_planner_fastq::stage_api::args::BenchFastqValidateArgs {
            sample_id: "sample".to_string(),
            r1: PathBuf::from("reads_R1.fastq.gz"),
            r2: Some(PathBuf::from("reads_R2.fastq.gz")),
            out: PathBuf::from("out"),
            tools: vec!["fastqvalidator".to_string()],
            explain: false,
            strict: false,
            replicates: 1,
            jobs: 1,
            ci_bootstrap: None,
            threads: Some(9),
            validation_mode: Some("report_only".to_string()),
            pair_sync_policy: Some("skip_header_sync".to_string()),
        };

        let options = validate_plan_options(&args).unwrap_or_else(|err| panic!("options: {err}"));
        assert_eq!(options.threads, Some(9));
        assert_eq!(
            options.validation_mode,
            bijux_dna_domain_fastq::params::validate::ValidationMode::ReportOnly
        );
        assert_eq!(
            options.pair_sync_policy,
            bijux_dna_domain_fastq::params::validate::PairSyncPolicy::SkipHeaderSync
        );
    }

    #[test]
    fn validate_plan_options_default_single_end_policy_is_not_applicable() {
        let args = bijux_dna_planner_fastq::stage_api::args::BenchFastqValidateArgs {
            sample_id: "sample".to_string(),
            r1: PathBuf::from("reads.fastq.gz"),
            r2: None,
            out: PathBuf::from("out"),
            tools: vec!["fastqvalidator".to_string()],
            explain: false,
            strict: false,
            replicates: 1,
            jobs: 1,
            ci_bootstrap: None,
            threads: None,
            validation_mode: None,
            pair_sync_policy: None,
        };

        let options = validate_plan_options(&args).unwrap_or_else(|err| panic!("options: {err}"));
        assert_eq!(
            options.pair_sync_policy,
            bijux_dna_domain_fastq::params::validate::PairSyncPolicy::NotApplicable
        );
    }

    #[test]
    fn explicit_validation_mode_overrides_legacy_strict_flag() {
        let args = bijux_dna_planner_fastq::stage_api::args::BenchFastqValidateArgs {
            sample_id: "sample".to_string(),
            r1: PathBuf::from("reads_R1.fastq.gz"),
            r2: Some(PathBuf::from("reads_R2.fastq.gz")),
            out: PathBuf::from("out"),
            tools: vec!["fastqvalidator".to_string()],
            explain: false,
            strict: true,
            replicates: 1,
            jobs: 1,
            ci_bootstrap: None,
            threads: None,
            validation_mode: Some("report_only".to_string()),
            pair_sync_policy: None,
        };

        let options = validate_plan_options(&args).unwrap_or_else(|err| panic!("options: {err}"));
        assert!(!validation_failures_are_fatal(&args, &options));
    }

    #[test]
    fn derive_validate_metrics_prefers_governed_report_counts() {
        let temp = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let report_path = temp.path().join("validation.json");
        bijux_dna_infra::write_bytes(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.validate.report.v1",
                "stage": "fastq.validate_reads",
                "stage_id": "fastq.validate_reads",
                "tool_id": "fastqvalidator",
                "validation_mode": "strict",
                "pair_sync_policy": "require_header_sync",
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "validation_log_r1": "validation_r1.log",
                "validation_log_r2": "validation_r2.log",
                "validated_inputs": 2,
                "validated_reads_r1": 101_u64,
                "validated_reads_r2": 100_u64,
                "validated_pairs": 100_u64,
                "status_r1": 0,
                "status_r2": 0,
                "pair_sync_checked": true,
                "pair_sync_pass": false,
                "pair_count_match": false,
                "failure_class": "pair_count_mismatch",
                "strict_pass": false,
                "exit_code": 96
            })
            .to_string(),
        )
        .unwrap_or_else(|err| panic!("write report: {err}"));

        let metrics = derive_validate_metrics(
            &SeqkitMetrics { reads: 101, bases: 1000, mean_q: 31.0, gc_percent: 50.0 },
            Some(&SeqkitMetrics { reads: 100, bases: 990, mean_q: 30.5, gc_percent: 49.0 }),
            &report_path,
        );

        assert_eq!(metrics.reads_total, 201);
        assert_eq!(metrics.reads_invalid, 1);
        assert_eq!(metrics.reads_valid, 200);
        assert_eq!(metrics.validated_inputs, Some(2));
        assert_eq!(metrics.validated_pairs, Some(100));
        assert_eq!(metrics.pair_sync_checked, Some(true));
        assert_eq!(metrics.pair_sync_pass, Some(false));
        assert_eq!(metrics.pair_count_match, Some(false));
        assert_eq!(metrics.strict_pass, Some(false));
        assert_eq!(metrics.failure_class.as_deref(), Some("pair_count_mismatch"));
    }
}
