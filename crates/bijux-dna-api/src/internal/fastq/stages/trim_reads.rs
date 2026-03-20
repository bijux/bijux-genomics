use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::query_shared::{fetch_fastq_trim_v2, insert_fastq_trim_v2};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqTrimMetrics};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::params_hash;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::select_trim_tools;
use bijux_dna_planner_fastq::stage_api::fastq::trim_reads::plan;
use bijux_dna_planner_fastq::stage_api::{
    adapter_bank_context, contaminant_bank_context, inspect_headers, log_header_warnings,
    polyx_bank_context, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use uuid::Uuid;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{filter_tools_by_role, load_workspace_registry};

use super::trim_bench_common::{
    build_benchmark_context, derive_trim_delta, json_string, observe_fastq_stats,
    prepare_trim_bench,
};
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_TRIM_READS,
};

/// # Errors
/// Returns an error if planning, execution, metric derivation, or persistence fails.
pub fn bench_fastq_trim<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimArgs,
) -> Result<BenchOutcome<FastqTrimMetrics>> {
    let tools = select_trim_tools(&args.tools, false)?;
    let artifact_kind = if args.r2.is_some() {
        FastqArtifactKind::PairedEnd
    } else {
        FastqArtifactKind::SingleEnd
    };
    preflight_stage(STAGE_TRIM_READS.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_TRIM_READS.as_str(), &header);

    let registry = load_workspace_registry()
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_TRIM_READS.as_str(), &tools, &registry, false)?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_TRIM_READS,
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

    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_TRIM_READS.as_str());
    let all_tools: Vec<String> = registry
        .tools_for_stage(&stage_id)
        .iter()
        .map(|tool| tool.tool_id.to_string())
        .collect();
    let excluded: Vec<String> = all_tools
        .into_iter()
        .filter(|tool| !tools.contains(tool))
        .collect();

    if args.explain {
        write_explain_md(
            &bench_inputs.bench_dir,
            STAGE_TRIM_READS.as_str(),
            &tools,
            &excluded,
            None,
        )?;
        write_explain_plan_json(
            &bench_inputs.bench_dir,
            STAGE_TRIM_READS.as_str(),
            &tools,
            &registry,
            None,
        )?;
    }

    ensure_image_qa_passed(STAGE_TRIM_READS.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_TRIM_READS.as_str(), &tools, platform, catalog)?;

    let adapter_context = adapter_bank_context(
        args.adapter_bank_preset.as_deref(),
        args.adapter_bank.as_deref(),
        args.adapter_bank_file.as_deref(),
        &args.enable_adapters,
        &args.disable_adapters,
    )?;
    let polyx_context = polyx_bank_context(args.polyx_preset.as_deref())?;
    let contaminant_context = contaminant_bank_context(args.contaminant_preset.as_deref())?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut records = Vec::<BenchmarkRecord<FastqTrimMetrics>>::new();
    let mut failures = Vec::<RawFailure>::new();

    for tool in tools {
        let out_dir = bench_inputs.tools_root.join(&tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec = build_tool_execution_spec(
            STAGE_TRIM_READS.as_str(),
            &tool,
            &registry,
            catalog,
            platform,
        )?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = plan(
            &tool_spec,
            &bench_inputs.r1,
            args.r2.as_deref(),
            &out_dir,
            adapter_context.as_ref(),
            polyx_context.as_ref(),
            contaminant_context.as_ref(),
        )?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_trim_v2(
            &conn,
            &tool,
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

        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: STAGE_TRIM_READS.as_str().to_string(),
                tool: tool.clone(),
                reason: format!("tool `{tool}` failed with status {}", execution.exit_code),
                category: ErrorCategory::ToolError,
            });
            continue;
        }

        let output_r1 = plan.io.outputs[0].path.clone();
        let output_stats_r1 =
            observe_fastq_stats(catalog, platform, bench_inputs.runner, &output_r1)?;
        let output_stats_r2 = if args.r2.is_some() {
            Some(observe_fastq_stats(
                catalog,
                platform,
                bench_inputs.runner,
                &plan.io.outputs[1].path,
            )?)
        } else {
            None
        };
        let before_stats = combine_seqkit_metrics(&bench_inputs.input_stats, input_stats_r2.as_ref());
        let after_stats = combine_seqkit_metrics(&output_stats_r1, output_stats_r2.as_ref());
        let metrics = FastqTrimMetrics {
            reads_in: before_stats.reads,
            reads_out: after_stats.reads,
            bases_in: before_stats.bases,
            bases_out: after_stats.bases,
            pairs_in: input_stats_r2.as_ref().map(|stats| bench_inputs.input_stats.reads.min(stats.reads)),
            pairs_out: output_stats_r2.as_ref().map(|stats| output_stats_r1.reads.min(stats.reads)),
            mean_q_before: before_stats.mean_q,
            mean_q_after: after_stats.mean_q,
            delta_metrics: derive_trim_delta(&before_stats, &after_stats),
            adapter_preset: json_string(adapter_context.as_ref(), "preset")
                .or_else(|| args.adapter_bank_preset.clone()),
            adapter_bank_id: json_string(adapter_context.as_ref(), "bank_id"),
            adapter_bank_hash: json_string(adapter_context.as_ref(), "bank_hash"),
            adapter_overrides: if args.enable_adapters.is_empty() && args.disable_adapters.is_empty() {
                None
            } else {
                Some(
                    serde_json::json!({
                        "enable": args.enable_adapters,
                        "disable": args.disable_adapters,
                    })
                    .into(),
                )
            },
        };
        let metric_set = metric_set(metrics.clone());
        bijux_dna_analyze::validate_metric_set(&metric_set)?;

        let report = serde_json::json!({
            "schema_version": "bijux.fastq.trim_reads.report.v1",
            "stage_id": STAGE_TRIM_READS.as_str(),
            "tool_id": tool,
            "reads_in": metrics.reads_in,
            "reads_out": metrics.reads_out,
            "bases_in": metrics.bases_in,
            "bases_out": metrics.bases_out,
            "output_r1": output_r1,
            "output_r2": args.r2.as_ref().map(|_| plan.io.outputs[1].path.clone()),
            "mean_q_before": metrics.mean_q_before,
            "mean_q_after": metrics.mean_q_after,
            "runtime_s": execution.runtime_s,
            "memory_mb": execution.memory_mb,
            "adapter_bank": adapter_context,
            "polyx_bank": polyx_context,
            "contaminant_bank": contaminant_context,
        });
        bijux_dna_infra::atomic_write_json(&out_dir.join("trim_report.json"), &report)
            .context("write trim report")?;
        let metrics_json = serde_json::to_value(&metric_set)?;
        bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
            .context("write trim metrics")?;

        let context = build_benchmark_context(
            &tool,
            tool_spec.tool_version.clone(),
            image_digest,
            bench_inputs.runner,
            platform,
            input_hash.clone(),
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
        insert_fastq_trim_v2(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: bench_inputs.bench_dir,
        explain: args.explain,
    })
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
