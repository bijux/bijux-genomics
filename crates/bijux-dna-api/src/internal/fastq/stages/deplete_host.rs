use std::collections::HashMap;

use crate::internal::fastq::stages::record_identity::stable_params_hash;
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
    fetch_fastq_deplete_host_v1, insert_fastq_deplete_host_v1,
};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqDepleteHostMetrics};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::measure::SeqkitMetrics;
use bijux_dna_core::prelude::params_hash;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::params::screen::HostDepletionEffectiveParams;
use bijux_dna_domain_fastq::stages::ids::STAGE_DEPLETE_HOST;
use bijux_dna_domain_fastq::{DepleteHostReportV1, PairedMode, DEPLETE_HOST_REPORT_SCHEMA_VERSION};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::hash_file_sha256;
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::select_deplete_host_tools;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_planner_fastq::tool_adapters::stages::transform::deplete_host::plan_host_depletion_with_options;
use bijux_dna_planner_fastq::DepleteHostStageParams;
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use bijux_dna_stage_contract::StagePlanV1;

use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};

fn artifact_output_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    name: &str,
) -> Option<std::path::PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == name)
        .map(|artifact| artifact.path.clone())
}

fn required_host_output_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    name: &str,
) -> Result<std::path::PathBuf> {
    artifact_output_path(plan, name)
        .ok_or_else(|| anyhow!("host depletion plan missing output artifact {name}"))
}

fn artifact_input_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    name: &str,
) -> Option<std::path::PathBuf> {
    plan.io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == name)
        .map(|artifact| artifact.path.clone())
}

fn artifact_input_path_string(plan: &bijux_dna_stage_contract::StagePlanV1, name: &str) -> String {
    artifact_input_path(plan, name).map(|path| path.display().to_string()).unwrap_or_default()
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_deplete_host<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDepleteHostArgs,
) -> Result<BenchOutcome<FastqDepleteHostMetrics>> {
    let tools = select_deplete_host_benchmark_tools(args)?;
    let setup =
        prepare_deplete_host_benchmark_setup(catalog, platform, runner_override, args, &tools)?;

    if args.explain {
        write_deplete_host_benchmark_explain(&setup)?;
    }

    let runner = setup.bench_inputs.runner;
    ensure_deplete_host_benchmark_qa(catalog, platform, &setup.tools)?;

    let sqlite_path = setup.bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = setup.bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::<RawFailure>::new();
    let mut records = Vec::<BenchmarkRecord<FastqDepleteHostMetrics>>::new();

    for tool in setup.tools.clone() {
        let tool_plan =
            prepare_deplete_host_tool_plan(catalog, platform, args, &setup, jobs, tool)?;
        if let Ok(Some(record)) = fetch_fastq_deplete_host_v1(
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

        let execution = execute_deplete_host_tool(&tool_plan, runner, jobs)?;
        if let Some(failure) = deplete_host_tool_failure(&tool_plan, execution.exit_code) {
            failures.push(failure);
            continue;
        }

        let report = build_deplete_host_report(&DepleteHostReportInputs {
            plan: &tool_plan.plan,
            input_stats_r1: &setup.bench_inputs.input_stats,
            input_stats_r2: setup.input_stats_r2.as_ref(),
            catalog,
            platform,
            runner,
            tool: &tool_plan.tool,
            execution: &execution,
        })?;
        validate_host_report_identity(&tool_plan.tool, &report)?;
        validate_host_report_execution(&report, &execution)?;
        validate_host_report_paired_mode(args.r2.is_some(), &report)?;
        write_deplete_host_report(&report)?;
        let metrics = deplete_host_metrics_from_report(&report);
        let metric_set = metric_set(metrics.clone());
        bijux_dna_analyze::validate_metric_set(&metric_set)?;
        write_deplete_host_metrics(&tool_plan, &metric_set)?;

        let record =
            build_deplete_host_record(platform, &setup, &tool_plan, &execution, metric_set)?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_deplete_host_v1(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: setup.bench_inputs.bench_dir,
        explain: args.explain,
    })
}

fn select_deplete_host_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDepleteHostArgs,
) -> Result<Vec<String>> {
    let tools = select_deplete_host_tools(&args.tools)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_DEPLETE_HOST.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_DEPLETE_HOST.as_str(), &header);
    Ok(tools)
}

struct DepleteHostBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    bench_inputs: TrimBenchInputs,
    input_hash: String,
    input_stats_r2: Option<SeqkitMetrics>,
}

struct DepleteHostToolPlan {
    tool: String,
    tool_spec: ToolExecutionSpecV1,
    plan: StagePlanV1,
    params_hash: String,
    image_digest: String,
}

struct DepleteHostReportInputs<'a, S: ::std::hash::BuildHasher> {
    plan: &'a StagePlanV1,
    input_stats_r1: &'a SeqkitMetrics,
    input_stats_r2: Option<&'a SeqkitMetrics>,
    catalog: &'a HashMap<String, ToolImageSpec, S>,
    platform: &'a PlatformSpec,
    runner: RuntimeKind,
    tool: &'a str,
    execution: &'a StageResultV1,
}

fn prepare_deplete_host_benchmark_setup<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDepleteHostArgs,
    tools: &[String],
) -> Result<DepleteHostBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_DEPLETE_HOST.as_str(), tools, &registry, false)?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_DEPLETE_HOST,
    )?;
    let input_hash = deplete_host_input_hash(&bench_inputs, args)?;
    let input_stats_r2 = if let Some(r2) = args.r2.as_ref() {
        Some(observe_fastq_stats(catalog, platform, bench_inputs.runner, r2)?)
    } else {
        None
    };
    Ok(DepleteHostBenchmarkSetup { registry, tools, bench_inputs, input_hash, input_stats_r2 })
}

fn deplete_host_input_hash(
    bench_inputs: &TrimBenchInputs,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDepleteHostArgs,
) -> Result<String> {
    if let Some(r2) = args.r2.as_ref() {
        let r2_hash = hash_file_sha256(r2).context("hash host depletion input r2")?;
        return params_hash(&serde_json::json!({
            "r1": bench_inputs.input_hash,
            "r2": r2_hash,
        }))
        .context("combine host depletion paired input hashes");
    }
    Ok(bench_inputs.input_hash.clone())
}

fn prepare_deplete_host_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDepleteHostArgs,
    setup: &DepleteHostBenchmarkSetup,
    jobs: usize,
    tool: String,
) -> Result<DepleteHostToolPlan> {
    let out_dir = setup.bench_inputs.tools_root.join(&tool);
    bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
    let tool_spec = build_tool_execution_spec(
        STAGE_DEPLETE_HOST.as_str(),
        &tool,
        &setup.registry,
        catalog,
        platform,
    )?;
    let mut tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
    if let Some(threads) = args.threads {
        tool_spec.resources.threads = threads.max(1);
    }
    let plan = plan_host_depletion_with_options(
        &tool_spec,
        &setup.bench_inputs.r1,
        args.r2.as_deref(),
        &args.reference_index,
        &out_dir,
        &DepleteHostStageParams {
            threads: args.threads,
            host_identity_threshold: args.host_identity_threshold.unwrap_or(0.95),
            retain_unmapped_only: args.retain_unmapped_only.unwrap_or(true),
        },
    )?;
    let params_hash = stable_params_hash(&plan.params);
    let image_digest = benchmark_image_identity(&tool_spec);
    Ok(DepleteHostToolPlan { tool, tool_spec, plan, params_hash, image_digest })
}

fn execute_deplete_host_tool(
    tool_plan: &DepleteHostToolPlan,
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

fn deplete_host_tool_failure(
    tool_plan: &DepleteHostToolPlan,
    exit_code: i32,
) -> Option<RawFailure> {
    if exit_code == 0 {
        return None;
    }
    Some(RawFailure {
        stage: STAGE_DEPLETE_HOST.as_str().to_string(),
        tool: tool_plan.tool.clone(),
        reason: format!("tool `{}` failed with status {exit_code}", tool_plan.tool),
        category: ErrorCategory::ToolError,
    })
}

fn write_deplete_host_benchmark_explain(setup: &DepleteHostBenchmarkSetup) -> Result<()> {
    write_explain_md(
        &setup.bench_inputs.bench_dir,
        STAGE_DEPLETE_HOST.as_str(),
        &setup.tools,
        &[],
        None,
    )?;
    write_explain_plan_json(
        &setup.bench_inputs.bench_dir,
        STAGE_DEPLETE_HOST.as_str(),
        &setup.tools,
        &setup.registry,
        None,
    )
}

fn ensure_deplete_host_benchmark_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(STAGE_DEPLETE_HOST.as_str(), tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_DEPLETE_HOST.as_str(), tools, platform, catalog)
}

fn build_deplete_host_report<S: ::std::hash::BuildHasher>(
    inputs: &DepleteHostReportInputs<'_, S>,
) -> Result<DepleteHostReportV1> {
    let effective_params: HostDepletionEffectiveParams =
        serde_json::from_value(inputs.plan.effective_params.clone())
            .context("decode host depletion effective params")?;
    let output_r1 = required_host_output_path(inputs.plan, "host_depleted_reads_r1")?;
    let output_r2 = artifact_output_path(inputs.plan, "host_depleted_reads_r2");
    let removed_host_r1 = required_host_output_path(inputs.plan, "removed_host_reads_r1")?;
    let removed_host_r2 = artifact_output_path(inputs.plan, "removed_host_reads_r2");
    let report_json = required_host_output_path(inputs.plan, "host_depletion_report_json")?;
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
    let host_fraction_removed =
        if reads_in == 0 { 0.0 } else { u64_to_f64(reads_removed) / u64_to_f64(reads_in) };

    Ok(DepleteHostReportV1 {
        schema_version: DEPLETE_HOST_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_DEPLETE_HOST.as_str().to_string(),
        stage_id: STAGE_DEPLETE_HOST.as_str().to_string(),
        tool_id: inputs.tool.to_string(),
        paired_mode: effective_params.paired_mode,
        threads: effective_params.threads,
        reference_scope: effective_params.reference_scope,
        reference_catalog_id: effective_params.reference_catalog_id,
        reference_index_artifact_id: effective_params.reference_index_artifact_id,
        reference_index_backend: effective_params.reference_index_backend,
        reference_build_id: effective_params.reference_build_id,
        reference_digest: effective_params.reference_digest,
        masking_policy: effective_params.masking_policy,
        decoy_policy: effective_params.decoy_policy,
        decoy_catalog_id: effective_params.decoy_catalog_id,
        identity_threshold: effective_params.identity_threshold,
        retained_read_policy: effective_params.retained_read_policy,
        emit_removed_reads: effective_params.emit_removed_reads,
        report_format: effective_params.report_format,
        retain_unmapped_pairs: effective_params.retain_unmapped_pairs,
        input_r1: artifact_input_path_string(inputs.plan, "reads_r1"),
        input_r2: artifact_input_path(inputs.plan, "reads_r2")
            .map(|path| path.display().to_string()),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        removed_host_r1: removed_host_r1.display().to_string(),
        removed_host_r2: removed_host_r2.map(|path| path.display().to_string()),
        report_json: report_json.display().to_string(),
        reads_in,
        reads_out,
        reads_removed,
        bases_in,
        bases_out,
        bases_removed,
        pairs_in,
        pairs_out,
        host_fraction_removed,
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

fn deplete_host_metrics_from_report(report: &DepleteHostReportV1) -> FastqDepleteHostMetrics {
    FastqDepleteHostMetrics {
        reads_in: report.reads_in,
        reads_out: report.reads_out,
        bases_in: report.bases_in,
        bases_out: report.bases_out,
        pairs_in: report.pairs_in.unwrap_or(0),
        pairs_out: report.pairs_out.unwrap_or(0),
        host_fraction_removed: report.host_fraction_removed.clamp(0.0, 1.0),
        depletion_summary: serde_json::json!({
            "reads_removed": report.reads_removed,
            "bases_removed": report.bases_removed,
            "output_r1": report.output_r1,
            "output_r2": report.output_r2,
            "removed_host_r1": report.removed_host_r1,
            "removed_host_r2": report.removed_host_r2,
            "report_json": report.report_json,
            "reference_catalog_id": report.reference_catalog_id,
            "reference_index_backend": report.reference_index_backend,
            "raw_backend_report": report.raw_backend_report,
            "raw_backend_report_format": report.raw_backend_report_format,
        })
        .into(),
    }
}

fn validate_host_report_identity(tool: &str, report: &DepleteHostReportV1) -> Result<()> {
    if report.schema_version != DEPLETE_HOST_REPORT_SCHEMA_VERSION {
        return Err(anyhow!(
            "host depletion report schema mismatch: expected {}, observed {}",
            DEPLETE_HOST_REPORT_SCHEMA_VERSION,
            report.schema_version
        ));
    }
    if report.stage != STAGE_DEPLETE_HOST.as_str() || report.stage_id != STAGE_DEPLETE_HOST.as_str()
    {
        return Err(anyhow!(
            "host depletion report stage mismatch: observed stage={} stage_id={}",
            report.stage,
            report.stage_id
        ));
    }
    if report.tool_id != tool {
        return Err(anyhow!(
            "host depletion report tool mismatch: expected {}, observed {}",
            tool,
            report.tool_id
        ));
    }
    Ok(())
}

fn validate_host_report_execution(
    report: &DepleteHostReportV1,
    execution: &StageResultV1,
) -> Result<()> {
    if report.runtime_s.is_none_or(|observed| (observed - execution.runtime_s).abs() > f64::EPSILON)
    {
        return Err(anyhow!(
            "host depletion report runtime mismatch: expected {}, observed {:?}",
            execution.runtime_s,
            report.runtime_s
        ));
    }
    if report.memory_mb.is_none_or(|observed| (observed - execution.memory_mb).abs() > f64::EPSILON)
    {
        return Err(anyhow!(
            "host depletion report memory mismatch: expected {}, observed {:?}",
            execution.memory_mb,
            report.memory_mb
        ));
    }
    if report.exit_code != Some(execution.exit_code) {
        return Err(anyhow!(
            "host depletion report exit code mismatch: expected {}, observed {:?}",
            execution.exit_code,
            report.exit_code
        ));
    }
    Ok(())
}

fn validate_host_report_paired_mode(has_r2: bool, report: &DepleteHostReportV1) -> Result<()> {
    let expected = if has_r2 { PairedMode::PairedEnd } else { PairedMode::SingleEnd };
    if report.paired_mode != expected {
        return Err(anyhow!(
            "host depletion report paired mode mismatch: expected {:?}, observed {:?}",
            expected,
            report.paired_mode
        ));
    }
    Ok(())
}

fn write_deplete_host_report(report: &DepleteHostReportV1) -> Result<()> {
    bijux_dna_infra::atomic_write_json(std::path::Path::new(&report.report_json), report)
        .context("write host depletion report")
}

fn write_deplete_host_metrics(
    tool_plan: &DepleteHostToolPlan,
    metrics: &bijux_dna_analyze::MetricSet<FastqDepleteHostMetrics>,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(
        &tool_plan.plan.out_dir.join("metrics.json"),
        &serde_json::to_value(metrics)?,
    )
    .context("write host depletion metrics")
}

fn build_deplete_host_record(
    platform: &PlatformSpec,
    setup: &DepleteHostBenchmarkSetup,
    tool_plan: &DepleteHostToolPlan,
    execution: &StageResultV1,
    metrics: bijux_dna_analyze::MetricSet<FastqDepleteHostMetrics>,
) -> Result<BenchmarkRecord<FastqDepleteHostMetrics>> {
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

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}
