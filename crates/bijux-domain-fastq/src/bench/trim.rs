use std::collections::HashMap;
use std::fs;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_analyze::{
    append_jsonl, fetch_fastq_trim_v1, insert_fastq_trim_v1, BenchmarkContext, BenchmarkRecord,
    FastqTrimMetrics, MetricSet,
};
use bijux_engine::api::load_registry;
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use bijux_measure::ExecutionMetrics;
use uuid::Uuid;

use crate::image_qa::ensure_image_qa_passed;
use bijux_engine::api::validate_execution_outputs;
use bijux_engine::api::{bench_base_dir, bench_tools_dir};
use bijux_engine::api::{docker_rm, docker_stats_mb, run_tool_container};
use bijux_engine::api::{hash_file_sha256, input_fastq_stats, output_fastq_stats};

use super::analyze::failure::{classify_failure, BenchmarkFailure};
use super::analyze::report::write_trim_report;
use super::helpers::{
    compute_run_id, normalize_tool_list, params_hash, prepare_tool_run_dirs, resolve_image_for_run,
    write_execution_logs, write_explain_md, write_metrics_json, ExecutionManifest,
};

#[allow(clippy::too_many_lines)]
/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning, execution, or metric recording fails.
pub fn bench_fastq_trim<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::bench::args::BenchFastqTrimArgs,
) -> Result<()> {
    let runner = runner_override.unwrap_or(platform.runner);
    if runner != RunnerKind::Docker {
        return Err(anyhow!("benchmarking supports docker only for now"));
    }
    let tools = normalize_tool_list(&args.tools)?;
    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let bench_dir = bench_base_dir(&args.out, "trim", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "trim", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    println!("planned tools: {}", tools.join(", "));
    let selected = tools.clone();
    let all_tools: Vec<String> = registry
        .tools_for_stage("fastq.trim")
        .iter()
        .map(|tool| tool.tool_id.clone())
        .collect();
    let excluded: Vec<String> = all_tools
        .into_iter()
        .filter(|tool| !selected.contains(tool))
        .collect();
    write_explain_md(&bench_dir, "fastq.trim", &selected, &excluded, None)?;

    let r1 = args.r1.canonicalize().context("resolve r1 path")?;
    let r1_dir = r1
        .parent()
        .ok_or_else(|| anyhow!("r1 has no parent"))?
        .to_path_buf();

    let seqkit_spec = catalog
        .get("seqkit")
        .ok_or_else(|| anyhow!("seqkit missing from images.yaml"))?;
    let seqkit_image = resolve_image_for_run(seqkit_spec, platform)?;
    let input_hash = hash_file_sha256(&r1)?;
    let input_stats = input_fastq_stats(&seqkit_image, &r1_dir, &r1)?;

    ensure_image_qa_passed("fastq.trim", &tools, platform, catalog)?;

    let sqlite_path = bench_dir.join("bench.sqlite");
    let conn = bijux_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let mut records: Vec<BenchmarkRecord<FastqTrimMetrics>> = Vec::new();
    let mut new_records: Vec<BenchmarkRecord<FastqTrimMetrics>> = Vec::new();
    let mut failures: Vec<BenchmarkFailure> = Vec::new();

    for tool in tools {
        let record = (|| -> Result<BenchmarkRecord<FastqTrimMetrics>> {
            let spec = catalog
                .get(&tool)
                .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
            let image = resolve_image_for_run(spec, platform)?;
            let image_digest = spec
                .digest
                .as_ref()
                .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
                .to_string();
            let cached =
                fetch_fastq_trim_v1(&conn, &tool, &spec.version, &image_digest, &input_hash);
            if let Ok(Some(record)) = cached {
                return Ok(record);
            }

            let params = serde_json::json!({
                "sample_id": args.sample_id,
                "r1": r1,
            });
            let param_hash = params_hash(&params).unwrap_or_else(|_| Uuid::new_v4().to_string());
            let run_id =
                compute_run_id("fastq.trim", &tool, &image_digest, &input_hash, &param_hash);
            let run_dirs = prepare_tool_run_dirs(&tools_root, &tool, &run_id)?;
            let out_dir = run_dirs.artifacts_dir.clone();
            let start = Instant::now();
            let container_name = format!("bijux-bench-{}-{}", args.sample_id, Uuid::new_v4());
            let execution =
                run_tool_container(&tool, &image, &r1_dir, &r1, &out_dir, &container_name)?;
            let runtime_s = start.elapsed().as_secs_f64();
            let memory_mb = docker_stats_mb(&container_name)?;
            docker_rm(&container_name)?;

            let out_fastq = execution
                .output_fastq
                .as_ref()
                .ok_or_else(|| anyhow!("output fastq missing"))?;
            let output_stats = output_fastq_stats(&seqkit_image, &out_dir, out_fastq)?;

            let tool_manifest = registry
                .tool_by_id("fastq.trim", &tool)
                .ok_or_else(|| anyhow!("tool {tool} missing from manifests"))?;
            validate_execution_outputs(&tool_manifest.execution_contract, &out_dir)?;

            let metrics = FastqTrimMetrics {
                reads_in: input_stats.reads,
                reads_out: output_stats.reads,
                bases_in: input_stats.bases,
                bases_out: output_stats.bases,
                mean_q_before: input_stats.mean_q,
                mean_q_after: output_stats.mean_q,
            };
            let metric_set = MetricSet::new(metrics);
            metric_set.validate()?;

            let manifest = ExecutionManifest {
                run_id: run_id.clone(),
                stage: "fastq.trim".to_string(),
                tool: tool.clone(),
                tool_version: spec.version.clone(),
                image_digest: image_digest.clone(),
                command: execution.command.clone(),
                input_hashes: vec![input_hash.clone()],
                input_files: vec![r1.display().to_string()],
                output_dir: out_dir.display().to_string(),
                runner: runner.to_string(),
                platform: platform.name.clone(),
                arch: platform.arch.clone(),
            };
            fs::write(
                &run_dirs.manifest_path,
                serde_json::to_vec_pretty(&manifest)?,
            )
            .context("write execution manifest")?;
            write_execution_logs(&run_dirs, &execution.stdout, &execution.stderr)?;

            let context = BenchmarkContext {
                tool: tool.clone(),
                tool_version: spec.version.clone(),
                image_digest,
                runner: runner.to_string(),
                platform: platform.name.clone(),
                input_hash: input_hash.clone(),
                parameters: params,
            };
            let execution_metrics = ExecutionMetrics {
                runtime_s,
                memory_mb,
                exit_code: execution.exit_code,
            };
            write_metrics_json(&run_dirs, &execution_metrics, &metric_set)?;
            let record = BenchmarkRecord {
                context,
                execution: execution_metrics,
                metrics: metric_set,
            };
            record.validate()?;
            Ok(record)
        })();
        match record {
            Ok(record) => new_records.push(record),
            Err(err) => failures.push(classify_failure("fastq.trim", &tool, &err)),
        }
    }

    records.extend(new_records.iter().cloned());

    let bench_path = bench_dir.join("bench.jsonl");
    for record in &new_records {
        append_jsonl(&bench_path, record).context("write bench.jsonl")?;
    }

    for record in &new_records {
        insert_fastq_trim_v1(&conn, record).context("insert bench sqlite")?;
    }

    check_fastq_trim_comparability(&records);
    write_trim_report(&bench_dir, &records, &failures, args.explain)?;
    if !failures.is_empty() {
        return Err(anyhow!("benchmark failures: {}", failures.len()));
    }
    Ok(())
}

#[allow(clippy::cast_precision_loss)]
fn check_fastq_trim_comparability(records: &[BenchmarkRecord<FastqTrimMetrics>]) {
    if records.len() <= 1 {
        return;
    }
    let first = &records[0];
    let mut reads_in = first.metrics.metrics.reads_in;
    let mut bases_in = first.metrics.metrics.bases_in;
    let mut mean_q_before = first.metrics.metrics.mean_q_before;

    for record in records.iter().skip(1) {
        if record.metrics.metrics.reads_in != reads_in {
            tracing::warn!(
                tool = record.context.tool,
                reads_in = record.metrics.metrics.reads_in,
                "reads_in differs from baseline"
            );
            reads_in = record.metrics.metrics.reads_in;
        }
        if record.metrics.metrics.bases_in != bases_in {
            tracing::warn!(
                tool = record.context.tool,
                bases_in = record.metrics.metrics.bases_in,
                "bases_in differs from baseline"
            );
            bases_in = record.metrics.metrics.bases_in;
        }
        if (record.metrics.metrics.mean_q_before - mean_q_before).abs() > 1e-6 {
            tracing::warn!(
                tool = record.context.tool,
                mean_q_before = record.metrics.metrics.mean_q_before,
                "mean_q_before differs from baseline"
            );
            mean_q_before = record.metrics.metrics.mean_q_before;
        }
        if record.metrics.metrics.reads_in > 0 {
            let loss = 1.0
                - (record.metrics.metrics.reads_out as f64
                    / record.metrics.metrics.reads_in as f64);
            if loss < -1e-6 {
                tracing::warn!(
                    tool = record.context.tool,
                    reads_in = record.metrics.metrics.reads_in,
                    reads_out = record.metrics.metrics.reads_out,
                    "reads_out exceeds reads_in"
                );
            }
        }
    }
}
