use std::collections::HashMap;
use std::path::PathBuf;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_normalize_primers_v1, insert_fastq_normalize_primers_v1,
};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkRecord, FastqNormalizePrimersMetrics, MetricSet,
};
use bijux_dna_core::contract::{ExecutionStep, ToolRegistry};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::params_hash;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::params::PairedMode;
use bijux_dna_domain_fastq::{NormalizePrimersReportV1, NORMALIZE_PRIMERS_REPORT_SCHEMA_VERSION};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_planner_fastq::tool_adapters::fastq::normalize_primers::NormalizePrimersPlanOptions;
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use bijux_dna_stage_contract::StagePlanV1;
use uuid::Uuid;

use crate::internal::fastq::stages::preprocess::{
    enforce_amplicon_qc_thresholds_for_bench, materialize_amplicon_stage_outputs_for_bench,
    resolve_primer_set_governance, PrimerSetGovernance,
};
use crate::internal::fastq::stages::trim_bench_common::{
    build_benchmark_context, observe_fastq_stats,
};
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};

const STAGE_ID: &str = "fastq.normalize_primers";

/// Benchmark FASTQ primer normalization tools under governed contracts.
///
/// # Errors
/// Returns an error if planning, execution, report parsing, or persistence fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_normalize_primers<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqNormalizePrimersArgs,
) -> Result<BenchOutcome<FastqNormalizePrimersMetrics>> {
    let selected_tools = select_normalize_primers_benchmark_tools(args)?;
    let setup =
        prepare_normalize_primers_setup(catalog, platform, runner_override, args, &selected_tools)?;

    if args.explain {
        write_normalize_primers_explain(&setup)?;
    }

    ensure_normalize_primers_qa(catalog, platform, &setup.tools)?;

    let store = NormalizePrimersBenchmarkStore::from_setup(&setup);
    let conn = bijux_dna_analyze::open_sqlite(&store.sqlite_path)?;
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::new();

    for tool in &setup.tools {
        let tool_plan =
            prepare_normalize_primers_tool_plan(catalog, platform, args, &setup, jobs, tool)?;
        let cache_identity =
            NormalizePrimersCacheIdentity::from_plan(platform, &setup, tool, &tool_plan);
        if let Ok(Some(record)) = fetch_fastq_normalize_primers_v1(
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
        let tool_execution = execute_normalize_primers_tool(&tool_plan, setup.runner, jobs, tool)?;
        if tool_execution.result.exit_code != 0 {
            failures.push(normalize_primers_tool_failure(tool, tool_execution.result.exit_code));
            continue;
        }
        let outputs = resolve_normalize_primers_outputs(&tool_plan.plan)?;
        let observation = observe_normalize_primers_tool(
            catalog,
            platform,
            &setup,
            &tool_plan,
            &tool_execution,
            &outputs,
        )?;
        let measurements = project_normalize_primers_measurements(&setup, &observation);
        let report = build_normalize_primers_report(&NormalizePrimersReportInputs {
            args,
            setup: &setup,
            tool,
            tool_plan: &tool_plan,
            outputs: &outputs,
            observation: &observation,
            measurements: &measurements,
            tool_execution: &tool_execution,
        });
        let metric_set = build_normalize_primers_metric_set(&measurements, &report)?;
        write_normalize_primers_artifacts(&tool_plan, &outputs, &report, &metric_set)?;
        let record = BenchmarkRecord {
            context: build_benchmark_context(
                tool,
                tool_plan.tool_spec.tool_version.clone(),
                tool_plan.image_digest,
                setup.runner,
                platform,
                setup.input_hash.clone(),
                tool_plan.plan.params.clone(),
            ),
            execution: ExecutionMetrics {
                runtime_s: tool_execution.result.runtime_s,
                memory_mb: tool_execution.result.memory_mb,
                exit_code: tool_execution.result.exit_code,
            },
            metrics: metric_set,
        };
        record.validate()?;
        append_jsonl(&store.jsonl_path, &record)?;
        insert_fastq_normalize_primers_v1(&conn, &record)?;
        records.push(record);
    }

    Ok(BenchOutcome { records, failures, bench_dir: setup.bench_dir, explain: args.explain })
}

struct NormalizePrimersBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    runner: RuntimeKind,
    input_stats_r1: SeqkitMetrics,
    input_stats_r2: Option<SeqkitMetrics>,
    input_hash: String,
    bench_dir: PathBuf,
    tools_root: PathBuf,
    primer_governance: PrimerSetGovernance,
}

struct NormalizePrimersBenchmarkStore {
    sqlite_path: PathBuf,
    jsonl_path: PathBuf,
}

struct NormalizePrimersToolPlan {
    out_dir: PathBuf,
    tool_spec: ToolExecutionSpecV1,
    plan: StagePlanV1,
    params_hash: String,
    image_digest: String,
}

struct NormalizePrimersToolExecution {
    step: ExecutionStep,
    result: StageResultV1,
}

struct NormalizePrimersOutputs {
    output_r1: PathBuf,
    output_r2: Option<PathBuf>,
    report_json: PathBuf,
    orientation_report: PathBuf,
    primer_stats_json: PathBuf,
}

struct NormalizePrimersObservation {
    payload: serde_json::Value,
    output_stats_r1: SeqkitMetrics,
    output_stats_r2: Option<SeqkitMetrics>,
}

struct NormalizePrimersMeasurements {
    reads_in_total: u64,
    reads_out_total: u64,
    bases_in: u64,
    bases_out: u64,
    pairs_in: Option<u64>,
    pairs_out: Option<u64>,
    primer_trimmed_fraction: Option<f64>,
    orientation_forward_fraction: Option<f64>,
    primer_trimmed_reads: Option<u64>,
}

struct NormalizePrimersReportInputs<'a> {
    args: &'a bijux_dna_planner_fastq::stage_api::args::BenchFastqNormalizePrimersArgs,
    setup: &'a NormalizePrimersBenchmarkSetup,
    tool: &'a str,
    tool_plan: &'a NormalizePrimersToolPlan,
    outputs: &'a NormalizePrimersOutputs,
    observation: &'a NormalizePrimersObservation,
    measurements: &'a NormalizePrimersMeasurements,
    tool_execution: &'a NormalizePrimersToolExecution,
}

struct NormalizePrimersCacheIdentity<'a> {
    tool: &'a str,
    tool_version: String,
    image_digest: String,
    runner: String,
    platform: String,
    input_hash: String,
    params_hash: String,
}

impl<'a> NormalizePrimersCacheIdentity<'a> {
    fn from_plan(
        platform: &PlatformSpec,
        setup: &NormalizePrimersBenchmarkSetup,
        tool: &'a str,
        tool_plan: &NormalizePrimersToolPlan,
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

impl NormalizePrimersBenchmarkStore {
    fn from_setup(setup: &NormalizePrimersBenchmarkSetup) -> Self {
        Self {
            sqlite_path: setup.bench_dir.join("bench.sqlite"),
            jsonl_path: setup.bench_dir.join("bench.jsonl"),
        }
    }
}

fn prepare_normalize_primers_setup<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqNormalizePrimersArgs,
    selected_tools: &[String],
) -> Result<NormalizePrimersBenchmarkSetup> {
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
    let input_hash = normalize_primers_input_hash(args)?;
    let bench_dir_name =
        bench_dir_name(&bijux_dna_domain_fastq::stages::ids::STAGE_NORMALIZE_PRIMERS)
            .ok_or_else(|| anyhow!("bench dir missing for {STAGE_ID}"))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir)?;
    bijux_dna_infra::ensure_dir(&tools_root)?;
    let primer_governance = resolve_primer_set_governance(args.primer_set_id.as_deref())?;

    Ok(NormalizePrimersBenchmarkSetup {
        registry,
        tools,
        runner,
        input_stats_r1,
        input_stats_r2,
        input_hash,
        bench_dir,
        tools_root,
        primer_governance,
    })
}

fn normalize_primers_input_hash(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqNormalizePrimersArgs,
) -> Result<String> {
    if let Some(r2) = args.r2.as_deref() {
        return Ok(format!(
            "{}+{}",
            hash_file_sha256(&args.r1).context("hash normalize primers input r1")?,
            hash_file_sha256(r2).context("hash normalize primers input r2")?
        ));
    }
    hash_file_sha256(&args.r1).context("hash normalize primers input")
}

fn write_normalize_primers_explain(setup: &NormalizePrimersBenchmarkSetup) -> Result<()> {
    write_explain_md(&setup.bench_dir, STAGE_ID, &setup.tools, &[], None)?;
    write_explain_plan_json(&setup.bench_dir, STAGE_ID, &setup.tools, &setup.registry, None)
}

fn ensure_normalize_primers_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(STAGE_ID, tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_ID, tools, platform, catalog)
}

fn prepare_normalize_primers_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqNormalizePrimersArgs,
    setup: &NormalizePrimersBenchmarkSetup,
    jobs: usize,
    tool: &str,
) -> Result<NormalizePrimersToolPlan> {
    let out_dir = setup.tools_root.join(tool);
    bijux_dna_infra::ensure_dir(&out_dir)?;
    let tool_spec = build_tool_execution_spec(STAGE_ID, tool, &setup.registry, catalog, platform)?;
    let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
    let options = normalize_primers_plan_options(args, &setup.primer_governance);
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::normalize_primers::plan_with_options(
        &tool_spec,
        &args.r1,
        args.r2.as_deref(),
        &out_dir,
        &options,
    )?;
    let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
    let image_digest = tool_spec
        .image
        .digest
        .as_ref()
        .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
        .clone();
    Ok(NormalizePrimersToolPlan { out_dir, tool_spec, plan, params_hash, image_digest })
}

fn normalize_primers_plan_options(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqNormalizePrimersArgs,
    primer_governance: &PrimerSetGovernance,
) -> NormalizePrimersPlanOptions {
    NormalizePrimersPlanOptions {
        primer_set_id: primer_governance.primer_set_id.clone(),
        marker_id: Some(primer_governance.marker_id.clone()),
        primer_fasta: Some(primer_governance.primer_fasta.clone()),
        orientation_policy: args
            .orientation_policy
            .clone()
            .unwrap_or_else(|| "normalize_to_forward_primer".to_string()),
        max_mismatch_rate: args.max_mismatch_rate.unwrap_or(0.10),
        min_overlap_bp: args.min_overlap_bp.unwrap_or(10),
        strict_5p_anchor: args.strict_5p_anchor.unwrap_or(true),
        allow_iupac_codes: args.allow_iupac_codes.unwrap_or(true),
    }
}

fn execute_normalize_primers_tool(
    tool_plan: &NormalizePrimersToolPlan,
    runner: RuntimeKind,
    jobs: usize,
    tool: &str,
) -> Result<NormalizePrimersToolExecution> {
    let step = bijux_dna_stage_contract::execution_step_from_stage_plan(&tool_plan.plan);
    let result = execute_plans_with_jobs(vec![step.clone()], runner, jobs)?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing execution result for {tool}"))?;
    Ok(NormalizePrimersToolExecution { step, result })
}

fn normalize_primers_tool_failure(tool: &str, exit_code: i32) -> RawFailure {
    RawFailure {
        stage: STAGE_ID.to_string(),
        tool: tool.to_string(),
        reason: format!("tool {tool} failed with status {exit_code}"),
        category: ErrorCategory::ToolError,
    }
}

fn resolve_normalize_primers_outputs(plan: &StagePlanV1) -> Result<NormalizePrimersOutputs> {
    Ok(NormalizePrimersOutputs {
        output_r1: artifact_path(plan, "normalized_reads_r1")?,
        output_r2: artifact_path_optional(plan, "normalized_reads_r2"),
        report_json: artifact_path(plan, "report_json")?,
        orientation_report: artifact_path(plan, "primer_orientation_report")?,
        primer_stats_json: artifact_path(plan, "primer_stats_json")?,
    })
}

fn observe_normalize_primers_tool<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    setup: &NormalizePrimersBenchmarkSetup,
    tool_plan: &NormalizePrimersToolPlan,
    tool_execution: &NormalizePrimersToolExecution,
    outputs: &NormalizePrimersOutputs,
) -> Result<NormalizePrimersObservation> {
    let payload =
        materialize_amplicon_stage_outputs_for_bench(&tool_plan.out_dir, &tool_execution.step)?;
    enforce_amplicon_qc_thresholds_for_bench(&tool_plan.out_dir, STAGE_ID, &payload)?;
    let output_stats_r1 = observe_fastq_stats(catalog, platform, setup.runner, &outputs.output_r1)?;
    let output_stats_r2 = if let Some(output_r2) = outputs.output_r2.as_deref() {
        Some(observe_fastq_stats(catalog, platform, setup.runner, output_r2)?)
    } else {
        None
    };
    Ok(NormalizePrimersObservation { payload, output_stats_r1, output_stats_r2 })
}

fn project_normalize_primers_measurements(
    setup: &NormalizePrimersBenchmarkSetup,
    observation: &NormalizePrimersObservation,
) -> NormalizePrimersMeasurements {
    let primer_trimmed_fraction =
        observation.payload.get("primer_trimmed_fraction").and_then(serde_json::Value::as_f64);
    let orientation_forward_fraction =
        observation.payload.get("orientation_forward_fraction").and_then(serde_json::Value::as_f64);
    let reads_in_total =
        setup.input_stats_r1.reads + setup.input_stats_r2.as_ref().map_or(0, |stats| stats.reads);
    let reads_out_total = observation.output_stats_r1.reads
        + observation.output_stats_r2.as_ref().map_or(0, |stats| stats.reads);
    let bases_in =
        setup.input_stats_r1.bases + setup.input_stats_r2.as_ref().map_or(0, |stats| stats.bases);
    let bases_out = observation.output_stats_r1.bases
        + observation.output_stats_r2.as_ref().map_or(0, |stats| stats.bases);
    let pairs_in =
        setup.input_stats_r2.as_ref().map(|stats| setup.input_stats_r1.reads.min(stats.reads));
    let pairs_out = observation
        .output_stats_r2
        .as_ref()
        .map(|stats| observation.output_stats_r1.reads.min(stats.reads));
    let primer_trimmed_reads = primer_trimmed_fraction
        .and_then(|fraction| rounded_fraction_count(fraction, reads_in_total));
    NormalizePrimersMeasurements {
        reads_in_total,
        reads_out_total,
        bases_in,
        bases_out,
        pairs_in,
        pairs_out,
        primer_trimmed_fraction,
        orientation_forward_fraction,
        primer_trimmed_reads,
    }
}

fn build_normalize_primers_report(
    inputs: &NormalizePrimersReportInputs<'_>,
) -> NormalizePrimersReportV1 {
    NormalizePrimersReportV1 {
        schema_version: NORMALIZE_PRIMERS_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_ID.to_string(),
        stage_id: STAGE_ID.to_string(),
        tool_id: inputs.tool.to_string(),
        paired_mode: PairedMode::from_has_r2(inputs.args.r2.is_some()),
        primer_set_id: inputs.setup.primer_governance.primer_set_id.clone(),
        marker_id: Some(inputs.setup.primer_governance.marker_id.clone()),
        primer_fasta: Some(inputs.setup.primer_governance.primer_fasta.display().to_string()),
        orientation_policy: inputs
            .tool_plan
            .plan
            .effective_params
            .get("orientation_policy")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("normalize_to_forward_primer")
            .to_string(),
        max_mismatch_rate: inputs
            .tool_plan
            .plan
            .effective_params
            .get("max_mismatch_rate")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.10),
        min_overlap_bp: inputs
            .tool_plan
            .plan
            .effective_params
            .get("min_overlap_bp")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(10),
        input_r1: inputs.args.r1.display().to_string(),
        input_r2: inputs.args.r2.as_ref().map(|path| path.display().to_string()),
        output_r1: inputs.outputs.output_r1.display().to_string(),
        output_r2: inputs.outputs.output_r2.as_ref().map(|path| path.display().to_string()),
        reads_in: Some(inputs.measurements.reads_in_total),
        reads_out: Some(inputs.measurements.reads_out_total),
        bases_in: Some(inputs.measurements.bases_in),
        bases_out: Some(inputs.measurements.bases_out),
        pairs_in: inputs.measurements.pairs_in,
        pairs_out: inputs.measurements.pairs_out,
        primer_trimmed_reads: inputs.measurements.primer_trimmed_reads,
        primer_trimmed_fraction: inputs.measurements.primer_trimmed_fraction,
        orientation_forward_fraction: inputs.measurements.orientation_forward_fraction,
        primer_orientation_report: inputs.outputs.orientation_report.display().to_string(),
        primer_stats_json: inputs.outputs.primer_stats_json.display().to_string(),
        raw_backend_report: Some(inputs.outputs.primer_stats_json.display().to_string()),
        raw_backend_report_format: match inputs.tool {
            "cutadapt" => Some("cutadapt_json".to_string()),
            "seqkit" => Some("seqkit_grep".to_string()),
            _ => None,
        },
        runtime_s: Some(inputs.tool_execution.result.runtime_s),
        memory_mb: Some(inputs.tool_execution.result.memory_mb),
        used_fallback: inputs
            .observation
            .payload
            .get("used_fallback")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        backend_metrics: Some(inputs.observation.payload.clone()),
    }
}

fn build_normalize_primers_metric_set(
    measurements: &NormalizePrimersMeasurements,
    report: &NormalizePrimersReportV1,
) -> Result<MetricSet<FastqNormalizePrimersMetrics>> {
    let metrics = FastqNormalizePrimersMetrics {
        reads_in: measurements.reads_in_total,
        reads_out: measurements.reads_out_total,
        primer_trimmed_fraction: report.primer_trimmed_fraction.unwrap_or(0.0),
        orientation_forward_fraction: report.orientation_forward_fraction.unwrap_or(0.0),
    };
    let metric_set = metric_set(metrics);
    bijux_dna_analyze::validate_metric_set(&metric_set)?;
    Ok(metric_set)
}

fn write_normalize_primers_artifacts(
    tool_plan: &NormalizePrimersToolPlan,
    outputs: &NormalizePrimersOutputs,
    report: &NormalizePrimersReportV1,
    metric_set: &MetricSet<FastqNormalizePrimersMetrics>,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(&outputs.report_json, report)?;
    bijux_dna_infra::atomic_write_json(
        &tool_plan.out_dir.join("metrics.json"),
        &serde_json::to_value(metric_set)?,
    )?;
    Ok(())
}

fn select_normalize_primers_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqNormalizePrimersArgs,
) -> Result<Vec<String>> {
    let tools = bijux_dna_planner_fastq::select_normalize_primers_tools(&args.tools)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_ID, artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_ID, &header);
    Ok(tools)
}

fn artifact_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    artifact_name: &str,
) -> Result<std::path::PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_name)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| anyhow!("missing `{artifact_name}` output in normalize primers plan"))
}

fn artifact_path_optional(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    artifact_name: &str,
) -> Option<std::path::PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_name)
        .map(|artifact| artifact.path.clone())
}

fn rounded_fraction_count(fraction: f64, total: u64) -> Option<u64> {
    if !fraction.is_finite() || fraction <= 0.0 {
        return Some(0);
    }
    let rounded = (fraction * u64_to_f64(total)).round();
    if !rounded.is_finite() || rounded < 0.0 {
        return None;
    }
    format!("{rounded:.0}").parse::<u64>().ok()
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}
