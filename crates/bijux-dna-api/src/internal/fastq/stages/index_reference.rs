use std::collections::HashMap;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_workspace_registry};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_index_reference_v1, insert_fastq_index_reference_v1,
};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkContext, BenchmarkRecord, FastqIndexReferenceMetrics,
};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::params_hash;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::select_index_reference_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::index_reference::plan;
use bijux_dna_planner_fastq::stage_api::RawFailure;
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use uuid::Uuid;

use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_INDEX_REFERENCE,
};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;

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
    let reference_fasta = args
        .reference_fasta
        .canonicalize()
        .context("resolve reference fasta")?;
    let input_hash = hash_file_sha256(&reference_fasta).context("hash reference fasta")?;

    if args.explain {
        write_explain_md(
            &bench_dir,
            STAGE_INDEX_REFERENCE.as_str(),
            &tools,
            &[],
            None,
        )?;
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
        let plan = plan(&tool_spec, &reference_fasta, &out_dir)?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
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
            vec![bijux_dna_stage_contract::execution_step_from_stage_plan(
                &plan,
            )],
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

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir,
        explain: args.explain,
    })
}

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
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| out_dir.join("reference_index"));
    let index_root = out_dir.join("reference_index");
    let mut index_bytes = 0_u64;
    let mut index_file_count = 0_u64;
    if index_root.exists() {
        for entry in walkdir::WalkDir::new(&index_root).into_iter().flatten() {
            if entry.file_type().is_file() {
                index_file_count += 1;
                index_bytes += entry.metadata().map(|meta| meta.len()).unwrap_or(0);
            }
        }
    }
    let metrics = FastqIndexReferenceMetrics {
        reference_bytes: std::fs::metadata(reference_fasta)
            .with_context(|| format!("stat {}", reference_fasta.display()))?
            .len(),
        index_bytes,
        index_file_count,
    };
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    let report = serde_json::json!({
        "schema_version": "bijux.fastq.index_reference.report.v1",
        "stage_id": STAGE_INDEX_REFERENCE.as_str(),
        "tool_id": tool,
        "reference_fasta": reference_fasta,
        "reference_index": index_artifact,
        "reference_bytes": metrics.reference_bytes,
        "index_bytes": metrics.index_bytes,
        "index_file_count": metrics.index_file_count,
        "runtime_s": execution.runtime_s,
        "memory_mb": execution.memory_mb,
        "exit_code": execution.exit_code,
    });
    bijux_dna_infra::atomic_write_json(&out_dir.join("index_reference_report.json"), &report)
        .context("write index-reference report")?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write index-reference metrics")?;

    let context = BenchmarkContext {
        tool: tool.to_string(),
        tool_version: tool_spec.tool_version.clone(),
        image_digest: tool_spec
            .image
            .digest
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
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
