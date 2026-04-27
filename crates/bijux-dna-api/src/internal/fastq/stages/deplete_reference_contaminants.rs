use std::collections::HashMap;

use crate::internal::fastq::stages::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context, observe_fastq_stats, prepare_trim_bench,
    TrimBenchInputs,
};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_deplete_reference_contaminants_v1, insert_fastq_deplete_reference_contaminants_v1,
};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkRecord, FastqDepleteReferenceContaminantsMetrics,
};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::measure::SeqkitMetrics;
use bijux_dna_core::prelude::params_hash;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::params::screen::ReferenceContaminantEffectiveParams;
use bijux_dna_domain_fastq::{
    stages::ids::STAGE_DEPLETE_REFERENCE_CONTAMINANTS, DepleteReferenceContaminantsReportV1,
    DEPLETE_REFERENCE_CONTAMINANTS_REPORT_SCHEMA_VERSION,
};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::select_deplete_reference_contaminants_tools;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_planner_fastq::tool_adapters::stages::transform::deplete_reference_contaminants::plan_contaminant_screen_with_options;
use bijux_dna_planner_fastq::DepleteReferenceContaminantsStageParams;
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use bijux_dna_stage_contract::StagePlanV1;

use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_deplete_reference_contaminants<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDepleteReferenceContaminantsArgs,
) -> Result<BenchOutcome<FastqDepleteReferenceContaminantsMetrics>> {
    let tools = select_reference_contaminants_benchmark_tools(args)?;
    let setup = prepare_reference_contaminants_benchmark_setup(
        catalog,
        platform,
        runner_override,
        args,
        &tools,
    )?;

    if args.explain {
        write_reference_contaminants_benchmark_explain(&setup)?;
    }

    let runner = setup.bench_inputs.runner;
    ensure_reference_contaminants_benchmark_qa(catalog, platform, &setup.tools)?;

    let sqlite_path = setup.bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = setup.bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::<RawFailure>::new();
    let mut records = Vec::<BenchmarkRecord<FastqDepleteReferenceContaminantsMetrics>>::new();

    for tool in setup.tools.clone() {
        let tool_plan =
            prepare_reference_contaminants_tool_plan(catalog, platform, args, &setup, jobs, tool)?;
        if let Ok(Some(record)) = fetch_fastq_deplete_reference_contaminants_v1(
            &conn,
            &tool_plan.tool,
            &tool_plan.tool_spec.tool_version,
            &tool_plan.image_digest,
            &runner.to_string(),
            &platform.name,
            &setup.input_hash,
            &tool_plan.params_hash,
        ) {
            records.push(record);
            continue;
        }

        let execution = execute_reference_contaminants_tool(&tool_plan, runner, jobs)?;
        if let Some(failure) = reference_contaminants_tool_failure(&tool_plan, execution.exit_code)
        {
            failures.push(failure);
            continue;
        }

        let report =
            build_deplete_reference_contaminants_report(&ReferenceContaminantsReportInputs {
                plan: &tool_plan.plan,
                input_stats_r1: &setup.bench_inputs.input_stats,
                input_stats_r2: setup.input_stats_r2.as_ref(),
                catalog,
                platform,
                runner,
                tool: &tool_plan.tool,
                execution: &execution,
            })?;
        bijux_dna_infra::atomic_write_json(std::path::Path::new(&report.report_json), &report)
            .context("write reference contaminant depletion report")?;
        let metrics = reference_contaminants_metrics_from_report(&report);
        let metric_set = metric_set(metrics.clone());
        bijux_dna_analyze::validate_metric_set(&metric_set)?;
        write_reference_contaminants_metrics(&tool_plan, &metric_set)?;

        let record = build_reference_contaminants_record(
            platform, &setup, &tool_plan, &execution, metric_set,
        )?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_deplete_reference_contaminants_v1(&conn, &record)
            .context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: setup.bench_inputs.bench_dir,
        explain: args.explain,
    })
}

fn select_reference_contaminants_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDepleteReferenceContaminantsArgs,
) -> Result<Vec<String>> {
    let tools = select_deplete_reference_contaminants_tools(&args.tools)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_DEPLETE_REFERENCE_CONTAMINANTS.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_DEPLETE_REFERENCE_CONTAMINANTS.as_str(), &header);
    Ok(tools)
}

struct ReferenceContaminantsBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    bench_inputs: TrimBenchInputs,
    input_hash: String,
    input_stats_r2: Option<SeqkitMetrics>,
}

struct ReferenceContaminantsToolPlan {
    tool: String,
    tool_spec: ToolExecutionSpecV1,
    plan: StagePlanV1,
    params_hash: String,
    image_digest: String,
}

struct ReferenceContaminantsReportInputs<'a, S: ::std::hash::BuildHasher> {
    plan: &'a StagePlanV1,
    input_stats_r1: &'a SeqkitMetrics,
    input_stats_r2: Option<&'a SeqkitMetrics>,
    catalog: &'a HashMap<String, ToolImageSpec, S>,
    platform: &'a PlatformSpec,
    runner: RuntimeKind,
    tool: &'a str,
    execution: &'a StageResultV1,
}

fn prepare_reference_contaminants_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDepleteReferenceContaminantsArgs,
    setup: &ReferenceContaminantsBenchmarkSetup,
    jobs: usize,
    tool: String,
) -> Result<ReferenceContaminantsToolPlan> {
    let out_dir = setup.bench_inputs.tools_root.join(&tool);
    bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
    let tool_spec = build_tool_execution_spec(
        STAGE_DEPLETE_REFERENCE_CONTAMINANTS.as_str(),
        &tool,
        &setup.registry,
        catalog,
        platform,
    )?;
    let mut tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
    if let Some(threads) = args.threads {
        tool_spec.resources.threads = threads.max(1);
    }
    let plan = plan_contaminant_screen_with_options(
        &tool_spec,
        &setup.bench_inputs.r1,
        args.r2.as_deref(),
        &args.reference_index,
        &out_dir,
        &DepleteReferenceContaminantsStageParams {
            decoy_mode: args.decoy_mode.clone().unwrap_or_else(|| "phix_and_spikeins".to_string()),
            threads: args.threads,
        },
    )?;
    let params_hash =
        params_hash(&plan.params).unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());
    let image_digest = benchmark_image_identity(&tool_spec);
    Ok(ReferenceContaminantsToolPlan { tool, tool_spec, plan, params_hash, image_digest })
}

fn execute_reference_contaminants_tool(
    tool_plan: &ReferenceContaminantsToolPlan,
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

fn reference_contaminants_tool_failure(
    tool_plan: &ReferenceContaminantsToolPlan,
    exit_code: i32,
) -> Option<RawFailure> {
    if exit_code == 0 {
        return None;
    }
    Some(RawFailure {
        stage: STAGE_DEPLETE_REFERENCE_CONTAMINANTS.as_str().to_string(),
        tool: tool_plan.tool.clone(),
        reason: format!("tool `{}` failed with status {exit_code}", tool_plan.tool),
        category: ErrorCategory::ToolError,
    })
}

fn build_reference_contaminants_record(
    platform: &PlatformSpec,
    setup: &ReferenceContaminantsBenchmarkSetup,
    tool_plan: &ReferenceContaminantsToolPlan,
    execution: &StageResultV1,
    metrics: bijux_dna_analyze::MetricSet<FastqDepleteReferenceContaminantsMetrics>,
) -> Result<BenchmarkRecord<FastqDepleteReferenceContaminantsMetrics>> {
    let context = build_benchmark_context(
        &tool_plan.tool,
        tool_plan.tool_spec.tool_version.clone(),
        tool_plan.image_digest.clone(),
        setup.bench_inputs.runner,
        platform,
        setup.input_hash.clone(),
        tool_plan.plan.params.clone(),
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

fn write_reference_contaminants_metrics(
    tool_plan: &ReferenceContaminantsToolPlan,
    metrics: &bijux_dna_analyze::MetricSet<FastqDepleteReferenceContaminantsMetrics>,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(
        &tool_plan.plan.out_dir.join("metrics.json"),
        &serde_json::to_value(metrics)?,
    )
    .context("write reference contaminant depletion metrics")
}

fn prepare_reference_contaminants_benchmark_setup<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDepleteReferenceContaminantsArgs,
    tools: &[String],
) -> Result<ReferenceContaminantsBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(
        STAGE_DEPLETE_REFERENCE_CONTAMINANTS.as_str(),
        tools,
        &registry,
        false,
    )?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_DEPLETE_REFERENCE_CONTAMINANTS,
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
    Ok(ReferenceContaminantsBenchmarkSetup {
        registry,
        tools,
        bench_inputs,
        input_hash,
        input_stats_r2,
    })
}

fn write_reference_contaminants_benchmark_explain(
    setup: &ReferenceContaminantsBenchmarkSetup,
) -> Result<()> {
    write_explain_md(
        &setup.bench_inputs.bench_dir,
        STAGE_DEPLETE_REFERENCE_CONTAMINANTS.as_str(),
        &setup.tools,
        &[],
        None,
    )?;
    write_explain_plan_json(
        &setup.bench_inputs.bench_dir,
        STAGE_DEPLETE_REFERENCE_CONTAMINANTS.as_str(),
        &setup.tools,
        &setup.registry,
        None,
    )
}

fn ensure_reference_contaminants_benchmark_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(
        STAGE_DEPLETE_REFERENCE_CONTAMINANTS.as_str(),
        tools,
        platform,
        catalog,
    )?;
    ensure_tool_qa_passed(STAGE_DEPLETE_REFERENCE_CONTAMINANTS.as_str(), tools, platform, catalog)
}

fn build_deplete_reference_contaminants_report<S: ::std::hash::BuildHasher>(
    inputs: &ReferenceContaminantsReportInputs<'_, S>,
) -> Result<DepleteReferenceContaminantsReportV1> {
    let effective_params: ReferenceContaminantEffectiveParams =
        serde_json::from_value(inputs.plan.effective_params.clone())
            .context("decode reference contaminant effective params")?;
    let output_r1 = artifact_output_path(inputs.plan, "contaminant_screened_reads_r1")
        .unwrap_or_else(|| inputs.plan.out_dir.join("contaminant_screened.fastq.gz"));
    let output_r2 = artifact_output_path(inputs.plan, "contaminant_screened_reads_r2");
    let report_json = artifact_output_path(inputs.plan, "contaminant_screen_report_json")
        .unwrap_or_else(|| inputs.plan.out_dir.join("contaminant_screen_report.json"));
    let output_stats_r1 =
        observe_fastq_stats(inputs.catalog, inputs.platform, inputs.runner, &output_r1)?;
    let output_stats_r2 = if let Some(path) = output_r2.as_deref() {
        Some(observe_fastq_stats(inputs.catalog, inputs.platform, inputs.runner, path)?)
    } else {
        None
    };
    let reads_in =
        inputs.input_stats_r1.reads + inputs.input_stats_r2.map_or(0, |stats| stats.reads);
    let reads_out = output_stats_r1.reads + output_stats_r2.map_or(0, |stats| stats.reads);
    let bases_in =
        inputs.input_stats_r1.bases + inputs.input_stats_r2.map_or(0, |stats| stats.bases);
    let bases_out = output_stats_r1.bases + output_stats_r2.map_or(0, |stats| stats.bases);
    let reads_removed = reads_in.saturating_sub(reads_out);
    let bases_removed = bases_in.saturating_sub(bases_out);
    let pairs_in = inputs.input_stats_r2.map(|stats| inputs.input_stats_r1.reads.min(stats.reads));
    let pairs_out = output_stats_r2.as_ref().map(|stats| output_stats_r1.reads.min(stats.reads));
    let contaminant_fraction_removed =
        if reads_in == 0 { 0.0 } else { u64_to_f64(reads_removed) / u64_to_f64(reads_in) };

    Ok(DepleteReferenceContaminantsReportV1 {
        schema_version: DEPLETE_REFERENCE_CONTAMINANTS_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_DEPLETE_REFERENCE_CONTAMINANTS.as_str().to_string(),
        stage_id: STAGE_DEPLETE_REFERENCE_CONTAMINANTS.as_str().to_string(),
        tool_id: inputs.tool.to_string(),
        paired_mode: effective_params.paired_mode,
        threads: effective_params.threads,
        reference_catalog_id: effective_params.reference_catalog_id,
        contaminant_reference: effective_params.contaminant_reference,
        index_artifact: effective_params.index_artifact,
        reference_index_backend: effective_params.reference_index_backend,
        reference_build_id: effective_params.reference_build_id,
        reference_digest: effective_params.reference_digest,
        retain_unmapped_pairs: effective_params.retain_unmapped_pairs,
        input_r1: artifact_input_path_string(inputs.plan, "reads_r1"),
        input_r2: artifact_input_path(inputs.plan, "reads_r2")
            .map(|path| path.display().to_string()),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        report_json: report_json.display().to_string(),
        reads_in,
        reads_out,
        reads_removed,
        bases_in,
        bases_out,
        bases_removed,
        pairs_in,
        pairs_out,
        contaminant_fraction_removed,
        runtime_s: Some(inputs.execution.runtime_s),
        memory_mb: Some(inputs.execution.memory_mb),
        exit_code: Some(inputs.execution.exit_code),
        raw_backend_report: inputs
            .plan
            .params
            .get("raw_backend_report")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        raw_backend_report_format: inputs
            .plan
            .params
            .get("raw_backend_report_format")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        backend_metrics: Some(serde_json::json!({
            "reads_removed": reads_removed,
            "bases_removed": bases_removed,
        })),
    })
}

fn reference_contaminants_metrics_from_report(
    report: &DepleteReferenceContaminantsReportV1,
) -> FastqDepleteReferenceContaminantsMetrics {
    FastqDepleteReferenceContaminantsMetrics {
        reads_in: report.reads_in,
        reads_out: report.reads_out,
        bases_in: report.bases_in,
        bases_out: report.bases_out,
        pairs_in: report.pairs_in.unwrap_or(0),
        pairs_out: report.pairs_out.unwrap_or(0),
        contaminant_fraction_removed: report.contaminant_fraction_removed.clamp(0.0, 1.0),
        depletion_summary: serde_json::json!({
            "reads_removed": report.reads_removed,
            "bases_removed": report.bases_removed,
            "output_r1": report.output_r1,
            "output_r2": report.output_r2,
            "report_json": report.report_json,
            "contaminant_reference": report.contaminant_reference,
            "reference_index_backend": report.reference_index_backend,
            "raw_backend_report": report.raw_backend_report,
            "raw_backend_report_format": report.raw_backend_report_format,
        })
        .into(),
    }
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

fn artifact_output_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    artifact_id: &str,
) -> Option<std::path::PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_id)
        .map(|artifact| artifact.path.clone())
}

fn artifact_input_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    artifact_id: &str,
) -> Option<std::path::PathBuf> {
    plan.io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_id)
        .map(|artifact| artifact.path.clone())
}

fn artifact_input_path_string(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    artifact_id: &str,
) -> String {
    artifact_input_path(plan, artifact_id)
        .map(|path| path.display().to_string())
        .unwrap_or_default()
}
