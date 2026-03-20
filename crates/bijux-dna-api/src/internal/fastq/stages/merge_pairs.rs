use std::collections::HashMap;
use std::path::Path;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_workspace_registry};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::quality::{fetch_fastq_merge_v1, insert_fastq_merge_v1};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqMergeMetrics};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::params_hash;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, ensure_dir, hash_file_sha256};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::select_merge_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::merge_pairs::plan_merge;
use bijux_dna_planner_fastq::stage_api::FastqArtifactKind;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use uuid::Uuid;

use crate::internal::fastq::stages::trim_bench_common::{build_benchmark_context, observe_fastq_stats};
use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_MERGE_PAIRS,
};

pub fn bench_fastq_merge<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqMergeArgs,
) -> Result<BenchOutcome<FastqMergeMetrics>> {
    let tools = select_merge_tools(&args.tools)?;
    preflight_stage(STAGE_MERGE_PAIRS.as_str(), FastqArtifactKind::PairedEnd)?;
    let header = inspect_headers(&args.r1, Some(&args.r2), false)?;
    log_header_warnings(STAGE_MERGE_PAIRS.as_str(), &header);

    let registry = load_workspace_registry()
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_MERGE_PAIRS.as_str(), &tools, &registry, false)?;

    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir_name = bench_dir_name(&STAGE_MERGE_PAIRS)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_MERGE_PAIRS.as_str()))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    ensure_dir(&bench_dir).context("create bench output dir")?;
    ensure_dir(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, STAGE_MERGE_PAIRS.as_str(), &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, STAGE_MERGE_PAIRS.as_str(), &tools, &registry, None)?;
    }

    ensure_image_qa_passed(STAGE_MERGE_PAIRS.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_MERGE_PAIRS.as_str(), &tools, platform, catalog)?;

    let input_hash = merge_input_hash(&args.r1, &args.r2)?;
    let r1_stats = observe_fastq_stats(catalog, platform, runner, &args.r1)?;
    let r2_stats = observe_fastq_stats(catalog, platform, runner, &args.r2)?;
    let sqlite_path = bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::<BenchmarkRecord<FastqMergeMetrics>>::new();

    for tool in &tools {
        let out_dir = tools_root.join(tool);
        ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec =
            build_tool_execution_spec(STAGE_MERGE_PAIRS.as_str(), tool, &registry, catalog, platform)?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = plan_merge(&tool_spec, &args.r1, &args.r2, &out_dir)?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_merge_v1(
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
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: STAGE_MERGE_PAIRS.as_str().to_string(),
                tool: tool.clone(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
                category: ErrorCategory::ToolError,
            });
            continue;
        }
        let record = build_merge_record(
            catalog,
            platform,
            runner,
            &input_hash,
            &args.r1,
            &args.r2,
            &r1_stats,
            &r2_stats,
            tool,
            &tool_spec,
            &plan.params,
            &plan.io.outputs[0].path,
            &execution,
        )?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_merge_v1(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir,
        explain: args.explain,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_merge_record<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner: RuntimeKind,
    input_hash: &str,
    r1: &Path,
    r2: &Path,
    r1_stats: &bijux_dna_core::prelude::measure::SeqkitMetrics,
    r2_stats: &bijux_dna_core::prelude::measure::SeqkitMetrics,
    tool: &str,
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    params: &serde_json::Value,
    merged_reads: &Path,
    execution: &StageResultV1,
) -> Result<BenchmarkRecord<FastqMergeMetrics>> {
    let merged_stats = observe_merge_stats(catalog, platform, runner, merged_reads)?;
    let pairs_in = r1_stats.reads.min(r2_stats.reads);
    let reads_merged = merged_stats.reads.min(pairs_in);
    let reads_unmerged = pairs_in.saturating_sub(reads_merged);
    let merge_rate = if pairs_in == 0 {
        0.0
    } else {
        reads_merged as f64 / pairs_in as f64
    };
    let metrics = FastqMergeMetrics {
        reads_in: r1_stats.reads + r2_stats.reads,
        reads_out: merged_stats.reads,
        bases_in: r1_stats.bases + r2_stats.bases,
        bases_out: merged_stats.bases,
        pairs_in,
        pairs_out: reads_merged,
        reads_r1: r1_stats.reads,
        reads_r2: r2_stats.reads,
        reads_merged,
        reads_unmerged,
        merge_rate,
    };
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    let report = serde_json::json!({
        "schema_version": "bijux.fastq.merge_pairs.report.v1",
        "stage_id": STAGE_MERGE_PAIRS.as_str(),
        "tool_id": tool,
        "input_r1": r1,
        "input_r2": r2,
        "merged_reads": merged_reads,
        "reads_r1": metrics.reads_r1,
        "reads_r2": metrics.reads_r2,
        "reads_merged": metrics.reads_merged,
        "reads_unmerged": metrics.reads_unmerged,
        "merge_rate": metrics.merge_rate,
        "runtime_s": execution.runtime_s,
        "memory_mb": execution.memory_mb,
        "exit_code": execution.exit_code,
    });
    let out_dir = merged_reads
        .parent()
        .ok_or_else(|| anyhow!("merge output has no parent"))?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("merge_report.json"), &report)
        .context("write merge report")?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write merge metrics")?;

    let context = build_benchmark_context(
        tool,
        tool_spec.tool_version.clone(),
        tool_spec
            .image
            .digest
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        runner,
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

fn merge_input_hash(r1: &Path, r2: &Path) -> Result<String> {
    let r1_hash = hash_file_sha256(r1).context("hash merge r1")?;
    let r2_hash = hash_file_sha256(r2).context("hash merge r2")?;
    params_hash(&serde_json::json!({ "r1": r1_hash, "r2": r2_hash }))
        .context("combine paired merge input hashes")
}

fn observe_merge_stats<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner: RuntimeKind,
    merged_reads: &Path,
) -> Result<bijux_dna_core::prelude::measure::SeqkitMetrics> {
    observe_fastq_stats(catalog, platform, runner, merged_reads)
}
