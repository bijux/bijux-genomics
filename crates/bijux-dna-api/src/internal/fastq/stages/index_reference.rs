use std::collections::HashMap;

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::internal::fastq::stages::trim_bench_common::benchmark_image_identity;
use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_INDEX_REFERENCE,
};
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_index_reference_v1, insert_fastq_index_reference_v1,
};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkContext, BenchmarkRecord, FastqIndexReferenceMetrics,
};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_domain_fastq::{
    IndexReferenceFileEntryV1, IndexReferenceReportV1, INDEX_REFERENCE_REPORT_SCHEMA_VERSION,
};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::select_index_reference_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::RawFailure;
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;

/// Benchmark FASTQ reference indexing tools under governed stage contracts.
///
/// # Errors
/// Returns an error if planning, execution, indexing, or persistence fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_index_reference<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqIndexReferenceArgs,
) -> Result<BenchOutcome<FastqIndexReferenceMetrics>> {
    let tools = select_index_reference_tools(&args.tools)?;
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_INDEX_REFERENCE.as_str(), &tools, &registry, false)?;

    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir_name = bench_dir_name(&STAGE_INDEX_REFERENCE)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_INDEX_REFERENCE.as_str()))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_dna_infra::ensure_dir(&tools_root).context("create tools output dir")?;
    let reference_fasta = args.reference_fasta.canonicalize().context("resolve reference fasta")?;
    let input_hash = hash_file_sha256(&reference_fasta).context("hash reference fasta")?;

    if args.explain {
        write_explain_md(&bench_dir, STAGE_INDEX_REFERENCE.as_str(), &tools, &[], None)?;
        write_explain_plan_json(
            &bench_dir,
            STAGE_INDEX_REFERENCE.as_str(),
            &tools,
            &registry,
            None,
        )?;
    }

    ensure_image_qa_passed(STAGE_INDEX_REFERENCE.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_INDEX_REFERENCE.as_str(), &tools, platform, catalog)?;

    let sqlite_path = bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::<BenchmarkRecord<FastqIndexReferenceMetrics>>::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec = build_tool_execution_spec(
            STAGE_INDEX_REFERENCE.as_str(),
            tool,
            &registry,
            catalog,
            platform,
        )?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan =
            bijux_dna_planner_fastq::tool_adapters::fastq::index_reference::plan_with_options(
                &tool_spec,
                &reference_fasta,
                &out_dir,
                &bijux_dna_planner_fastq::IndexReferenceStageParams { threads: args.threads },
            )?;
        let params_hash = stable_params_hash(&plan.params);
        let image_digest = benchmark_image_identity(&tool_spec);
        if let Ok(Some(record)) = fetch_fastq_index_reference_v1(
            &conn,
            tool,
            &tool_spec.tool_version,
            &image_digest,
            &runner.to_string(),
            &platform.name,
            &input_hash,
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
        let record = build_index_reference_record(
            platform,
            runner,
            &input_hash,
            &reference_fasta,
            tool,
            &tool_spec,
            &plan.params,
            &out_dir,
            &execution,
        )?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_index_reference_v1(&conn, &record).context("insert bench sqlite")?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: STAGE_INDEX_REFERENCE.as_str().to_string(),
                tool: tool.clone(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
                category: ErrorCategory::ToolError,
            });
        }
        records.push(record);
    }

    Ok(BenchOutcome { records, failures, bench_dir, explain: args.explain })
}

#[allow(clippy::too_many_arguments)]
fn build_index_reference_record(
    platform: &PlatformSpec,
    runner: RuntimeKind,
    input_hash: &str,
    reference_fasta: &std::path::Path,
    tool: &str,
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    params: &serde_json::Value,
    out_dir: &std::path::Path,
    execution: &bijux_dna_runner::step_runner::StageResultV1,
) -> Result<BenchmarkRecord<FastqIndexReferenceMetrics>> {
    let index_artifact = params
        .get("reference_index")
        .and_then(serde_json::Value::as_str)
        .map_or_else(|| out_dir.join("reference_index"), std::path::PathBuf::from);
    let report_json = params
        .get("report_json")
        .and_then(serde_json::Value::as_str)
        .map_or_else(|| out_dir.join("index_reference_report.json"), std::path::PathBuf::from);
    let index_root = out_dir.join("reference_index");
    let reference_bytes = std::fs::metadata(reference_fasta)
        .with_context(|| format!("stat {}", reference_fasta.display()))?
        .len();
    let emitted_files = collect_index_reference_files(&index_root);
    let report = canonical_index_reference_report(
        tool,
        params,
        reference_fasta,
        reference_bytes,
        &index_artifact,
        &report_json,
        emitted_files,
        execution,
    );
    let metrics = FastqIndexReferenceMetrics {
        reference_bytes: report.reference_bytes,
        index_bytes: report.index_bytes,
        index_file_count: report.index_file_count,
    };
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    bijux_dna_infra::atomic_write_json(&report_json, &report)
        .context("write index-reference report")?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write index-reference metrics")?;

    let context = BenchmarkContext {
        tool: tool.to_string(),
        tool_version: tool_spec.tool_version.clone(),
        image_digest: benchmark_image_identity(tool_spec),
        runner: runner.to_string(),
        platform: platform.name.clone(),
        input_hash: input_hash.to_string(),
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

fn collect_index_reference_files(index_root: &std::path::Path) -> Vec<IndexReferenceFileEntryV1> {
    let mut files = Vec::new();
    if !index_root.exists() {
        return files;
    }
    for entry in walkdir::WalkDir::new(index_root).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let relative_path = entry
            .path()
            .strip_prefix(index_root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();
        let bytes = entry.metadata().map(|meta| meta.len()).unwrap_or(0);
        files.push(IndexReferenceFileEntryV1 { relative_path, bytes });
    }
    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    files
}

#[allow(clippy::too_many_arguments)]
fn canonical_index_reference_report(
    tool_id: &str,
    params: &serde_json::Value,
    reference_fasta: &std::path::Path,
    reference_bytes: u64,
    reference_index: &std::path::Path,
    report_json: &std::path::Path,
    emitted_files: Vec<IndexReferenceFileEntryV1>,
    execution: &bijux_dna_runner::step_runner::StageResultV1,
) -> IndexReferenceReportV1 {
    let threads = params
        .get("threads")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(1);
    let index_prefix = match tool_id {
        "bowtie2_build" => Some("reference".to_string()),
        _ => None,
    };
    let index_bytes = emitted_files.iter().map(|entry| entry.bytes).sum::<u64>();
    let backend_metrics = serde_json::json!({
        "index_directory": reference_index
            .parent()
            .unwrap_or(reference_index)
            .display()
            .to_string(),
        "emitted_file_names": emitted_files
            .iter()
            .map(|entry| entry.relative_path.clone())
            .collect::<Vec<_>>(),
    });
    IndexReferenceReportV1 {
        schema_version: INDEX_REFERENCE_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_INDEX_REFERENCE.as_str().to_string(),
        stage_id: STAGE_INDEX_REFERENCE.as_str().to_string(),
        tool_id: tool_id.to_string(),
        threads,
        index_format: tool_id.to_string(),
        reference_fasta: reference_fasta.display().to_string(),
        reference_bytes,
        reference_index: reference_index.display().to_string(),
        report_json: report_json.display().to_string(),
        index_prefix,
        index_file_count: emitted_files.len() as u64,
        index_bytes,
        emitted_files,
        runtime_s: Some(execution.runtime_s),
        memory_mb: Some(execution.memory_mb),
        exit_code: Some(execution.exit_code),
        backend_metrics: Some(backend_metrics),
    }
}
