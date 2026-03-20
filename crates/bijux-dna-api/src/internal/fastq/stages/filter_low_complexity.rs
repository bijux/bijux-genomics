use std::collections::HashMap;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{filter_tools_by_role, load_workspace_registry};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::quality::{
    fetch_fastq_filter_low_complexity_v1, insert_fastq_filter_low_complexity_v1,
};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkRecord, FastqLowComplexityMetrics,
};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::params_hash;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::select_filter_low_complexity_tools;
use bijux_dna_planner_fastq::stage_api::fastq::filter_low_complexity::{
    plan_low_complexity, LowComplexityPlanOptions,
};
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use uuid::Uuid;

use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_FILTER_LOW_COMPLEXITY,
};
use crate::internal::fastq::stages::trim_bench_common::{
    build_benchmark_context, derive_trim_delta, observe_fastq_stats, prepare_trim_bench,
};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;

pub fn bench_fastq_filter_low_complexity<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqFilterLowComplexityArgs,
) -> Result<BenchOutcome<FastqLowComplexityMetrics>> {
    let tools = select_filter_low_complexity_tools(&args.tools)?;
    let artifact_kind = if args.r2.is_some() {
        FastqArtifactKind::PairedEnd
    } else {
        FastqArtifactKind::SingleEnd
    };
    preflight_stage(STAGE_FILTER_LOW_COMPLEXITY.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_FILTER_LOW_COMPLEXITY.as_str(), &header);

    let registry = load_workspace_registry()
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_FILTER_LOW_COMPLEXITY.as_str(), &tools, &registry, false)?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_FILTER_LOW_COMPLEXITY,
    )?;
    let input_hash = if let Some(r2) = args.r2.as_deref() {
        format!(
            "{}+{}",
            bench_inputs.input_hash,
            bijux_dna_infra::hash_file_sha256(r2)?
        )
    } else {
        bench_inputs.input_hash.clone()
    };
    let input_stats_r2 = if let Some(r2) = args.r2.as_deref() {
        Some(observe_fastq_stats(catalog, platform, bench_inputs.runner, r2)?)
    } else {
        None
    };

    if args.explain {
        write_explain_md(
            &bench_inputs.bench_dir,
            STAGE_FILTER_LOW_COMPLEXITY.as_str(),
            &tools,
            &[],
            None,
        )?;
        write_explain_plan_json(
            &bench_inputs.bench_dir,
            STAGE_FILTER_LOW_COMPLEXITY.as_str(),
            &tools,
            &registry,
            None,
        )?;
    }

    ensure_image_qa_passed(STAGE_FILTER_LOW_COMPLEXITY.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_FILTER_LOW_COMPLEXITY.as_str(), &tools, platform, catalog)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let options = LowComplexityPlanOptions {
        entropy_threshold: args.entropy_threshold,
        polyx_threshold: args.polyx_threshold,
    };
    let mut failures = Vec::new();
    let mut records = Vec::<BenchmarkRecord<FastqLowComplexityMetrics>>::new();
    for tool in &tools {
        let out_dir = bench_inputs.tools_root.join(tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec = build_tool_execution_spec(
            STAGE_FILTER_LOW_COMPLEXITY.as_str(),
            tool,
            &registry,
            catalog,
            platform,
        )?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan =
            plan_low_complexity(&tool_spec, &bench_inputs.r1, args.r2.as_deref(), &out_dir, &options)?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_filter_low_complexity_v1(
            &conn,
            tool,
            &tool_spec.tool_version,
            &image_digest,
            &bench_inputs.runner.to_string(),
            &platform.name,
            &input_hash,
            &params_hash,
        ) {
            records.push(record);
            continue;
        }
        let execution = execute_plans_with_jobs(
            vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&plan)],
            bench_inputs.runner,
            jobs,
        )?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing execution result for {tool}"))?;
        let record = build_low_complexity_record(
            catalog,
            platform,
            &bench_inputs,
            input_stats_r2.as_ref(),
            tool,
            &tool_spec,
            &input_hash,
            &plan.params,
            &plan.io.outputs[0].path,
            plan.io.outputs.get(1).map(|artifact| artifact.path.as_path()),
            &execution,
        )?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_filter_low_complexity_v1(&conn, &record).context("insert bench sqlite")?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: STAGE_FILTER_LOW_COMPLEXITY.as_str().to_string(),
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
        bench_dir: bench_inputs.bench_dir,
        explain: args.explain,
    })
}

fn build_low_complexity_record<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    bench_inputs: &crate::internal::fastq::stages::trim_bench_common::TrimBenchInputs,
    input_stats_r2: Option<&SeqkitMetrics>,
    tool: &str,
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    input_hash: &str,
    params: &serde_json::Value,
    output_reads: &std::path::Path,
    output_reads_r2: Option<&std::path::Path>,
    execution: &StageResultV1,
) -> Result<BenchmarkRecord<FastqLowComplexityMetrics>> {
    let output_stats_r1 = if execution.exit_code == 0 && output_reads.exists() {
        observe_fastq_stats(catalog, platform, bench_inputs.runner, output_reads)?
    } else {
        bench_inputs.input_stats.clone()
    };
    let output_stats_r2 = if let Some(output_reads_r2) = output_reads_r2 {
        if execution.exit_code == 0 && output_reads_r2.exists() {
            Some(observe_fastq_stats(
                catalog,
                platform,
                bench_inputs.runner,
                output_reads_r2,
            )?)
        } else {
            input_stats_r2.cloned()
        }
    } else {
        None
    };
    let before_stats = combine_seqkit_metrics(&bench_inputs.input_stats, input_stats_r2);
    let after_stats = combine_seqkit_metrics(&output_stats_r1, output_stats_r2.as_ref());
    let reads_removed = before_stats.reads.saturating_sub(after_stats.reads);
    let metrics = FastqLowComplexityMetrics {
        reads_in: before_stats.reads,
        reads_out: after_stats.reads,
        bases_in: before_stats.bases,
        bases_out: after_stats.bases,
        reads_removed_low_complexity: reads_removed,
        mean_q_before: before_stats.mean_q,
        mean_q_after: after_stats.mean_q,
        delta_metrics: derive_trim_delta(&before_stats, &after_stats),
    };
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    let report = serde_json::json!({
        "schema_version": "bijux.fastq.filter_low_complexity.report.v1",
        "stage_id": STAGE_FILTER_LOW_COMPLEXITY.as_str(),
        "tool_id": tool,
        "input_fastq": bench_inputs.r1,
        "output_fastq": output_reads,
        "output_fastq_r2": output_reads_r2,
        "reads_in": metrics.reads_in,
        "reads_out": metrics.reads_out,
        "reads_removed_low_complexity": metrics.reads_removed_low_complexity,
        "runtime_s": execution.runtime_s,
        "memory_mb": execution.memory_mb,
        "exit_code": execution.exit_code,
    });
    let out_dir = output_reads
        .parent()
        .ok_or_else(|| anyhow!("low-complexity output has no parent"))?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("low_complexity_report.json"), &report)
        .context("write low-complexity report")?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write low-complexity metrics")?;

    let context = build_benchmark_context(
        tool,
        tool_spec.tool_version.clone(),
        tool_spec
            .image
            .digest
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        bench_inputs.runner,
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

fn combine_seqkit_metrics(primary: &SeqkitMetrics, secondary: Option<&SeqkitMetrics>) -> SeqkitMetrics {
    let secondary_reads = secondary.map_or(0, |stats| stats.reads);
    let secondary_bases = secondary.map_or(0, |stats| stats.bases);
    let total_bases = primary.bases + secondary_bases;
    let weighted_mean_q = if total_bases == 0 {
        0.0
    } else {
        ((primary.mean_q * primary.bases as f64)
            + secondary.map_or(0.0, |stats| stats.mean_q * stats.bases as f64))
            / total_bases as f64
    };
    let weighted_gc = if total_bases == 0 {
        0.0
    } else {
        ((primary.gc_percent * primary.bases as f64)
            + secondary.map_or(0.0, |stats| stats.gc_percent * stats.bases as f64))
            / total_bases as f64
    };
    SeqkitMetrics {
        reads: primary.reads + secondary_reads,
        bases: total_bases,
        mean_q: weighted_mean_q,
        gc_percent: weighted_gc,
    }
}
