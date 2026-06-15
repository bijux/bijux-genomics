use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_overrepresented_v1, insert_fastq_overrepresented_v1,
};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkRecord, FastqOverrepresentedMetrics, MetricSet,
    StageMetricSchema,
};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::{
    FastqOverrepresentedProfileParams, OverrepresentedSequenceRowV1, PairedMode,
    ProfileOverrepresentedReportV1, PROFILE_OVERREPRESENTED_REPORT_SCHEMA_VERSION,
};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::profile_overrepresented_sequences::plan_with_options;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use bijux_dna_stage_contract::StagePlanV1;
use serde::Serialize;

use crate::internal::fastq::stages::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context,
};
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};

const STAGE_ID: &str = "fastq.profile_overrepresented_sequences";
const LOCAL_PROFILE_OVERREPRESENTED_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.profile_overrepresented_sequences.local_smoke.report.v1";

#[derive(Debug, Clone, Serialize)]
struct LocalProfileOverrepresentedSequencesSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    planned_tool_id: String,
    report_tool_id: String,
    top_sequence: String,
    top_count: u64,
    top_fraction: f64,
    flagged_sequences: u64,
    overrepresented_tsv: String,
    case_report_json: String,
    raw_backend_report: String,
}

/// Benchmark FASTQ overrepresented-sequence profiling tools.
///
/// # Errors
/// Returns an error if planning, execution, report parsing, or persistence fails.
pub fn bench_fastq_profile_overrepresented<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileOverrepresentedArgs,
) -> Result<BenchOutcome<FastqOverrepresentedMetrics>> {
    let selected_tools = select_overrepresented_benchmark_tools(args)?;
    let setup =
        prepare_overrepresented_benchmark_setup(platform, runner_override, args, &selected_tools)?;

    if args.explain {
        write_overrepresented_benchmark_explain(&setup)?;
    }

    ensure_overrepresented_benchmark_qa(catalog, platform, &setup.tools)?;

    let store = OverrepresentedBenchmarkStore::from_setup(&setup);
    let conn = bijux_dna_analyze::open_sqlite(&store.sqlite_path).context("open bench sqlite")?;
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::<BenchmarkRecord<FastqOverrepresentedMetrics>>::new();

    for tool in &setup.tools {
        let tool_plan = prepare_overrepresented_tool_plan(catalog, platform, args, &setup, tool)?;
        let cache_identity = OverrepresentedCacheIdentity::from_plan(platform, &setup, &tool_plan);
        if let Ok(Some(record)) = fetch_fastq_overrepresented_v1(
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
        let execution = execute_overrepresented_tool(&tool_plan, setup.runner, jobs)?;
        if let Some(failure) = overrepresented_tool_failure(&tool_plan, &execution) {
            failures.push(failure);
            continue;
        }
        let observation = observe_overrepresented_tool(args, &tool_plan.plan)?;
        let metric_set = build_overrepresented_metric_set(&observation)?;
        let report = build_overrepresented_report(OverrepresentedReportInputs {
            tool,
            args,
            artifacts: &observation.artifacts,
            effective_params: &observation.effective_params,
            payload: observation.payload.clone(),
            execution_metrics: overrepresented_execution_metrics(&execution),
        });
        write_overrepresented_artifacts(&tool_plan, &observation, &report, &metric_set)?;
        let record = build_overrepresented_record(
            &OverrepresentedRecordInputs {
                platform,
                setup: &setup,
                tool,
                tool_plan: &tool_plan,
                execution: &execution,
            },
            metric_set,
        )?;
        persist_overrepresented_record(&store, &record, |record| {
            insert_fastq_overrepresented_v1(&conn, record).context("insert bench sqlite")
        })?;
        records.push(record);
    }

    Ok(BenchOutcome { records, failures, bench_dir: setup.bench_dir, explain: args.explain })
}

/// Materialize the governed local-smoke `fastq.profile_overrepresented_sequences` summary TSV.
///
/// The written top-level artifact lives at
/// `runs/bench/local-smoke/fastq.profile_overrepresented_sequences/overrepresented.tsv`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_profile_overrepresented_sequences_smoke_summary() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases =
        bijux_dna_planner_fastq::stage_api::local_profile_overrepresented_sequences_smoke_plans(
            &repo_root,
        )?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "governed fastq.profile_overrepresented_sequences local smoke must resolve exactly one case"
        ));
    };

    let output_root =
        repo_root.join("runs/bench/local-smoke/fastq.profile_overrepresented_sequences");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let summary = materialize_local_profile_overrepresented_sequences_smoke_case(
        &repo_root,
        case,
        &output_root,
    )?;
    let report_path = output_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    Ok(output_root.join("overrepresented.tsv"))
}

fn materialize_local_profile_overrepresented_sequences_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_fastq::LocalProfileOverrepresentedSequencesSmokeCasePlan,
    output_root: &Path,
) -> Result<LocalProfileOverrepresentedSequencesSmokeReport> {
    let effective_params = serde_json::from_value::<FastqOverrepresentedProfileParams>(
        case.plan.effective_params.clone(),
    )
    .map_err(|error| {
        anyhow!("decode profile-overrepresented local-smoke effective params: {error}")
    })?;

    let input_r1 = repo_root.join(&case.r1);
    let input_r2 = case.r2.as_ref().map(|path| repo_root.join(path));
    let case_output_tsv = resolve_smoke_path(
        repo_root,
        required_output_path(&case.plan, "overrepresented_sequences_tsv")?,
    );
    let case_output_json = resolve_smoke_path(
        repo_root,
        required_output_path(&case.plan, "overrepresented_sequences_json")?,
    );
    let case_report_json =
        resolve_smoke_path(repo_root, required_output_path(&case.plan, "report_json")?);
    let raw_backend_report = resolve_smoke_path(
        repo_root,
        &case.plan.out_dir.join("profile_overrepresented_sequences.local.log"),
    );

    let mut report = bijux_dna_domain_fastq::stages::contract::profile_overrepresented_sequences(
        &input_r1,
        input_r2.as_deref(),
        &effective_params,
        &case_output_tsv,
        &case_output_json,
        &case_report_json,
        Some(&raw_backend_report),
    )?;

    report.input_r1 = case.r1.display().to_string();
    report.input_r2 = case.r2.as_ref().map(|path| path.display().to_string());
    report.overrepresented_sequences_tsv = path_relative_to_repo(repo_root, &case_output_tsv);
    report.overrepresented_sequences_json = path_relative_to_repo(repo_root, &case_output_json);
    report.report_json = path_relative_to_repo(repo_root, &case_report_json);
    report.raw_backend_report = Some(path_relative_to_repo(repo_root, &raw_backend_report));
    bijux_dna_infra::atomic_write_json(&case_report_json, &report)?;

    let top_level_tsv = output_root.join("overrepresented.tsv");
    copy_smoke_artifact(&case_output_tsv, &top_level_tsv)?;

    let top_row = report.rows.first().ok_or_else(|| {
        anyhow!("profile_overrepresented_sequences local smoke expected at least one ranked row")
    })?;

    Ok(LocalProfileOverrepresentedSequencesSmokeReport {
        schema_version: LOCAL_PROFILE_OVERREPRESENTED_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: STAGE_ID.to_string(),
        sample_id: case.sample_id.clone(),
        planned_tool_id: case.plan.tool_id.as_str().to_string(),
        report_tool_id: report.tool_id,
        top_sequence: top_row.sequence.clone(),
        top_count: top_row.count,
        top_fraction: top_row.fraction,
        flagged_sequences: report.flagged_sequences,
        overrepresented_tsv: path_relative_to_repo(repo_root, &top_level_tsv),
        case_report_json: path_relative_to_repo(repo_root, &case_report_json),
        raw_backend_report: path_relative_to_repo(repo_root, &raw_backend_report),
    })
}

struct OverrepresentedBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    runner: RuntimeKind,
    bench_dir: PathBuf,
    tools_root: PathBuf,
    input_hash: String,
}

struct OverrepresentedBenchmarkStore {
    sqlite_path: PathBuf,
    jsonl_path: PathBuf,
}

impl OverrepresentedBenchmarkStore {
    fn from_setup(setup: &OverrepresentedBenchmarkSetup) -> Self {
        Self {
            sqlite_path: setup.bench_dir.join("bench.sqlite"),
            jsonl_path: setup.bench_dir.join("bench.jsonl"),
        }
    }
}

struct OverrepresentedToolPlan {
    tool: String,
    out_dir: PathBuf,
    tool_spec: ToolExecutionSpecV1,
    plan: StagePlanV1,
    params_hash: String,
    image_digest: String,
}

struct OverrepresentedToolExecution {
    result: StageResultV1,
}

struct OverrepresentedCacheIdentity {
    tool: String,
    tool_version: String,
    image_digest: String,
    runner: String,
    platform: String,
    input_hash: String,
    params_hash: String,
}

impl OverrepresentedCacheIdentity {
    fn from_plan(
        platform: &PlatformSpec,
        setup: &OverrepresentedBenchmarkSetup,
        tool_plan: &OverrepresentedToolPlan,
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

struct OverrepresentedArtifacts {
    output_tsv: PathBuf,
    output_json: PathBuf,
    report_json: PathBuf,
}

struct OverrepresentedObservation {
    artifacts: OverrepresentedArtifacts,
    effective_params: FastqOverrepresentedProfileParams,
    payload: OverrepresentedPayload,
}

struct OverrepresentedReportInputs<'a> {
    tool: &'a str,
    args: &'a bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileOverrepresentedArgs,
    artifacts: &'a OverrepresentedArtifacts,
    effective_params: &'a FastqOverrepresentedProfileParams,
    payload: OverrepresentedPayload,
    execution_metrics: ExecutionMetrics,
}

struct OverrepresentedRecordInputs<'a> {
    platform: &'a PlatformSpec,
    setup: &'a OverrepresentedBenchmarkSetup,
    tool: &'a str,
    tool_plan: &'a OverrepresentedToolPlan,
    execution: &'a OverrepresentedToolExecution,
}

fn select_overrepresented_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileOverrepresentedArgs,
) -> Result<Vec<String>> {
    let tools = bijux_dna_planner_fastq::select_profile_overrepresented_tools(&args.tools)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_ID, artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_ID, &header);
    Ok(tools)
}

fn overrepresented_input_hash(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileOverrepresentedArgs,
) -> Result<String> {
    if let Some(r2) = args.r2.as_deref() {
        return Ok(format!(
            "{}+{}",
            hash_file_sha256(&args.r1).context("hash overrepresented input r1")?,
            hash_file_sha256(r2).context("hash overrepresented input r2")?
        ));
    }
    hash_file_sha256(&args.r1).context("hash overrepresented input")
}

fn prepare_overrepresented_benchmark_setup(
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileOverrepresentedArgs,
    selected_tools: &[String],
) -> Result<OverrepresentedBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_ID, selected_tools, &registry, false)?;
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir_name = bench_dir_name(
        &bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_OVERREPRESENTED_SEQUENCES,
    )
    .ok_or_else(|| anyhow!("bench dir missing for {STAGE_ID}"))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_dna_infra::ensure_dir(&tools_root).context("create tools output dir")?;
    let input_hash = overrepresented_input_hash(args)?;
    Ok(OverrepresentedBenchmarkSetup { registry, tools, runner, bench_dir, tools_root, input_hash })
}

fn write_overrepresented_benchmark_explain(setup: &OverrepresentedBenchmarkSetup) -> Result<()> {
    write_explain_md(&setup.bench_dir, STAGE_ID, &setup.tools, &[], None)?;
    write_explain_plan_json(&setup.bench_dir, STAGE_ID, &setup.tools, &setup.registry, None)
}

fn ensure_overrepresented_benchmark_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(STAGE_ID, tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_ID, tools, platform, catalog)
}

fn prepare_overrepresented_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileOverrepresentedArgs,
    setup: &OverrepresentedBenchmarkSetup,
    tool: &str,
) -> Result<OverrepresentedToolPlan> {
    let out_dir = setup.tools_root.join(tool);
    bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
    let tool_spec = build_tool_execution_spec(STAGE_ID, tool, &setup.registry, catalog, platform)?;
    let plan = plan_with_options(
        &tool_spec,
        &args.r1,
        args.r2.as_deref(),
        &out_dir,
        args.threads,
        args.top_k,
    )?;
    let params_hash = stable_params_hash(&plan.params);
    let image_digest = benchmark_image_identity(&tool_spec);
    Ok(OverrepresentedToolPlan {
        tool: tool.to_string(),
        out_dir,
        tool_spec,
        plan,
        params_hash,
        image_digest,
    })
}

fn execute_overrepresented_tool(
    tool_plan: &OverrepresentedToolPlan,
    runner: RuntimeKind,
    jobs: usize,
) -> Result<OverrepresentedToolExecution> {
    let result = execute_plans_with_jobs(
        vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&tool_plan.plan)],
        runner,
        jobs,
    )?
    .into_iter()
    .next()
    .ok_or_else(|| anyhow!("missing execution result for {}", tool_plan.tool))?;
    Ok(OverrepresentedToolExecution { result })
}

fn overrepresented_tool_failure(
    tool_plan: &OverrepresentedToolPlan,
    execution: &OverrepresentedToolExecution,
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

fn overrepresented_execution_metrics(execution: &OverrepresentedToolExecution) -> ExecutionMetrics {
    ExecutionMetrics {
        runtime_s: execution.result.runtime_s,
        memory_mb: execution.result.memory_mb,
        exit_code: execution.result.exit_code,
    }
}

fn persist_overrepresented_record(
    store: &OverrepresentedBenchmarkStore,
    record: &BenchmarkRecord<FastqOverrepresentedMetrics>,
    insert_record: impl FnOnce(&BenchmarkRecord<FastqOverrepresentedMetrics>) -> Result<()>,
) -> Result<()> {
    append_jsonl(&store.jsonl_path, record).context("write bench.jsonl")?;
    insert_record(record)
}

fn prepare_overrepresented_artifacts(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileOverrepresentedArgs,
    plan: &StagePlanV1,
) -> Result<OverrepresentedArtifacts> {
    let output_tsv = required_output_path(plan, "overrepresented_sequences_tsv")?.to_path_buf();
    let output_json = required_output_path(plan, "overrepresented_sequences_json")?.to_path_buf();
    let report_json = required_output_path(plan, "report_json")?.to_path_buf();
    validate_overrepresented_artifact_paths(&output_tsv, &output_json, &report_json)?;
    if !output_tsv.exists() || !output_json.exists() {
        materialize_overrepresented_outputs(
            &args.r1,
            args.r2.as_deref(),
            &output_tsv,
            &output_json,
            args.top_k.unwrap_or(50).max(1),
        )?;
    }
    Ok(OverrepresentedArtifacts { output_tsv, output_json, report_json })
}

fn validate_overrepresented_artifact_paths(
    output_tsv: &Path,
    output_json: &Path,
    report_json: &Path,
) -> Result<()> {
    let mut paths = BTreeSet::new();
    for path in [output_tsv, output_json, report_json] {
        if !paths.insert(path) {
            return Err(anyhow!(
                "profile_overrepresented_sequences output path reused by multiple artifacts: {}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn observe_overrepresented_tool(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileOverrepresentedArgs,
    plan: &StagePlanV1,
) -> Result<OverrepresentedObservation> {
    let artifacts = prepare_overrepresented_artifacts(args, plan)?;
    let effective_params: FastqOverrepresentedProfileParams =
        serde_json::from_value(plan.effective_params.clone())
            .context("parse overrepresented effective params")?;
    let payload = read_overrepresented_payload(&artifacts.output_json)?;
    Ok(OverrepresentedObservation { artifacts, effective_params, payload })
}

fn build_overrepresented_metric_set(
    observation: &OverrepresentedObservation,
) -> Result<MetricSet<FastqOverrepresentedMetrics>> {
    let metric_set = metric_set(observation.payload.metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;
    Ok(metric_set)
}

fn build_overrepresented_report(
    inputs: OverrepresentedReportInputs<'_>,
) -> ProfileOverrepresentedReportV1 {
    ProfileOverrepresentedReportV1 {
        schema_version: PROFILE_OVERREPRESENTED_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_ID.to_string(),
        stage_id: STAGE_ID.to_string(),
        tool_id: inputs.tool.to_string(),
        paired_mode: if inputs.args.r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        threads: inputs.effective_params.threads,
        top_k: inputs.effective_params.top_k,
        input_r1: inputs.args.r1.display().to_string(),
        input_r2: inputs.args.r2.as_ref().map(|path| path.display().to_string()),
        overrepresented_sequences_tsv: inputs.artifacts.output_tsv.display().to_string(),
        overrepresented_sequences_json: inputs.artifacts.output_json.display().to_string(),
        report_json: inputs.artifacts.report_json.display().to_string(),
        sequence_count: inputs.payload.metrics.sequence_count,
        flagged_sequences: inputs.payload.metrics.flagged_sequences,
        top_fraction: inputs.payload.metrics.top_fraction,
        rows: inputs.payload.rows,
        runtime_s: Some(inputs.execution_metrics.runtime_s),
        memory_mb: Some(inputs.execution_metrics.memory_mb),
        exit_code: Some(inputs.execution_metrics.exit_code),
        raw_backend_report: None,
        raw_backend_report_format: None,
    }
}

fn write_overrepresented_report(
    report_json: &Path,
    report: &ProfileOverrepresentedReportV1,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(report_json, report).context("write overrepresented report")
}

fn write_overrepresented_metrics(
    out_dir: &Path,
    metric_set: &MetricSet<FastqOverrepresentedMetrics>,
) -> Result<()> {
    let metrics_json = serde_json::to_value(metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write overrepresented metrics")
}

fn write_overrepresented_artifacts(
    tool_plan: &OverrepresentedToolPlan,
    observation: &OverrepresentedObservation,
    report: &ProfileOverrepresentedReportV1,
    metric_set: &MetricSet<FastqOverrepresentedMetrics>,
) -> Result<()> {
    validate_overrepresented_report_identity(tool_plan, report)?;
    validate_overrepresented_report_metrics(report, &metric_set.metrics)?;
    write_overrepresented_report(&observation.artifacts.report_json, report)?;
    write_overrepresented_metrics(&tool_plan.out_dir, metric_set)?;
    validate_overrepresented_written_artifacts(&observation.artifacts, &tool_plan.out_dir)
}

fn validate_overrepresented_report_identity(
    tool_plan: &OverrepresentedToolPlan,
    report: &ProfileOverrepresentedReportV1,
) -> Result<()> {
    if report.schema_version != PROFILE_OVERREPRESENTED_REPORT_SCHEMA_VERSION {
        return Err(anyhow!(
            "profile_overrepresented_sequences report schema mismatch: expected {}, observed {}",
            PROFILE_OVERREPRESENTED_REPORT_SCHEMA_VERSION,
            report.schema_version
        ));
    }
    if report.stage != STAGE_ID || report.stage_id != STAGE_ID {
        return Err(anyhow!(
            "profile_overrepresented_sequences report stage mismatch: observed stage={} stage_id={}",
            report.stage,
            report.stage_id
        ));
    }
    if report.tool_id != tool_plan.tool {
        return Err(anyhow!(
            "profile_overrepresented_sequences report tool mismatch: expected {}, observed {}",
            tool_plan.tool,
            report.tool_id
        ));
    }
    Ok(())
}

fn validate_overrepresented_report_metrics(
    report: &ProfileOverrepresentedReportV1,
    metrics: &FastqOverrepresentedMetrics,
) -> Result<()> {
    if report.sequence_count != metrics.sequence_count {
        return Err(anyhow!(
            "profile_overrepresented_sequences report sequence count mismatch: expected {}, observed {}",
            metrics.sequence_count,
            report.sequence_count
        ));
    }
    if report.flagged_sequences != metrics.flagged_sequences {
        return Err(anyhow!(
            "profile_overrepresented_sequences report flagged count mismatch: expected {}, observed {}",
            metrics.flagged_sequences,
            report.flagged_sequences
        ));
    }
    if (report.top_fraction - metrics.top_fraction).abs() > f64::EPSILON {
        return Err(anyhow!(
            "profile_overrepresented_sequences report top fraction mismatch: expected {}, observed {}",
            metrics.top_fraction,
            report.top_fraction
        ));
    }
    validate_overrepresented_payload_metrics(Path::new(&report.report_json), metrics, &report.rows)
}

fn validate_overrepresented_written_artifacts(
    artifacts: &OverrepresentedArtifacts,
    out_dir: &Path,
) -> Result<()> {
    for path in [
        artifacts.output_tsv.as_path(),
        artifacts.output_json.as_path(),
        artifacts.report_json.as_path(),
        out_dir.join("metrics.json").as_path(),
    ] {
        let metadata = std::fs::metadata(path).with_context(|| {
            format!("read profile_overrepresented_sequences artifact {}", path.display())
        })?;
        if metadata.len() == 0 {
            return Err(anyhow!(
                "profile_overrepresented_sequences artifact is empty: {}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn build_overrepresented_record(
    inputs: &OverrepresentedRecordInputs<'_>,
    metric_set: MetricSet<FastqOverrepresentedMetrics>,
) -> Result<BenchmarkRecord<FastqOverrepresentedMetrics>> {
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
        execution: overrepresented_execution_metrics(inputs.execution),
        metrics: metric_set,
    };
    record.validate()?;
    Ok(record)
}

fn materialize_overrepresented_outputs(
    input_fastq: &Path,
    input_fastq_r2: Option<&Path>,
    output_tsv: &Path,
    output_json: &Path,
    top_k: u32,
) -> Result<()> {
    let mut counts = BTreeMap::<String, u64>::new();
    for path in std::iter::once(input_fastq).chain(input_fastq_r2.into_iter()) {
        accumulate_overrepresented_counts(path, &mut counts)?;
    }
    let total: u64 = counts.values().sum();
    if total == 0 {
        return Err(anyhow!("profile_overrepresented_sequences found no read evidence"));
    }
    let mut ranked = counts.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let top = ranked
        .iter()
        .take(usize::try_from(top_k).unwrap_or(usize::MAX))
        .cloned()
        .collect::<Vec<_>>();
    let top_fraction = if total == 0 {
        0.0
    } else {
        top.first().map_or(0.0, |(_, count)| u64_to_f64(*count) / u64_to_f64(total))
    };
    let flagged_sequences = top
        .iter()
        .filter(|(_, count)| total > 0 && (u64_to_f64(*count) / u64_to_f64(total)) >= 0.01)
        .count();

    let rows = top
        .iter()
        .map(|(sequence, count)| {
            let fraction = if total == 0 { 0.0 } else { u64_to_f64(*count) / u64_to_f64(total) };
            OverrepresentedSequenceRowV1 {
                sequence: sequence.clone(),
                count: *count,
                fraction,
                flag: if fraction >= 0.01 {
                    "overrepresented".to_string()
                } else {
                    "background".to_string()
                },
            }
        })
        .collect::<Vec<_>>();

    let mut tsv = String::from("sequence\tcount\tfraction\tflag\n");
    for row in &rows {
        tsv.push_str(&row.sequence);
        tsv.push('\t');
        tsv.push_str(&row.count.to_string());
        tsv.push('\t');
        let fraction_text = format!("{:.6}", row.fraction);
        tsv.push_str(&fraction_text);
        tsv.push('\t');
        tsv.push_str(&row.flag);
        tsv.push('\n');
    }
    bijux_dna_infra::atomic_write_bytes(output_tsv, tsv.as_bytes())?;
    bijux_dna_infra::atomic_write_json(
        output_json,
        &serde_json::json!({
            "schema_version": "bijux.fastq.profile_overrepresented_sequences.v1",
            "top_k": top_k,
            "sequence_count": usize_to_u64(rows.len()),
            "flagged_sequences": usize_to_u64(flagged_sequences),
            "top_fraction": top_fraction,
            "rows": rows,
        }),
    )?;
    Ok(())
}

fn accumulate_overrepresented_counts(
    path: &Path,
    counts: &mut BTreeMap<String, u64>,
) -> Result<()> {
    let lines = open_fastq_lines(path)?;
    let chunks = lines.chunks_exact(4);
    if !chunks.remainder().is_empty() {
        return Err(anyhow!("truncated FASTQ record in {}", path.display()));
    }
    for record in chunks {
        if !record[0].starts_with('@') || !record[2].starts_with('+') {
            return Err(anyhow!("invalid FASTQ framing in {}", path.display()));
        }
        if record[1].len() != record[3].len() {
            return Err(anyhow!("FASTQ sequence/quality length mismatch in {}", path.display()));
        }
        *counts.entry(record[1].trim().to_string()).or_insert(0) += 1;
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct OverrepresentedPayload {
    metrics: FastqOverrepresentedMetrics,
    rows: Vec<OverrepresentedSequenceRowV1>,
}

fn read_overrepresented_payload(path: &Path) -> Result<OverrepresentedPayload> {
    let value: serde_json::Value = serde_json::from_slice(
        &std::fs::read(path).with_context(|| format!("read {}", path.display()))?,
    )?;
    validate_overrepresented_payload_schema(path, &value)?;
    let rows = parse_overrepresented_rows(path, &value)?;
    let metrics = FastqOverrepresentedMetrics {
        sequence_count: value
            .get("sequence_count")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_else(|| usize_to_u64(rows.len())),
        flagged_sequences: value
            .get("flagged_sequences")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_else(|| {
                rows.iter()
                    .filter(|row| row.flag == "overrepresented")
                    .count()
                    .try_into()
                    .unwrap_or(u64::MAX)
            }),
        top_fraction: value
            .get("top_fraction")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or_else(|| rows.first().map_or(0.0, |row| row.fraction)),
    };
    metrics.validate()?;
    validate_overrepresented_payload_metrics(path, &metrics, &rows)?;
    Ok(OverrepresentedPayload { metrics, rows })
}

fn validate_overrepresented_payload_metrics(
    path: &Path,
    metrics: &FastqOverrepresentedMetrics,
    rows: &[OverrepresentedSequenceRowV1],
) -> Result<()> {
    let row_count = usize_to_u64(rows.len());
    if metrics.sequence_count != row_count {
        return Err(anyhow!(
            "overrepresented payload sequence count mismatch in {}: expected {}, observed {}",
            path.display(),
            row_count,
            metrics.sequence_count
        ));
    }
    let flagged_count = rows.iter().filter(|row| row.flag == "overrepresented").count();
    let flagged_count = usize_to_u64(flagged_count);
    if metrics.flagged_sequences != flagged_count {
        return Err(anyhow!(
            "overrepresented payload flagged count mismatch in {}: expected {}, observed {}",
            path.display(),
            flagged_count,
            metrics.flagged_sequences
        ));
    }
    let top_fraction = rows.first().map_or(0.0, |row| row.fraction);
    if (metrics.top_fraction - top_fraction).abs() > f64::EPSILON {
        return Err(anyhow!(
            "overrepresented payload top fraction mismatch in {}: expected {}, observed {}",
            path.display(),
            top_fraction,
            metrics.top_fraction
        ));
    }
    Ok(())
}

fn parse_overrepresented_rows(
    path: &Path,
    value: &serde_json::Value,
) -> Result<Vec<OverrepresentedSequenceRowV1>> {
    let rows = value
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("overrepresented payload missing rows: {}", path.display()))?;
    rows.iter()
        .enumerate()
        .map(|(index, entry)| {
            let sequence = entry
                .get("sequence")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow!("overrepresented row {index} missing sequence"))?;
            let count = entry
                .get("count")
                .and_then(serde_json::Value::as_u64)
                .ok_or_else(|| anyhow!("overrepresented row {index} missing count"))?;
            let fraction = entry
                .get("fraction")
                .and_then(serde_json::Value::as_f64)
                .ok_or_else(|| anyhow!("overrepresented row {index} missing fraction"))?;
            let flag = entry
                .get("flag")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow!("overrepresented row {index} missing flag"))?;
            Ok(OverrepresentedSequenceRowV1 {
                sequence: sequence.to_string(),
                count,
                fraction,
                flag: flag.to_string(),
            })
        })
        .collect()
}

fn validate_overrepresented_payload_schema(path: &Path, value: &serde_json::Value) -> Result<()> {
    let schema_version =
        value.get("schema_version").and_then(serde_json::Value::as_str).ok_or_else(|| {
            anyhow!("overrepresented payload missing schema_version: {}", path.display())
        })?;
    if schema_version != "bijux.fastq.profile_overrepresented_sequences.v1" {
        return Err(anyhow!(
            "overrepresented payload schema mismatch in {}: expected {}, observed {}",
            path.display(),
            "bijux.fastq.profile_overrepresented_sequences.v1",
            schema_version
        ));
    }
    Ok(())
}

fn required_output_path<'a>(
    plan: &'a bijux_dna_stage_contract::StagePlanV1,
    artifact_id: &str,
) -> Result<&'a Path> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_id)
        .map(|artifact| artifact.path.as_path())
        .ok_or_else(|| anyhow!("missing required output artifact `{artifact_id}`"))
}

fn resolve_smoke_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

fn copy_smoke_artifact(from: &Path, to: &Path) -> Result<()> {
    if let Some(parent) = to.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    std::fs::copy(from, to)
        .with_context(|| format!("copy smoke artifact {} -> {}", from.display(), to.display()))?;
    Ok(())
}

fn open_fastq_lines(path: &Path) -> Result<Vec<String>> {
    let file =
        std::fs::File::open(path).with_context(|| format!("open fastq {}", path.display()))?;
    if path.extension().and_then(|ext| ext.to_str()) == Some("gz") {
        let decoder = flate2::read::MultiGzDecoder::new(file);
        let reader = BufReader::new(decoder);
        return reader
            .lines()
            .collect::<std::result::Result<Vec<_>, _>>()
            .with_context(|| format!("read gz fastq {}", path.display()));
    }
    let reader = BufReader::new(file);
    reader
        .lines()
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("read fastq {}", path.display()))
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

fn usize_to_u64(value: usize) -> u64 {
    value.try_into().unwrap_or(u64::MAX)
}
