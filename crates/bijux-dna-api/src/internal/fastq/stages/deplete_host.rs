use std::collections::HashMap;

use crate::internal::fastq::stages::trim_bench_common::{
    build_benchmark_context, observe_fastq_stats, prepare_trim_bench,
};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_workspace_registry};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_deplete_host_v1, insert_fastq_deplete_host_v1,
};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqDepleteHostMetrics};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::params_hash;
use bijux_dna_domain_fastq::stages::ids::STAGE_DEPLETE_HOST;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::hash_file_sha256;
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_planner_fastq::tool_adapters::stages::transform::deplete_host::plan_host_depletion;
use bijux_dna_planner_fastq::select_deplete_host_tools;
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;

use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};

fn output_path(plan: &bijux_dna_stage_contract::StagePlanV1, name: &str) -> Result<std::path::PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == name)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| anyhow!("missing planned output {name}"))
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_deplete_host<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqDepleteHostArgs,
) -> Result<BenchOutcome<FastqDepleteHostMetrics>> {
    let tools = select_deplete_host_tools(&args.tools)?;
    let artifact_kind = if args.r2.is_some() {
        FastqArtifactKind::PairedEnd
    } else {
        FastqArtifactKind::SingleEnd
    };
    preflight_stage(STAGE_DEPLETE_HOST.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings(STAGE_DEPLETE_HOST.as_str(), &header);

    let registry = load_workspace_registry()
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_DEPLETE_HOST.as_str(), &tools, &registry, false)?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_DEPLETE_HOST,
    )?;
    let input_hash = if let Some(r2) = args.r2.as_ref() {
        format!("{}+{}", bench_inputs.input_hash, hash_file_sha256(r2)?)
    } else {
        bench_inputs.input_hash.clone()
    };
    let input_stats_r2 = if let Some(r2) = args.r2.as_ref() {
        Some(observe_fastq_stats(catalog, platform, bench_inputs.runner, r2)?)
    } else {
        None
    };

    if args.explain {
        write_explain_md(
            &bench_inputs.bench_dir,
            STAGE_DEPLETE_HOST.as_str(),
            &tools,
            &[],
            None,
        )?;
        write_explain_plan_json(
            &bench_inputs.bench_dir,
            STAGE_DEPLETE_HOST.as_str(),
            &tools,
            &registry,
            None,
        )?;
    }

    let runner = ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed(STAGE_DEPLETE_HOST.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_DEPLETE_HOST.as_str(), &tools, platform, catalog)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::<RawFailure>::new();
    let mut records = Vec::<BenchmarkRecord<FastqDepleteHostMetrics>>::new();

    for tool in tools {
        let out_dir = bench_inputs.tools_root.join(&tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec =
            build_tool_execution_spec(STAGE_DEPLETE_HOST.as_str(), &tool, &registry, catalog, platform)?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = plan_host_depletion(
            &tool_spec,
            &bench_inputs.r1,
            args.r2.as_deref(),
            &args.reference_index,
            &out_dir,
        )?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_deplete_host_v1(
            &conn,
            &tool,
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
                stage: STAGE_DEPLETE_HOST.as_str().to_string(),
                tool: tool.clone(),
                reason: format!("tool `{tool}` failed with status {}", execution.exit_code),
                category: ErrorCategory::ToolError,
            });
            continue;
        }

        let output_r1 = output_path(&plan, "host_depleted_reads_r1")?;
        let output_stats_r1 = observe_fastq_stats(catalog, platform, runner, &output_r1)?;
        let (reads_in, reads_out, bases_in, bases_out, pairs_in, pairs_out, summary) =
            if let Some(input_r2) = input_stats_r2.as_ref() {
                let output_r2 = output_path(&plan, "host_depleted_reads_r2")?;
                let output_stats_r2 = observe_fastq_stats(catalog, platform, runner, &output_r2)?;
                (
                    bench_inputs.input_stats.reads + input_r2.reads,
                    output_stats_r1.reads + output_stats_r2.reads,
                    bench_inputs.input_stats.bases + input_r2.bases,
                    output_stats_r1.bases + output_stats_r2.bases,
                    bench_inputs.input_stats.reads.min(input_r2.reads),
                    output_stats_r1.reads.min(output_stats_r2.reads),
                    serde_json::json!({
                        "reads_removed": (bench_inputs.input_stats.reads + input_r2.reads)
                            .saturating_sub(output_stats_r1.reads + output_stats_r2.reads),
                        "bases_removed": (bench_inputs.input_stats.bases + input_r2.bases)
                            .saturating_sub(output_stats_r1.bases + output_stats_r2.bases),
                        "output_r1": output_r1,
                        "output_r2": output_r2,
                        "removed_host_r1": output_path(&plan, "removed_host_reads_r1")?,
                        "removed_host_r2": output_path(&plan, "removed_host_reads_r2")?,
                        "report_json": output_path(&plan, "host_depletion_report_json")?,
                    }),
                )
            } else {
                (
                    bench_inputs.input_stats.reads,
                    output_stats_r1.reads,
                    bench_inputs.input_stats.bases,
                    output_stats_r1.bases,
                    0,
                    0,
                    serde_json::json!({
                        "reads_removed": bench_inputs.input_stats.reads.saturating_sub(output_stats_r1.reads),
                        "bases_removed": bench_inputs.input_stats.bases.saturating_sub(output_stats_r1.bases),
                        "output_fastq": output_r1,
                        "removed_host_reads": output_path(&plan, "removed_host_reads_r1")?,
                        "report_json": output_path(&plan, "host_depletion_report_json")?,
                    }),
                )
            };
        let host_fraction_removed = if reads_in == 0 {
            0.0
        } else {
            1.0 - (reads_out as f64 / reads_in as f64)
        };
        let metrics = FastqDepleteHostMetrics {
            reads_in,
            reads_out,
            bases_in,
            bases_out,
            pairs_in,
            pairs_out,
            host_fraction_removed: host_fraction_removed.clamp(0.0, 1.0),
            depletion_summary: summary.into(),
        };
        let metric_set = metric_set(metrics.clone());
        bijux_dna_analyze::validate_metric_set(&metric_set)?;
        let stage_report = serde_json::json!({
            "schema_version": "bijux.fastq.deplete_host.report.v1",
            "stage_id": STAGE_DEPLETE_HOST.as_str(),
            "tool_id": tool,
            "host_fraction_removed": metrics.host_fraction_removed,
            "reads_in": metrics.reads_in,
            "reads_out": metrics.reads_out,
            "bases_in": metrics.bases_in,
            "bases_out": metrics.bases_out,
            "runtime_s": execution.runtime_s,
            "memory_mb": execution.memory_mb,
        });
        bijux_dna_infra::atomic_write_json(&out_dir.join("host_depletion_report.json"), &stage_report)
            .context("write host depletion report")?;
        bijux_dna_infra::atomic_write_json(
            &out_dir.join("metrics.json"),
            &serde_json::to_value(&metric_set)?,
        )
        .context("write host depletion metrics")?;

        let context = build_benchmark_context(
            &tool,
            tool_spec.tool_version.clone(),
            image_digest,
            runner,
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
        insert_fastq_deplete_host_v1(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: bench_inputs.bench_dir,
        explain: args.explain,
    })
}
