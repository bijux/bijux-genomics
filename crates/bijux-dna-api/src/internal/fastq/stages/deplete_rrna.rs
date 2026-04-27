use std::collections::HashMap;

use crate::internal::fastq::stages::trim_bench_common::{
    build_benchmark_context, observe_fastq_stats, prepare_trim_bench, TrimBenchInputs,
};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_deplete_rrna_v1, insert_fastq_deplete_rrna_v1,
};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqDepleteRrnaMetrics};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::measure::SeqkitMetrics;
use bijux_dna_core::prelude::params_hash;
use bijux_dna_domain_fastq::params::screen::RrnaEffectiveParams;
use bijux_dna_domain_fastq::{
    DepleteRrnaReportV1, DEPLETE_RRNA_REPORT_SCHEMA_VERSION, STAGE_DEPLETE_RRNA,
};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::select_deplete_rrna_tools;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_planner_fastq::tool_adapters::stages::qc::deplete_rrna::plan_rrna_with_options;
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;

use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};

/// # Errors
/// Returns an error if planning or execution fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_deplete_rrna<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDepleteRrnaArgs,
) -> Result<BenchOutcome<FastqDepleteRrnaMetrics>> {
    let tools = select_rrna_benchmark_tools(args)?;
    let setup = prepare_rrna_benchmark_setup(catalog, platform, runner_override, args, &tools)?;

    if args.explain {
        write_explain_md(
            &setup.bench_inputs.bench_dir,
            STAGE_DEPLETE_RRNA.as_str(),
            &setup.tools,
            &[],
            None,
        )?;
        write_explain_plan_json(
            &setup.bench_inputs.bench_dir,
            STAGE_DEPLETE_RRNA.as_str(),
            &setup.tools,
            &setup.registry,
            None,
        )?;
    }

    let runner = setup.bench_inputs.runner;
    ensure_image_qa_passed(STAGE_DEPLETE_RRNA.as_str(), &setup.tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_DEPLETE_RRNA.as_str(), &setup.tools, platform, catalog)?;

    let sqlite_path = setup.bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = setup.bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::<RawFailure>::new();
    let mut records = Vec::<BenchmarkRecord<FastqDepleteRrnaMetrics>>::new();

    for tool in setup.tools.clone() {
        let out_dir = setup.bench_inputs.tools_root.join(&tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec = build_tool_execution_spec(
            STAGE_DEPLETE_RRNA.as_str(),
            &tool,
            &setup.registry,
            catalog,
            platform,
        )?;
        let mut tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        if let Some(threads) = args.threads {
            tool_spec.resources.threads = threads.max(1);
        }
        let plan = plan_rrna_with_options(
            &tool_spec,
            &setup.bench_inputs.r1,
            args.r2.as_deref(),
            &out_dir,
            &bijux_dna_planner_fastq::DepleteRrnaStageParams {
                rrna_db: args.rrna_db.clone().unwrap_or_else(|| "rrna_reference".to_string()),
                min_identity: args.min_identity.unwrap_or(0.95),
                threads: args.threads,
            },
        )?;
        let params_hash =
            params_hash(&plan.params).unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_deplete_rrna_v1(
            &conn,
            &tool,
            &tool_spec.tool_version,
            &image_digest,
            &runner.to_string(),
            &platform.name,
            &setup.input_hash,
            &params_hash,
        ) {
            records.push(record);
            continue;
        }

        let execution = execute_plans_with_jobs(
            vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&plan)],
            runner,
            jobs,
        )?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing execution result for {tool}"))?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: STAGE_DEPLETE_RRNA.as_str().to_string(),
                tool: tool.clone(),
                reason: format!("tool `{tool}` failed with status {}", execution.exit_code),
                category: ErrorCategory::ToolError,
            });
            continue;
        }

        let report = build_rrna_report(
            &plan,
            &setup.bench_inputs.input_stats,
            setup.input_stats_r2.as_ref(),
            catalog,
            platform,
            runner,
            &tool,
            &execution,
        )?;
        bijux_dna_infra::atomic_write_json(std::path::Path::new(&report.rrna_report_json), &report)
            .context("write rrna depletion report")?;
        let metrics = FastqDepleteRrnaMetrics {
            reads_in: report.reads_in,
            reads_out: report.reads_out,
            bases_in: report.bases_in,
            bases_out: report.bases_out,
            pairs_in: report.pairs_in.unwrap_or(0),
            pairs_out: report.pairs_out.unwrap_or(0),
            rrna_fraction_removed: report.rrna_fraction_removed.clamp(0.0, 1.0),
            depletion_summary: serde_json::json!({
                "reads_removed": report.reads_removed,
                "bases_removed": report.bases_removed,
                "output_r1": report.output_r1,
                "output_r2": report.output_r2,
                "report_tsv": report.rrna_report_tsv,
                "report_json": report.rrna_report_json,
                "database_artifact_id": report.database_artifact_id,
                "screening_engine": report.screening_engine,
            })
            .into(),
        };
        let metric_set = metric_set(metrics.clone());
        bijux_dna_analyze::validate_metric_set(&metric_set)?;
        bijux_dna_infra::atomic_write_json(
            &out_dir.join("metrics.json"),
            &serde_json::to_value(&metric_set)?,
        )
        .context("write rrna depletion metrics")?;

        let context = build_benchmark_context(
            &tool,
            tool_spec.tool_version.clone(),
            image_digest,
            runner,
            platform,
            setup.input_hash.clone(),
            plan.params.clone(),
        );
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
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_deplete_rrna_v1(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: setup.bench_inputs.bench_dir,
        explain: args.explain,
    })
}

fn select_rrna_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDepleteRrnaArgs,
) -> Result<Vec<String>> {
    let tools = select_deplete_rrna_tools(&args.tools)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_DEPLETE_RRNA.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_DEPLETE_RRNA.as_str(), &header);
    Ok(tools)
}

struct RrnaBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    bench_inputs: TrimBenchInputs,
    input_hash: String,
    input_stats_r2: Option<SeqkitMetrics>,
}

fn prepare_rrna_benchmark_setup<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDepleteRrnaArgs,
    tools: &[String],
) -> Result<RrnaBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_DEPLETE_RRNA.as_str(), tools, &registry, false)?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_DEPLETE_RRNA,
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
    Ok(RrnaBenchmarkSetup { registry, tools, bench_inputs, input_hash, input_stats_r2 })
}

#[allow(clippy::too_many_arguments)]
fn build_rrna_report<S: ::std::hash::BuildHasher>(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    input_stats_r1: &bijux_dna_core::prelude::measure::SeqkitMetrics,
    input_stats_r2: Option<&bijux_dna_core::prelude::measure::SeqkitMetrics>,
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner: RuntimeKind,
    tool: &str,
    execution: &bijux_dna_runner::step_runner::StageResultV1,
) -> Result<DepleteRrnaReportV1> {
    let effective_params: RrnaEffectiveParams =
        serde_json::from_value(plan.effective_params.clone())
            .context("decode rrna effective params")?;
    let output_r1 = artifact_output_path(plan, "rrna_filtered_reads_r1")
        .unwrap_or_else(|| plan.out_dir.join("rrna_filtered.fastq.gz"));
    let output_r2 = artifact_output_path(plan, "rrna_filtered_reads_r2");
    let rrna_report_tsv = artifact_output_path(plan, "rrna_report_tsv")
        .unwrap_or_else(|| plan.out_dir.join("rrna_report.tsv"));
    let rrna_report_json = artifact_output_path(plan, "rrna_report_json")
        .unwrap_or_else(|| plan.out_dir.join("rrna_report.json"));
    let output_stats_r1 = observe_fastq_stats(catalog, platform, runner, &output_r1)?;
    let output_stats_r2 = if let Some(path) = output_r2.as_deref() {
        Some(observe_fastq_stats(catalog, platform, runner, path)?)
    } else {
        None
    };
    let reads_in = input_stats_r1.reads + input_stats_r2.map_or(0, |stats| stats.reads);
    let reads_out = output_stats_r1.reads + output_stats_r2.map_or(0, |stats| stats.reads);
    let bases_in = input_stats_r1.bases + input_stats_r2.map_or(0, |stats| stats.bases);
    let bases_out = output_stats_r1.bases + output_stats_r2.map_or(0, |stats| stats.bases);
    let reads_removed = reads_in.saturating_sub(reads_out);
    let bases_removed = bases_in.saturating_sub(bases_out);
    let pairs_in = input_stats_r2.map(|stats| input_stats_r1.reads.min(stats.reads));
    let pairs_out = output_stats_r2.as_ref().map(|stats| output_stats_r1.reads.min(stats.reads));
    let rrna_fraction_removed =
        if reads_in == 0 { 0.0 } else { u64_to_f64(reads_removed) / u64_to_f64(reads_in) };

    Ok(DepleteRrnaReportV1 {
        schema_version: DEPLETE_RRNA_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_DEPLETE_RRNA.as_str().to_string(),
        stage_id: STAGE_DEPLETE_RRNA.as_str().to_string(),
        tool_id: tool.to_string(),
        paired_mode: effective_params.paired_mode,
        threads: effective_params.threads,
        rrna_db: effective_params.contaminant_db,
        database_artifact_id: effective_params.database_artifact_id,
        database_build_id: effective_params.database_build_id,
        screening_engine: effective_params.screening_engine,
        report_format: effective_params.report_format,
        emit_removed_reads: effective_params.emit_removed_reads,
        min_identity: plan.params.get("min_identity").and_then(serde_json::Value::as_f64),
        input_r1: artifact_input_path_string(plan, "reads_r1"),
        input_r2: artifact_input_path(plan, "reads_r2").map(|path| path.display().to_string()),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        rrna_report_tsv: rrna_report_tsv.display().to_string(),
        rrna_report_json: rrna_report_json.display().to_string(),
        reads_in,
        reads_out,
        reads_removed,
        bases_in,
        bases_out,
        bases_removed,
        pairs_in,
        pairs_out,
        rrna_fraction_removed,
        runtime_s: Some(execution.runtime_s),
        memory_mb: Some(execution.memory_mb),
        exit_code: Some(execution.exit_code),
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: Some(serde_json::json!({
            "reads_removed": reads_removed,
            "bases_removed": bases_removed,
        })),
    })
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
