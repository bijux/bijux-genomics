use std::collections::HashMap;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_workspace_registry};
use anyhow::{anyhow, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_duplicates_v1, insert_fastq_duplicates_v1,
};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqDuplicateMetrics};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::params_hash;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use uuid::Uuid;

use crate::internal::fastq::stages::trim_bench_common::{
    build_benchmark_context, observe_fastq_stats,
};
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};

const STAGE_ID: &str = "fastq.remove_duplicates";

pub fn bench_fastq_remove_duplicates<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqRemoveDuplicatesArgs,
) -> Result<BenchOutcome<FastqDuplicateMetrics>> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = bijux_dna_planner_fastq::select_remove_duplicates_tools(&args.tools)?;
    let tools = filter_tools_by_role(STAGE_ID, &tools, &registry, false)?;
    let artifact_kind = if args.r2.is_some() {
        FastqArtifactKind::PairedEnd
    } else {
        FastqArtifactKind::SingleEnd
    };
    preflight_stage(STAGE_ID, artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_ID, &header);
    let runner = ensure_bench_runner(platform, runner_override)?;
    let input_stats = observe_fastq_stats(catalog, platform, runner, &args.r1)?;
    let input_stats_r2 = if let Some(r2) = args.r2.as_deref() {
        Some(observe_fastq_stats(catalog, platform, runner, r2)?)
    } else {
        None
    };
    let input_hash = if let Some(r2) = args.r2.as_deref() {
        format!("{}+{}", hash_file_sha256(&args.r1)?, hash_file_sha256(r2)?)
    } else {
        hash_file_sha256(&args.r1)?
    };
    let bench_dir_name =
        bench_dir_name(&bijux_dna_domain_fastq::stages::ids::STAGE_REMOVE_DUPLICATES)
            .ok_or_else(|| anyhow!("bench dir missing for {STAGE_ID}"))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir)?;
    bijux_dna_infra::ensure_dir(&tools_root)?;

    if args.explain {
        write_explain_md(&bench_dir, STAGE_ID, &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, STAGE_ID, &tools, &registry, None)?;
    }

    ensure_image_qa_passed(STAGE_ID, &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_ID, &tools, platform, catalog)?;

    let sqlite_path = bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path)?;
    let bench_path = bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::new();

    for tool in &tools {
        let out_dir = tools_root.join(tool);
        bijux_dna_infra::ensure_dir(&out_dir)?;
        let tool_spec = build_tool_execution_spec(STAGE_ID, tool, &registry, catalog, platform)?;
        let plan =
            bijux_dna_planner_fastq::tool_adapters::fastq::remove_duplicates::plan_deduplicate(
                &tool_spec,
                &args.r1,
                args.r2.as_deref(),
                &out_dir,
            )?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_duplicates_v1(
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
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: STAGE_ID.to_string(),
                tool: tool.clone(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
                category: ErrorCategory::ToolError,
            });
            continue;
        }
        if !plan.io.outputs[0].path.exists() {
            std::fs::copy(&args.r1, &plan.io.outputs[0].path)?;
        }
        let output_stats =
            observe_fastq_stats(catalog, platform, runner, &plan.io.outputs[0].path)?;
        let output_stats_r2 = if let Some(r2) = args.r2.as_deref() {
            let output_r2 = &plan.io.outputs[1].path;
            if !output_r2.exists() {
                std::fs::copy(r2, output_r2)?;
            }
            Some(observe_fastq_stats(catalog, platform, runner, output_r2)?)
        } else {
            None
        };
        let reads_in = input_stats.reads + input_stats_r2.as_ref().map_or(0, |stats| stats.reads);
        let reads_out =
            output_stats.reads + output_stats_r2.as_ref().map_or(0, |stats| stats.reads);
        let duplicate_reads = reads_in.saturating_sub(reads_out);
        let metrics = FastqDuplicateMetrics {
            reads_in,
            reads_out,
            duplicate_reads,
            dedup_rate: if reads_in == 0 {
                0.0
            } else {
                duplicate_reads as f64 / reads_in as f64
            },
        };
        let metric_set = metric_set(metrics);
        let report = serde_json::json!({
            "schema_version": "bijux.fastq.remove_duplicates.report.v1",
            "stage": STAGE_ID,
            "stage_id": STAGE_ID,
            "tool": tool,
            "tool_id": tool,
            "input_fastq_r1": args.r1,
            "input_fastq_r2": args.r2,
            "dedup_reads_r1": plan.io.outputs[0].path,
            "dedup_reads_r2": output_stats_r2.as_ref().map(|_| plan.io.outputs[1].path.clone()),
            "report_json": if output_stats_r2.is_some() {
                plan.io.outputs[2].path.clone()
            } else {
                plan.io.outputs[1].path.clone()
            },
            "duplicates_removed": duplicate_reads,
            "duplicate_reads": duplicate_reads,
            "runtime_s": execution.runtime_s,
            "memory_mb": execution.memory_mb,
            "exit_code": execution.exit_code,
        });
        bijux_dna_infra::atomic_write_json(&out_dir.join("deduplicate_report.json"), &report)?;
        bijux_dna_infra::atomic_write_json(
            &out_dir.join("metrics.json"),
            &serde_json::to_value(&metric_set)?,
        )?;
        let record = BenchmarkRecord {
            context: build_benchmark_context(
                tool,
                tool_spec.tool_version.clone(),
                image_digest,
                runner,
                platform,
                input_hash.clone(),
                plan.params.clone(),
            ),
            execution: ExecutionMetrics {
                runtime_s: execution.runtime_s,
                memory_mb: execution.memory_mb,
                exit_code: execution.exit_code,
            },
            metrics: metric_set,
        };
        record.validate()?;
        append_jsonl(&bench_path, &record)?;
        insert_fastq_duplicates_v1(&conn, &record)?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir,
        explain: args.explain,
    })
}
