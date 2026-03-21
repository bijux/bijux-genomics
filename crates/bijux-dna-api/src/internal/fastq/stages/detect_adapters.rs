use std::collections::HashMap;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{filter_tools_by_role, load_workspace_registry};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_detect_adapters_v1, insert_fastq_detect_adapters_v1,
};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqDetectAdaptersMetrics};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::params_hash;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::select_detect_adapters_tools;
use bijux_dna_planner_fastq::stage_api::fastq::detect_adapters::plan;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use uuid::Uuid;

use crate::internal::fastq::stages::trim_bench_common::{
    build_benchmark_context, prepare_trim_bench,
};
use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_DETECT_ADAPTERS,
};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;

pub fn bench_fastq_detect_adapters<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDetectAdaptersArgs,
) -> Result<BenchOutcome<FastqDetectAdaptersMetrics>> {
    let tools = select_detect_adapters_tools(&args.tools)?;
    let artifact_kind = if args.r2.is_some() {
        FastqArtifactKind::PairedEnd
    } else {
        FastqArtifactKind::SingleEnd
    };
    preflight_stage(STAGE_DETECT_ADAPTERS.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_DETECT_ADAPTERS.as_str(), &header);

    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_DETECT_ADAPTERS.as_str(), &tools, &registry, false)?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_DETECT_ADAPTERS,
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
        Some(
            crate::internal::fastq::stages::trim_bench_common::observe_fastq_stats(
                catalog,
                platform,
                bench_inputs.runner,
                r2,
            )?,
        )
    } else {
        None
    };

    if args.explain {
        write_explain_md(
            &bench_inputs.bench_dir,
            STAGE_DETECT_ADAPTERS.as_str(),
            &tools,
            &[],
            None,
        )?;
        write_explain_plan_json(
            &bench_inputs.bench_dir,
            STAGE_DETECT_ADAPTERS.as_str(),
            &tools,
            &registry,
            None,
        )?;
    }

    ensure_image_qa_passed(STAGE_DETECT_ADAPTERS.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_DETECT_ADAPTERS.as_str(), &tools, platform, catalog)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::<BenchmarkRecord<FastqDetectAdaptersMetrics>>::new();
    for tool in &tools {
        let out_dir = bench_inputs.tools_root.join(tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec = build_tool_execution_spec(
            STAGE_DETECT_ADAPTERS.as_str(),
            tool,
            &registry,
            catalog,
            platform,
        )?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = plan(&tool_spec, &bench_inputs.r1, args.r2.as_deref(), &out_dir)?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_detect_adapters_v1(
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
        let record = build_detect_record(
            platform,
            &bench_inputs,
            input_stats_r2.as_ref(),
            tool,
            &tool_spec,
            &input_hash,
            &plan.params,
            &out_dir,
            &execution,
        )?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_detect_adapters_v1(&conn, &record).context("insert bench sqlite")?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: STAGE_DETECT_ADAPTERS.as_str().to_string(),
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

fn build_detect_record(
    platform: &PlatformSpec,
    bench_inputs: &crate::internal::fastq::stages::trim_bench_common::TrimBenchInputs,
    input_stats_r2: Option<&SeqkitMetrics>,
    tool: &str,
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    input_hash: &str,
    params: &serde_json::Value,
    out_dir: &std::path::Path,
    execution: &StageResultV1,
) -> Result<BenchmarkRecord<FastqDetectAdaptersMetrics>> {
    let (candidate_adapter_count, adapter_trimmed_fraction) = detect_adapter_summary(out_dir)?;
    let reads_in = bench_inputs.input_stats.reads + input_stats_r2.map_or(0, |stats| stats.reads);
    let bases_in = bench_inputs.input_stats.bases + input_stats_r2.map_or(0, |stats| stats.bases);
    let mean_q = if bases_in == 0 {
        0.0
    } else {
        ((bench_inputs.input_stats.mean_q * bench_inputs.input_stats.bases as f64)
            + input_stats_r2.map_or(0.0, |stats| stats.mean_q * stats.bases as f64))
            / bases_in as f64
    };
    let metrics = FastqDetectAdaptersMetrics {
        reads_in,
        reads_out: reads_in,
        bases_in,
        bases_out: bases_in,
        mean_q,
        candidate_adapter_count,
        adapter_trimmed_fraction,
    };
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    let report = serde_json::json!({
        "schema_version": "bijux.fastq.detect_adapters.report.v1",
        "stage_id": STAGE_DETECT_ADAPTERS.as_str(),
        "tool_id": tool,
        "inspection_mode": "evidence_only",
        "report_only": true,
        "evidence_engine": tool,
        "input_fastq": bench_inputs.r1,
        "paired_input": input_stats_r2.is_some(),
        "candidate_adapter_count": metrics.candidate_adapter_count,
        "adapter_trimmed_fraction": metrics.adapter_trimmed_fraction,
        "fastqc_dir": out_dir.join("fastqc"),
        "runtime_s": execution.runtime_s,
        "memory_mb": execution.memory_mb,
        "exit_code": execution.exit_code,
    });
    bijux_dna_infra::atomic_write_json(&out_dir.join("adapter_report.json"), &report)
        .context("write adapter report")?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write adapter metrics")?;

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

fn detect_adapter_summary(out_dir: &std::path::Path) -> Result<(u64, Option<f64>)> {
    let fastp_json = out_dir.join("fastp.json");
    if fastp_json.exists() {
        let raw = std::fs::read_to_string(&fastp_json)
            .with_context(|| format!("read {}", fastp_json.display()))?;
        let parsed: serde_json::Value = serde_json::from_str(&raw)
            .with_context(|| format!("parse {}", fastp_json.display()))?;
        let adapter_trimmed_reads = parsed
            .pointer("/adapter_cutting/adapter_trimmed_reads")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let total_reads = parsed
            .pointer("/summary/before_filtering/total_reads")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let fraction = if total_reads > 0 {
            Some(adapter_trimmed_reads as f64 / total_reads as f64)
        } else {
            None
        };
        let count = if adapter_trimmed_reads > 0 { 1 } else { 0 };
        return Ok((count, fraction));
    }
    Ok((u64::from(out_dir.join("fastqc").exists()), None))
}
