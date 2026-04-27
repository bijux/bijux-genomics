use std::collections::HashMap;
use std::path::PathBuf;

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{fetch_fastq_stats_v1, insert_fastq_stats_v1};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkContext, BenchmarkRecord, FastqStatsMetrics,
    LengthHistogramBin, MetricSet,
};
use bijux_dna_core::metrics::MetricContextV1;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use bijux_dna_runtime::{RunProvenanceV1, StageObservabilityContextV1};

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use bijux_dna_core::contract::validate_execution_outputs;
use bijux_dna_core::prelude::measure::SeqkitMetrics;
use bijux_dna_domain_fastq::{
    metrics::SeqkitToolMetricsV1, PairedMode, ProfileReadsHistogramBinV1,
    ProfileReadsMateSummaryV1, ProfileReadsReportV1, PROFILE_READS_REPORT_SCHEMA_VERSION,
};
use bijux_dna_infra::hash_file_sha256;
use bijux_dna_infra::{bench_base_dir, bench_tools_dir};
use bijux_dna_planner_fastq::select_stats_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::profile_reads::plan_stats_with_threads;
use bijux_dna_planner_fastq::stage_api::observer::{
    input_fastq_stats, length_histogram_command, parse_length_histogram, parse_seqkit_stats,
};
use bijux_dna_planner_fastq::stage_api::StagePlanJson;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind,
};
use bijux_dna_runner::backend::docker::executor::resolve_image_for_run;
use bijux_dna_runner::step_runner::execute_observer_command;
use bijux_dna_runtime::recording::{
    compute_run_id, prepare_tool_run_dirs, write_execution_logs, write_metrics_envelope,
    write_metrics_json, write_run_manifest, write_stage_plan_json, RunArtifactInput, RunDirs,
};

use super::trim_bench_common::benchmark_image_identity;
use crate::public_bridge::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_PROFILE_READS,
};
use bijux_dna_core::contract::{ContractVersion, ExecutionManifest, ToolRegistry};
use bijux_dna_planner_fastq::stage_api::RawFailure;

/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning, execution, or metric recording fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_stats_neutral<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqStatsArgs,
) -> Result<BenchOutcome<FastqStatsMetrics>> {
    let tools = select_stats_benchmark_tools(args)?;
    let setup = prepare_stats_benchmark_setup(catalog, platform, runner_override, args, &tools)?;
    let selected = setup.tools.clone();
    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_PROFILE_READS.as_str());
    let all_tools: Vec<String> = setup
        .registry
        .tools_for_stage(&stage_id)
        .iter()
        .map(|tool| tool.tool_id.to_string())
        .collect();
    let excluded: Vec<String> =
        all_tools.into_iter().filter(|tool| !selected.contains(tool)).collect();
    if args.explain {
        write_explain_md(
            &setup.bench_inputs.bench_dir,
            STAGE_PROFILE_READS.as_str(),
            &selected,
            &excluded,
            None,
        )?;
        write_explain_plan_json(
            &setup.bench_inputs.bench_dir,
            STAGE_PROFILE_READS.as_str(),
            &selected,
            &setup.registry,
            None,
        )?;
    }
    ensure_image_qa_passed(STAGE_PROFILE_READS.as_str(), &setup.tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_PROFILE_READS.as_str(), &setup.tools, platform, catalog)?;

    let store = StatsBenchmarkStore::from_bench_inputs(&setup.bench_inputs);
    let conn = bijux_dna_analyze::open_sqlite(&store.sqlite_path).context("open bench sqlite")?;
    let mut records: Vec<BenchmarkRecord<FastqStatsMetrics>> = Vec::new();
    let mut new_records: Vec<BenchmarkRecord<FastqStatsMetrics>> = Vec::new();
    let mut failures: Vec<RawFailure> = Vec::new();

    for tool in &setup.tools {
        let tool_plan = prepare_stats_tool_plan(catalog, platform, args, &setup, tool)?;
        let cache_identity =
            StatsCacheIdentity::from_plan(platform, &setup.bench_inputs, &tool_plan);
        let cached = fetch_fastq_stats_v1(
            &conn,
            &cache_identity.tool,
            &cache_identity.tool_version,
            &cache_identity.image_digest,
            &cache_identity.runner,
            &cache_identity.platform,
            &cache_identity.input_hash,
            &cache_identity.params_hash,
        );
        if let Ok(Some(record)) = cached {
            records.push(record);
            continue;
        }
        match run_stats_tool(platform, args, &setup.bench_inputs, &tool_plan) {
            Ok(record) => new_records.push(record),
            Err(err) => failures.push(RawFailure {
                stage: STAGE_PROFILE_READS.as_str().to_string(),
                tool: tool_plan.tool.clone(),
                reason: err.to_string(),
                category: ErrorCategory::ToolError,
            }),
        }
    }

    records.extend(new_records.iter().cloned());

    for record in &new_records {
        append_jsonl(&store.jsonl_path, record).context("write bench.jsonl")?;
    }

    for record in &new_records {
        insert_fastq_stats_v1(&conn, record).context("insert bench sqlite")?;
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: setup.bench_inputs.bench_dir,
        explain: args.explain,
    })
}

fn select_stats_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqStatsArgs,
) -> Result<Vec<String>> {
    let tools = select_stats_tools(&args.tools)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_PROFILE_READS.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_PROFILE_READS.as_str(), &header);
    Ok(tools)
}

struct StatsBenchInputs {
    runner: RuntimeKind,
    r1: PathBuf,
    r2: Option<PathBuf>,
    input_hash: String,
    input_stats: SeqkitMetrics,
    input_stats_r2: Option<SeqkitMetrics>,
    length_hist: Vec<LengthHistogramBin>,
    length_hist_r2: Option<Vec<LengthHistogramBin>>,
    bench_dir: PathBuf,
    tools_root: PathBuf,
}

struct StatsBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    bench_inputs: StatsBenchInputs,
}

struct StatsBenchmarkStore {
    sqlite_path: PathBuf,
    jsonl_path: PathBuf,
}

impl StatsBenchmarkStore {
    fn from_bench_inputs(bench_inputs: &StatsBenchInputs) -> Self {
        Self {
            sqlite_path: bench_inputs.bench_dir.join("bench.sqlite"),
            jsonl_path: bench_inputs.bench_dir.join("bench.jsonl"),
        }
    }
}

struct StatsToolPlan {
    tool: String,
    tool_spec: bijux_dna_core::prelude::ToolExecutionSpecV1,
    plan: bijux_dna_stage_contract::StagePlanV1,
    params_hash: String,
    image_digest: String,
}

struct StatsToolExecution {
    result: StageResultV1,
}

struct StatsObservation {
    metric_set: MetricSet<FastqStatsMetrics>,
}

struct StatsCacheIdentity {
    tool: String,
    tool_version: String,
    image_digest: String,
    runner: String,
    platform: String,
    input_hash: String,
    params_hash: String,
}

impl StatsCacheIdentity {
    fn from_plan(
        platform: &PlatformSpec,
        bench_inputs: &StatsBenchInputs,
        tool_plan: &StatsToolPlan,
    ) -> Self {
        Self {
            tool: tool_plan.tool.clone(),
            tool_version: tool_plan.tool_spec.tool_version.clone(),
            image_digest: tool_plan.image_digest.clone(),
            runner: bench_inputs.runner.to_string(),
            platform: platform.name.clone(),
            input_hash: bench_inputs.input_hash.clone(),
            params_hash: tool_plan.params_hash.clone(),
        }
    }
}

fn prepare_stats_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqStatsArgs,
    setup: &StatsBenchmarkSetup,
    tool: &str,
) -> Result<StatsToolPlan> {
    let tool_spec = build_tool_execution_spec(
        STAGE_PROFILE_READS.as_str(),
        tool,
        &setup.registry,
        catalog,
        platform,
    )?;
    let tool_dir = setup.bench_inputs.tools_root.join(tool);
    let plan = plan_stats_with_threads(
        &tool_spec,
        &setup.bench_inputs.r1,
        setup.bench_inputs.r2.as_deref(),
        &tool_dir,
        args.threads,
    )?;
    let params_hash = stable_params_hash(&plan.params);
    let image_digest = benchmark_image_identity(&tool_spec);
    Ok(StatsToolPlan { tool: tool.to_string(), tool_spec, plan, params_hash, image_digest })
}

fn execute_stats_tool(
    tool_plan: &StatsToolPlan,
    runner: RuntimeKind,
    jobs: usize,
) -> Result<StatsToolExecution> {
    let step = bijux_dna_stage_contract::execution_step_from_stage_plan(&tool_plan.plan);
    let result = execute_plans_with_jobs(vec![step], runner, jobs)?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing execution result for {}", tool_plan.tool))?;
    Ok(StatsToolExecution { result })
}

fn observe_profile_reads(
    bench_inputs: &StatsBenchInputs,
    tool_plan: &StatsToolPlan,
    execution: &StatsToolExecution,
) -> Result<StatsObservation> {
    let length_histogram = combine_length_histograms(
        &bench_inputs.length_hist,
        bench_inputs.length_hist_r2.as_deref(),
    );
    let backend_rows = parse_seqkit_stats_rows(&execution.result.stdout).unwrap_or_else(|_| {
        fallback_seqkit_rows(&bench_inputs.input_stats, bench_inputs.input_stats_r2.as_ref())
    });
    materialize_profile_reads_outputs(
        &tool_plan.plan,
        &backend_rows,
        &length_histogram,
        &execution_metrics_from_stage_result(&execution.result),
    )?;
    let report = std::fs::read_to_string(required_plan_output_path(&tool_plan.plan, "qc_json")?)
        .ok()
        .and_then(|raw| bijux_dna_domain_fastq::observer::parse_profile_reads_report(&raw).ok())
        .ok_or_else(|| anyhow!("profile_reads governed report was not materialized"))?;
    let metric_set = build_profile_reads_metric_set(&report)?;
    Ok(StatsObservation { metric_set })
}

fn build_profile_reads_metric_set(
    report: &ProfileReadsReportV1,
) -> Result<MetricSet<FastqStatsMetrics>> {
    let metrics = FastqStatsMetrics {
        reads_total: report.reads_total,
        bases_total: report.bases_total,
        mean_q: report.mean_q,
        gc_percent: report.gc_percent,
        length_histogram: report
            .length_histogram
            .iter()
            .map(|bin| LengthHistogramBin { length: bin.length, count: bin.count })
            .collect(),
    };
    let metric_set = metric_set(metrics);
    bijux_dna_analyze::validate_metric_set(&metric_set)?;
    Ok(metric_set)
}

fn write_profile_reads_execution_manifest(
    platform: &PlatformSpec,
    bench_inputs: &StatsBenchInputs,
    tool_plan: &StatsToolPlan,
    run_id: &str,
    out_dir: &std::path::Path,
    run_dirs: &RunDirs,
    execution: &StatsToolExecution,
) -> Result<()> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_PROFILE_READS.as_str());
    let tool_manifest = registry
        .tool_by_id(&stage_id, &bijux_dna_core::ids::ToolId::new(&tool_plan.tool))
        .ok_or_else(|| anyhow!("tool {} missing from manifests", tool_plan.tool))?;
    validate_execution_outputs(&tool_manifest.execution_contract, out_dir)?;
    let manifest = ExecutionManifest {
        contract_version: ContractVersion::v1(),
        run_id: run_id.to_string(),
        stage: STAGE_PROFILE_READS.as_str().to_string(),
        tool: tool_plan.tool.clone(),
        tool_version: tool_plan.tool_spec.tool_version.clone(),
        image_digest: tool_plan.image_digest.clone(),
        command: execution.result.command.clone(),
        input_hashes: vec![bench_inputs.input_hash.clone()],
        input_files: vec![bench_inputs.r1.display().to_string()],
        output_dir: out_dir.display().to_string(),
        runner: bench_inputs.runner.to_string(),
        platform: platform.name.clone(),
        arch: platform.arch.clone(),
    };
    bijux_dna_infra::atomic_write_json(&run_dirs.manifest_path, &manifest)
        .context("write execution manifest")?;
    write_execution_logs(&run_dirs.logs_dir, &execution.result.stdout, &execution.result.stderr)
}

fn build_profile_reads_context(
    platform: &PlatformSpec,
    bench_inputs: &StatsBenchInputs,
    tool_plan: &StatsToolPlan,
    params: &serde_json::Value,
) -> BenchmarkContext {
    BenchmarkContext {
        tool: tool_plan.tool.clone(),
        tool_version: tool_plan.tool_spec.tool_version.clone(),
        image_digest: tool_plan.image_digest.clone(),
        runner: bench_inputs.runner.to_string(),
        platform: platform.name.clone(),
        input_hash: bench_inputs.input_hash.clone(),
        parameters: params.clone().into(),
    }
}

fn build_profile_reads_observability_context(
    platform: &PlatformSpec,
    bench_inputs: &StatsBenchInputs,
    tool_plan: &StatsToolPlan,
    params: &serde_json::Value,
    parameters_json_normalized: serde_json::Value,
) -> StageObservabilityContextV1 {
    StageObservabilityContextV1 {
        stage_id: STAGE_PROFILE_READS.as_str().to_string(),
        stage_version: tool_plan.plan.stage_version.0,
        tool_id: tool_plan.tool.clone(),
        tool_version: tool_plan.tool_spec.tool_version.clone(),
        input_fingerprint: bench_inputs.input_hash.clone(),
        parameters_fingerprint: tool_plan.params_hash.clone(),
        parameters_json: params.clone(),
        parameters_json_normalized,
        metric_context: MetricContextV1 {
            tool_id: tool_plan.tool.clone(),
            tool_version: tool_plan.tool_spec.tool_version.clone(),
            image_digest: Some(tool_plan.image_digest.clone()),
            runner: bench_inputs.runner.to_string(),
            platform: platform.name.clone(),
            input_hash: bench_inputs.input_hash.clone(),
            params_hash: tool_plan.params_hash.clone(),
            presets: std::collections::BTreeMap::new(),
            banks: std::collections::BTreeMap::new(),
        },
    }
}

fn prepare_stats_benchmark_setup<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqStatsArgs,
    selected_tools: &[String],
) -> Result<StatsBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools =
        filter_tools_by_role(STAGE_PROFILE_READS.as_str(), selected_tools, &registry, false)?;
    let bench_inputs = prepare_stats_bench(catalog, platform, runner_override, args)?;
    Ok(StatsBenchmarkSetup { registry, tools, bench_inputs })
}

#[allow(clippy::too_many_lines)]
fn prepare_stats_bench<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqStatsArgs,
) -> Result<StatsBenchInputs> {
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir_name = bench_dir_name(&STAGE_PROFILE_READS)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_PROFILE_READS.as_str()))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_dna_infra::ensure_dir(&tools_root).context("create tools output dir")?;

    println!("planned tools: {}", select_stats_tools(&args.tools)?.join(", "));

    let r1 = args.r1.canonicalize().context("resolve r1 path")?;
    let r1_dir = r1.parent().ok_or_else(|| anyhow!("r1 has no parent"))?.to_path_buf();

    let tool_id = bijux_dna_planner_fastq::stage_api::TOOL_SEQKIT;
    let tool_spec =
        catalog.get(tool_id).ok_or_else(|| anyhow!("{tool_id} missing from images.toml"))?;
    let tool_image = resolve_image_for_run(tool_spec, platform)?;

    let input_hash = if let Some(r2) = args.r2.as_deref() {
        format!("{}+{}", hash_file_sha256(&r1)?, hash_file_sha256(r2)?)
    } else {
        hash_file_sha256(&r1)?
    };
    let stats_spec = input_fastq_stats(&r1_dir, &r1)?;
    let stats_output = execute_observer_command(
        &tool_image.full_name,
        stats_spec.mount_dir.as_path(),
        &stats_spec.args,
        runner,
    )?;
    if stats_output.exit_code != 0 {
        return Err(anyhow!("seqkit stats failed: {}", stats_output.stderr));
    }
    let input_stats = parse_seqkit_stats(&stats_output.stdout)?;
    let (r2, input_stats_r2, length_hist_r2) = if let Some(r2) = args.r2.as_deref() {
        let r2 = r2.canonicalize().context("resolve r2 path")?;
        let r2_dir = r2.parent().ok_or_else(|| anyhow!("r2 has no parent"))?.to_path_buf();
        let stats_spec = input_fastq_stats(&r2_dir, &r2)?;
        let stats_output = execute_observer_command(
            &tool_image.full_name,
            stats_spec.mount_dir.as_path(),
            &stats_spec.args,
            runner,
        )?;
        if stats_output.exit_code != 0 {
            return Err(anyhow!("seqkit stats failed for r2: {}", stats_output.stderr));
        }
        let hist_spec = length_histogram_command(&r2_dir, &r2)?;
        let hist_output = execute_observer_command(
            &tool_image.full_name,
            hist_spec.mount_dir.as_path(),
            &hist_spec.args,
            runner,
        )?;
        if hist_output.exit_code != 0 {
            return Err(anyhow!("seqkit length histogram failed for r2: {}", hist_output.stderr));
        }
        (
            Some(r2),
            Some(parse_seqkit_stats(&stats_output.stdout)?),
            Some(
                parse_length_histogram(&hist_output.stdout)?
                    .into_iter()
                    .map(|(length, count)| LengthHistogramBin { length, count })
                    .collect(),
            ),
        )
    } else {
        (None, None, None)
    };

    let hist_spec = length_histogram_command(&r1_dir, &r1)?;
    let hist_output = execute_observer_command(
        &tool_image.full_name,
        hist_spec.mount_dir.as_path(),
        &hist_spec.args,
        runner,
    )?;
    if hist_output.exit_code != 0 {
        return Err(anyhow!("seqkit length histogram failed: {}", hist_output.stderr));
    }
    let length_hist = parse_length_histogram(&hist_output.stdout)?
        .into_iter()
        .map(|(length, count)| LengthHistogramBin { length, count })
        .collect();

    Ok(StatsBenchInputs {
        runner,
        r1,
        r2,
        input_hash,
        input_stats,
        input_stats_r2,
        length_hist,
        length_hist_r2,
        bench_dir,
        tools_root,
    })
}

fn run_stats_tool(
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqStatsArgs,
    bench_inputs: &StatsBenchInputs,
    tool_plan: &StatsToolPlan,
) -> Result<BenchmarkRecord<FastqStatsMetrics>> {
    let tool = &tool_plan.tool;
    println!("→ stats {tool}");
    let plan = &tool_plan.plan;
    let plan_json = StagePlanJson::from_plan(plan);
    let params = plan.params.clone();
    let param_hash = tool_plan.params_hash.clone();
    let image_digest = tool_plan.image_digest.clone();
    let run_id = compute_run_id(
        STAGE_PROFILE_READS.as_str(),
        tool,
        &image_digest,
        &bench_inputs.input_hash,
        &param_hash,
    );
    let run_dirs = prepare_tool_run_dirs(&bench_inputs.tools_root, tool, &run_id)?;
    let out_dir = run_dirs.artifacts_dir.clone();
    let _plan_path = write_stage_plan_json(&run_dirs, "fastq_stats_neutral.plan.json", &plan_json)?;
    let execution = execute_stats_tool(tool_plan, bench_inputs.runner, bench_jobs(args.jobs))?;
    let observation = observe_profile_reads(bench_inputs, tool_plan, &execution)?;

    write_profile_reads_execution_manifest(
        platform,
        bench_inputs,
        tool_plan,
        &run_id,
        &out_dir,
        &run_dirs,
        &execution,
    )?;
    let context = build_profile_reads_context(platform, bench_inputs, tool_plan, &params);
    let execution_metrics = execution_metrics_from_stage_result(&execution.result);
    let metrics_json = serde_json::to_value(&observation.metric_set)?;
    let parameters_json_normalized =
        bijux_dna_core::contract::canonical::parameters_json_canonicalization(&params);
    let stage_ctx = build_profile_reads_observability_context(
        platform,
        bench_inputs,
        tool_plan,
        &params,
        parameters_json_normalized,
    );
    let _metrics_envelope_path = write_metrics_envelope(
        &bijux_dna_runtime::recording::run_artifacts_dir_for_out(&out_dir),
        &stage_ctx,
        &metrics_json,
        std::slice::from_ref(&bench_inputs.input_hash),
    )?;
    let envelope = &observation.metric_set;
    write_metrics_json(&run_dirs, &execution_metrics, envelope)?;
    let adapter_bank_path = bijux_dna_planner_fastq::stage_api::adapter_bank_path();
    let run_provenance = RunProvenanceV1 {
        schema_version: "bijux.run_provenance.v1".to_string(),
        tool_image_digest: Some(image_digest.clone()),
        tool_version: tool_plan.tool_spec.tool_version.clone(),
        params_hash: param_hash.clone(),
        input_hashes: vec![bench_inputs.input_hash.clone()],
        reference_genome: None,
        pipeline_id: STAGE_PROFILE_READS.as_str().to_string(),
        git_commit: std::env::var("BIJUX_GIT_COMMIT")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "not_recorded".to_string()),
        build_profile: std::env::var("BIJUX_BUILD_PROFILE")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "not_recorded".to_string()),
        plan_hash: std::env::var("BIJUX_PLAN_HASH").ok(),
    };
    let extra_artifacts = [RunArtifactInput { name: "adapter_bank", path: adapter_bank_path }];
    let stage_contract_hash = None;
    write_run_manifest(
        &run_dirs,
        STAGE_PROFILE_READS.as_str(),
        tool,
        &run_provenance,
        stage_contract_hash,
        &extra_artifacts,
    )?;
    let record =
        BenchmarkRecord { context, execution: execution_metrics, metrics: observation.metric_set };
    record.validate()?;
    Ok(record)
}

fn required_plan_output_path<'a>(
    plan: &'a bijux_dna_stage_contract::StagePlanV1,
    artifact_name: &str,
) -> Result<&'a std::path::Path> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_name)
        .map(|artifact| artifact.path.as_path())
        .ok_or_else(|| anyhow!("profile_reads plan missing `{artifact_name}` output"))
}

fn execution_metrics_from_stage_result(execution: &StageResultV1) -> ExecutionMetrics {
    ExecutionMetrics {
        runtime_s: execution.runtime_s,
        memory_mb: execution.memory_mb,
        exit_code: execution.exit_code,
    }
}

fn parse_seqkit_stats_rows(stdout: &str) -> Result<Vec<ProfileReadsMateSummaryV1>> {
    let mut lines = stdout.lines();
    let header = lines.next().ok_or_else(|| anyhow!("empty seqkit stats stdout"))?;
    let header_fields: Vec<&str> = header.split('\t').collect();
    let col_index = |name: &str| -> Result<usize> {
        header_fields
            .iter()
            .position(|field| field == &name)
            .ok_or_else(|| anyhow!("seqkit stats header missing `{name}`"))
    };
    let reads_idx = col_index("num_seqs")?;
    let bases_idx = col_index("sum_len")?;
    let mean_q_idx =
        header_fields.iter().position(|field| field == &"avg_qual" || field == &"mean_qual");
    let gc_idx =
        header_fields.iter().position(|field| field.to_ascii_lowercase().starts_with("gc"));
    let file_idx = header_fields.iter().position(|field| field == &"file").unwrap_or(0);

    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        rows.push(ProfileReadsMateSummaryV1 {
            label: fields.get(file_idx).copied().unwrap_or("reads").to_string(),
            reads: fields
                .get(reads_idx)
                .ok_or_else(|| anyhow!("seqkit row missing reads"))?
                .parse()?,
            bases: fields
                .get(bases_idx)
                .ok_or_else(|| anyhow!("seqkit row missing bases"))?
                .parse()?,
            mean_q: mean_q_idx
                .and_then(|idx| fields.get(idx))
                .and_then(|value| value.parse::<f64>().ok()),
            gc_percent: gc_idx
                .and_then(|idx| fields.get(idx))
                .and_then(|value| value.parse::<f64>().ok()),
        });
    }
    if rows.is_empty() {
        return Err(anyhow!("seqkit stats stdout contained no data rows"));
    }
    Ok(rows)
}

fn fallback_seqkit_rows(
    r1: &SeqkitMetrics,
    r2: Option<&SeqkitMetrics>,
) -> Vec<ProfileReadsMateSummaryV1> {
    let mut rows = vec![ProfileReadsMateSummaryV1 {
        label: "reads_r1".to_string(),
        reads: r1.reads,
        bases: r1.bases,
        mean_q: Some(r1.mean_q),
        gc_percent: Some(r1.gc_percent),
    }];
    if let Some(r2) = r2 {
        rows.push(ProfileReadsMateSummaryV1 {
            label: "reads_r2".to_string(),
            reads: r2.reads,
            bases: r2.bases,
            mean_q: Some(r2.mean_q),
            gc_percent: Some(r2.gc_percent),
        });
    }
    rows
}

fn materialize_profile_reads_outputs(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    mate_summaries: &[ProfileReadsMateSummaryV1],
    length_histogram: &[LengthHistogramBin],
    execution_metrics: &ExecutionMetrics,
) -> Result<()> {
    let qc_json = required_plan_output_path(plan, "qc_json")?;
    let qc_tsv = required_plan_output_path(plan, "qc_tsv")?;
    let qc_plots_dir = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "qc_plots_dir")
        .map(|artifact| artifact.path.clone());

    let reads_total = mate_summaries.iter().map(|summary| summary.reads).sum::<u64>();
    let bases_total = mate_summaries.iter().map(|summary| summary.bases).sum::<u64>();
    let mean_q = weighted_optional_metric(mate_summaries, |summary| summary.mean_q);
    let gc_percent = weighted_optional_metric(mate_summaries, |summary| summary.gc_percent);
    let backend_metrics = mate_summaries
        .iter()
        .map(|summary| SeqkitToolMetricsV1 {
            schema_version: "bijux.seqkit.metrics.v1".to_string(),
            reads: summary.reads,
            bases: summary.bases,
            mean_q: summary.mean_q,
            gc_percent: summary.gc_percent,
        })
        .collect::<Vec<_>>();
    let report = ProfileReadsReportV1 {
        schema_version: PROFILE_READS_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_PROFILE_READS.as_str().to_string(),
        stage_id: STAGE_PROFILE_READS.as_str().to_string(),
        tool_id: plan.tool_id.to_string(),
        paired_mode: if mate_summaries.len() > 1 {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        threads: plan.resources.threads,
        input_r1: plan
            .io
            .inputs
            .first()
            .map(|artifact| artifact.path.display().to_string())
            .ok_or_else(|| anyhow!("profile_reads report requires declared primary input"))?,
        input_r2: plan
            .io
            .inputs
            .iter()
            .find(|artifact| artifact.name.as_str() == "reads_r2")
            .map(|artifact| artifact.path.display().to_string()),
        qc_json: qc_json.display().to_string(),
        qc_tsv: qc_tsv.display().to_string(),
        qc_plots_dir: qc_plots_dir.as_ref().map(|path| path.display().to_string()),
        length_histogram_source: "seqkit_fx2tab".to_string(),
        reads_total,
        bases_total,
        mean_q,
        gc_percent,
        length_histogram: length_histogram
            .iter()
            .map(|bin| ProfileReadsHistogramBinV1 { length: bin.length, count: bin.count })
            .collect(),
        mate_summaries: mate_summaries.to_vec(),
        runtime_s: Some(execution_metrics.runtime_s),
        memory_mb: Some(execution_metrics.memory_mb),
        exit_code: Some(execution_metrics.exit_code),
        raw_backend_report: Some(qc_tsv.display().to_string()),
        raw_backend_report_format: Some("seqkit_stats_tsv".to_string()),
        backend_metrics: Some(backend_metrics),
    };

    bijux_dna_infra::atomic_write_json(qc_json, &report)?;
    bijux_dna_infra::atomic_write_bytes(qc_tsv, profile_reads_tsv(mate_summaries).as_bytes())?;
    if let Some(plots_dir) = qc_plots_dir {
        bijux_dna_infra::ensure_dir(&plots_dir)?;
        let histogram_json = plots_dir.join("length_histogram.json");
        let histogram_tsv = plots_dir.join("length_histogram.tsv");
        let histogram_payload = serde_json::json!({
            "schema_version": "bijux.fastq.profile_reads.length_histogram.v1",
            "bins": length_histogram
                .iter()
                .map(|bin| serde_json::json!({
                    "length": bin.length,
                    "count": bin.count,
                }))
                .collect::<Vec<_>>(),
        });
        bijux_dna_infra::atomic_write_json(&histogram_json, &histogram_payload)?;
        bijux_dna_infra::atomic_write_bytes(
            &histogram_tsv,
            profile_reads_histogram_tsv(length_histogram).as_bytes(),
        )?;
    }
    Ok(())
}

fn weighted_optional_metric(
    summaries: &[ProfileReadsMateSummaryV1],
    selector: impl Fn(&ProfileReadsMateSummaryV1) -> Option<f64>,
) -> f64 {
    let total_bases = summaries.iter().map(|summary| summary.bases).sum::<u64>();
    if total_bases == 0 {
        return 0.0;
    }
    let weighted_sum = summaries.iter().fold(0.0, |acc, summary| {
        acc + selector(summary).unwrap_or(0.0) * u64_to_f64(summary.bases)
    });
    weighted_sum / u64_to_f64(total_bases)
}

fn profile_reads_tsv(mate_summaries: &[ProfileReadsMateSummaryV1]) -> String {
    let mut out = String::from("label\treads\tbases\tmean_q\tgc_percent\n");
    for summary in mate_summaries {
        out.push_str(&summary.label);
        out.push('\t');
        out.push_str(&summary.reads.to_string());
        out.push('\t');
        out.push_str(&summary.bases.to_string());
        out.push('\t');
        out.push_str(&summary.mean_q.map_or_else(String::new, |value| format!("{value:.3}")));
        out.push('\t');
        out.push_str(&summary.gc_percent.map_or_else(String::new, |value| format!("{value:.3}")));
        out.push('\n');
    }
    out
}

fn profile_reads_histogram_tsv(length_histogram: &[LengthHistogramBin]) -> String {
    let mut out = String::from("length\tcount\n");
    for bin in length_histogram {
        out.push_str(&bin.length.to_string());
        out.push('\t');
        out.push_str(&bin.count.to_string());
        out.push('\n');
    }
    out
}

fn combine_length_histograms(
    primary: &[LengthHistogramBin],
    secondary: Option<&[LengthHistogramBin]>,
) -> Vec<LengthHistogramBin> {
    let mut bins = std::collections::BTreeMap::<u64, u64>::new();
    for bin in primary {
        *bins.entry(bin.length).or_insert(0) += bin.count;
    }
    if let Some(secondary) = secondary {
        for bin in secondary {
            *bins.entry(bin.length).or_insert(0) += bin.count;
        }
    }
    bins.into_iter().map(|(length, count)| LengthHistogramBin { length, count }).collect()
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}
