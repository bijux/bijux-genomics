use std::collections::HashMap;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_registry};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{fetch_fastq_umi_v1, insert_fastq_umi_v1};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqUmiMetrics};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::params_hash;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::select_umi_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::extract_umis::plan_umi;
use bijux_dna_planner_fastq::stage_api::FastqArtifact;
use bijux_dna_planner_fastq::stage_api::{
    ensure_umi_headers, inspect_headers, log_header_warnings, preflight_stage, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use uuid::Uuid;

use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_EXTRACT_UMIS,
};
use crate::internal::fastq::stages::trim_bench_common::{
    build_benchmark_context, observe_fastq_stats,
};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_umi<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqUmiArgs,
) -> Result<BenchOutcome<bijux_dna_analyze::FastqUmiMetrics>> {
    let tools = select_umi_tools(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage(STAGE_EXTRACT_UMIS.as_str(), artifact.kind)?;
    let r2 = args
        .r2
        .as_deref()
        .ok_or_else(|| anyhow!("umi stage requires paired-end input"))?;
    let header = inspect_headers(&args.r1, Some(r2), false)?;
    log_header_warnings(STAGE_EXTRACT_UMIS.as_str(), &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_EXTRACT_UMIS.as_str(), &tools, &registry, false)?;

    let bench_dir_name = bench_dir_name(&STAGE_EXTRACT_UMIS)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_EXTRACT_UMIS.as_str()))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_dna_infra::ensure_dir(&tools_root).context("create tools output dir")?;
    let input_hash = hash_file_sha256(&args.r1).context("hash umi input")?;

    if args.explain {
        write_explain_md(&bench_dir, STAGE_EXTRACT_UMIS.as_str(), &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, STAGE_EXTRACT_UMIS.as_str(), &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed(STAGE_EXTRACT_UMIS.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_EXTRACT_UMIS.as_str(), &tools, platform, catalog)?;

    ensure_umi_headers(&args.r1, args.r2.as_deref())?;

    let sqlite_path = bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let input_stats = observe_fastq_stats(catalog, platform, platform.runner, &args.r1)?;
    let mut records = Vec::<BenchmarkRecord<FastqUmiMetrics>>::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec =
            build_tool_execution_spec(STAGE_EXTRACT_UMIS.as_str(), tool, &registry, catalog, platform)?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = plan_umi(&tool_spec, &args.r1, r2, &out_dir)?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_umi_v1(
            &conn,
            tool,
            &tool_spec.tool_version,
            &image_digest,
            &platform.runner.to_string(),
            &platform.name,
            &input_hash,
            &params_hash,
        ) {
            records.push(record);
            continue;
        }
        let execution = execute_plans_with_jobs(
            vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&plan)],
            platform.runner,
            jobs,
        )?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing execution result for {tool}"))?;
        let record = build_umi_record(
            catalog,
            platform,
            &input_hash,
            &args.r1,
            &input_stats,
            tool,
            &tool_spec,
            &plan.params,
            &out_dir,
            &execution,
        )?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_umi_v1(&conn, &record).context("insert bench sqlite")?;
        if execution.exit_code != 0 {
            let tool_name = tool.clone();
            failures.push(RawFailure {
                stage: STAGE_EXTRACT_UMIS.as_str().to_string(),
                tool: tool.clone(),
                reason: format!(
                    "tool {tool_name} failed with status {}",
                    execution.exit_code
                ),
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

fn build_umi_record<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    input_hash: &str,
    r1: &std::path::Path,
    input_stats: &bijux_dna_core::prelude::measure::SeqkitMetrics,
    tool: &str,
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    params: &serde_json::Value,
    out_dir: &std::path::Path,
    execution: &StageResultV1,
) -> Result<BenchmarkRecord<FastqUmiMetrics>> {
    let output_r1 = out_dir.join("reads_r1.fastq.gz");
    let output_stats = if execution.exit_code == 0 && output_r1.exists() {
        observe_fastq_stats(catalog, platform, platform.runner, &output_r1)?
    } else {
        input_stats.clone()
    };
    let dedup_rate = if input_stats.reads == 0 {
        0.0
    } else {
        1.0 - (output_stats.reads as f64 / input_stats.reads as f64)
    };
    let metrics = FastqUmiMetrics {
        reads_in: input_stats.reads,
        reads_out: output_stats.reads,
        bases_in: input_stats.bases,
        bases_out: output_stats.bases,
        pairs_in: Some(input_stats.reads),
        pairs_out: Some(output_stats.reads),
        dedup_rate,
    };
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    let report = serde_json::json!({
        "schema_version": "bijux.fastq.extract_umis.report.v1",
        "stage_id": STAGE_EXTRACT_UMIS.as_str(),
        "tool_id": tool,
        "input_r1": r1,
        "output_r1": output_r1,
        "output_r2": out_dir.join("reads_r2.fastq.gz"),
        "reads_in": metrics.reads_in,
        "reads_out": metrics.reads_out,
        "bases_in": metrics.bases_in,
        "bases_out": metrics.bases_out,
        "dedup_rate": metrics.dedup_rate,
        "runtime_s": execution.runtime_s,
        "memory_mb": execution.memory_mb,
        "exit_code": execution.exit_code,
    });
    bijux_dna_infra::atomic_write_json(&out_dir.join("umi_report.json"), &report)
        .context("write umi report")?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write umi metrics")?;

    let context = build_benchmark_context(
        tool,
        tool_spec.tool_version.clone(),
        tool_spec
            .image
            .digest
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        platform.runner,
        platform,
        input_hash.to_string(),
        params.clone(),
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
    Ok(record)
}
