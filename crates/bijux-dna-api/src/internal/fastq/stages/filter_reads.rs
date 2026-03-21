use std::collections::HashMap;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{filter_tools_by_role, load_workspace_registry};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::quality::{fetch_fastq_filter_v2, insert_fastq_filter_v2};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqFilterMetrics};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::select_filter_tools;
use bijux_dna_planner_fastq::stage_api::fastq::filter_reads::{plan_filter, FilterPlanOptions};
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use crate::internal::fastq::stages::trim_bench_common::{
    build_benchmark_context, derive_trim_delta, observe_fastq_stats, prepare_trim_bench,
};
use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_FILTER_READS,
};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_filter<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqFilterArgs,
) -> Result<BenchOutcome<bijux_dna_analyze::FastqFilterMetrics>> {
    let tools = select_filter_tools(&args.tools)?;
    let artifact_kind = if args.r2.is_some() {
        FastqArtifactKind::PairedEnd
    } else {
        FastqArtifactKind::SingleEnd
    };
    preflight_stage(STAGE_FILTER_READS.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_FILTER_READS.as_str(), &header);

    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_FILTER_READS.as_str(), &tools, &registry, false)?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_FILTER_READS,
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
        Some(observe_fastq_stats(
            catalog,
            platform,
            bench_inputs.runner,
            r2,
        )?)
    } else {
        None
    };

    if args.explain {
        write_explain_md(
            &bench_inputs.bench_dir,
            STAGE_FILTER_READS.as_str(),
            &tools,
            &[],
            None,
        )?;
        write_explain_plan_json(
            &bench_inputs.bench_dir,
            STAGE_FILTER_READS.as_str(),
            &tools,
            &registry,
            None,
        )?;
    }

    ensure_image_qa_passed(STAGE_FILTER_READS.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_FILTER_READS.as_str(), &tools, platform, catalog)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let filter_options = FilterPlanOptions {
        max_n: args.max_n,
        max_n_fraction: None,
        max_n_count: None,
        low_complexity_threshold: args.low_complexity_threshold,
        entropy_threshold: None,
        kmer_ref: args.kmer_ref.clone(),
        redundant_filters: Vec::new(),
        polyx_policy: None,
    };
    let mut failures = Vec::new();
    let mut records = Vec::<BenchmarkRecord<FastqFilterMetrics>>::new();
    for tool in &tools {
        let out_dir = bench_inputs.tools_root.join(tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec = build_tool_execution_spec(
            STAGE_FILTER_READS.as_str(),
            tool,
            &registry,
            catalog,
            platform,
        )?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = plan_filter(
            &tool_spec,
            &args.r1,
            args.r2.as_deref(),
            &out_dir,
            &filter_options,
        )?;
        let params_hash = stable_params_hash(&plan.params);
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_filter_v2(
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
            vec![bijux_dna_stage_contract::execution_step_from_stage_plan(
                &plan,
            )],
            bench_inputs.runner,
            jobs,
        )?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing execution result for {tool}"))?;
        let record = build_filter_record(
            catalog,
            platform,
            &bench_inputs,
            input_stats_r2.as_ref(),
            tool,
            &tool_spec,
            &input_hash,
            &plan.params,
            &plan.io.outputs[0].path,
            plan.io
                .outputs
                .get(1)
                .map(|artifact| artifact.path.as_path()),
            &execution,
        )?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_filter_v2(&conn, &record).context("insert bench sqlite")?;
        if execution.exit_code != 0 {
            let tool_name = tool.clone();
            failures.push(RawFailure {
                stage: STAGE_FILTER_READS.as_str().to_string(),
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
        bench_dir: bench_inputs.bench_dir,
        explain: args.explain,
    })
}

fn build_filter_record<S: ::std::hash::BuildHasher>(
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
) -> Result<BenchmarkRecord<FastqFilterMetrics>> {
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
    let reads_in = bench_inputs.input_stats.reads + input_stats_r2.map_or(0, |stats| stats.reads);
    let reads_out = output_stats_r1.reads + output_stats_r2.as_ref().map_or(0, |stats| stats.reads);
    let bases_in = bench_inputs.input_stats.bases + input_stats_r2.map_or(0, |stats| stats.bases);
    let bases_out = output_stats_r1.bases + output_stats_r2.as_ref().map_or(0, |stats| stats.bases);
    let reads_dropped = reads_in.saturating_sub(reads_out);
    let pairs_in = input_stats_r2.map(|stats| bench_inputs.input_stats.reads.min(stats.reads));
    let pairs_out = output_stats_r2
        .as_ref()
        .map(|stats| output_stats_r1.reads.min(stats.reads));
    let metrics = FastqFilterMetrics {
        reads_in,
        reads_out,
        reads_dropped,
        reads_removed_by_n: 0,
        reads_removed_by_entropy: 0,
        reads_removed_low_complexity: 0,
        reads_removed_by_kmer: 0,
        reads_removed_contaminant_kmer: 0,
        reads_removed_by_length: 0,
        bases_in,
        bases_out,
        pairs_in,
        pairs_out,
        mean_q_before: bench_inputs.input_stats.mean_q,
        mean_q_after: output_stats_r1.mean_q,
        delta_metrics: derive_trim_delta(&bench_inputs.input_stats, &output_stats_r1),
    };
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    let report = serde_json::json!({
        "schema_version": "bijux.fastq.filter_reads.report.v2",
        "stage_id": STAGE_FILTER_READS.as_str(),
        "tool_id": tool,
        "input_fastq": bench_inputs.r1,
        "output_fastq": output_reads,
        "input_fastq_r2": input_stats_r2.map(|_| serde_json::Value::String("paired".to_string())),
        "output_fastq_r2": output_reads_r2,
        "reads_in": metrics.reads_in,
        "reads_out": metrics.reads_out,
        "reads_dropped": metrics.reads_dropped,
        "bases_in": metrics.bases_in,
        "bases_out": metrics.bases_out,
        "mean_q_before": metrics.mean_q_before,
        "mean_q_after": metrics.mean_q_after,
        "runtime_s": execution.runtime_s,
        "memory_mb": execution.memory_mb,
        "exit_code": execution.exit_code,
    });
    let out_dir = output_reads
        .parent()
        .ok_or_else(|| anyhow!("filter output has no parent"))?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("filter_report.json"), &report)
        .context("write filter report")?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write filter metrics")?;

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
