use std::fs;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_bench::{
    append_jsonl, fetch_fastq_trim_v1, insert_fastq_trim_v1, BenchmarkContext, BenchmarkRecord,
    ExecutionMetrics, FastqTrimMetrics, StageMetricSchema,
};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use uuid::Uuid;

use crate::image_qa::ensure_image_qa_passed;
use crate::utils::{
    bench_base_dir, bench_tools_dir, docker_rm, docker_stats_mb, hash_file_sha256,
    input_fastq_stats, output_fastq_stats, run_tool_container,
};

use super::helpers::{normalize_tool_list, resolve_image_for_run, ExecutionManifest};
use super::report::write_trim_report;

#[allow(clippy::too_many_lines)]
pub fn bench_fastq_trim(
    catalog: &std::collections::HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::cli::BenchFastqTrimArgs,
) -> Result<()> {
    let runner = runner_override.unwrap_or(platform.runner);
    if runner != RunnerKind::Docker {
        return Err(anyhow!("benchmarking supports docker only for now"));
    }
    let tools = normalize_tool_list(&args.tools)?;
    let bench_dir = bench_base_dir(&args.out, "trim", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "trim", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    println!("planned tools: {}", tools.join(", "));

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
    let conn = bijux_bench::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let mut records: Vec<BenchmarkRecord<FastqTrimMetrics>> = Vec::new();
    let mut new_records: Vec<BenchmarkRecord<FastqTrimMetrics>> = Vec::new();

    for tool in tools {
        let spec = catalog
            .get(&tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let image = resolve_image_for_run(spec, platform)?;
        let image_digest = spec
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .to_string();
        let cached = fetch_fastq_trim_v1(&conn, &tool, &spec.version, &image_digest, &input_hash);
        if let Ok(Some(record)) = cached {
            records.push(record);
            continue;
        }

        let tool_dir = tools_root.join(&tool);
        let out_dir = tool_dir.join("out");
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let start = Instant::now();
        let container_name = format!("bijux-bench-{}-{}", args.sample_id, Uuid::new_v4());
        let execution = run_tool_container(&tool, &image, &r1_dir, &r1, &out_dir, &container_name)?;
        let runtime_s = start.elapsed().as_secs_f64();
        let memory_mb = docker_stats_mb(&container_name)?;
        docker_rm(&container_name)?;

        let out_fastq = execution
            .output_fastq
            .as_ref()
            .ok_or_else(|| anyhow!("output fastq missing"))?;
        let output_stats = output_fastq_stats(&seqkit_image, &out_dir, out_fastq)?;

        let metrics = FastqTrimMetrics {
            reads_in: input_stats.reads,
            reads_out: output_stats.reads,
            bases_in: input_stats.bases,
            bases_out: output_stats.bases,
            mean_q_before: input_stats.mean_q,
            mean_q_after: output_stats.mean_q,
        };
        metrics.validate()?;

        let manifest = ExecutionManifest {
            tool: tool.clone(),
            tool_version: spec.version.clone(),
            image_digest: image_digest.clone(),
            command: execution.command.clone(),
            input_hashes: vec![input_hash.clone()],
            input_files: vec![r1.display().to_string()],
            runner: runner.to_string(),
            platform: platform.name.clone(),
            arch: platform.arch.clone(),
        };
        let manifest_path = tool_dir.join("execution_manifest.json");
        fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)
            .context("write execution manifest")?;

        let context = BenchmarkContext {
            tool: tool.clone(),
            tool_version: spec.version.clone(),
            image_digest,
            runner: runner.to_string(),
            platform: platform.name.clone(),
            input_hash: input_hash.clone(),
            parameters: serde_json::json!({
                "sample_id": args.sample_id,
                "r1": r1,
            }),
        };
        let execution_metrics = ExecutionMetrics {
            runtime_s,
            memory_mb,
            exit_code: execution.exit_code,
        };
        let record = BenchmarkRecord {
            context,
            execution: execution_metrics,
            metrics,
        };
        record.validate()?;
        new_records.push(record);
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
    write_trim_report(&bench_dir, &records, args.explain)?;
    Ok(())
}

#[allow(clippy::cast_precision_loss)]
fn check_fastq_trim_comparability(records: &[BenchmarkRecord<FastqTrimMetrics>]) {
    if records.len() <= 1 {
        return;
    }
    let first = &records[0];
    let mut reads_in = first.metrics.reads_in;
    let mut bases_in = first.metrics.bases_in;
    let mut mean_q_before = first.metrics.mean_q_before;

    for record in records.iter().skip(1) {
        if record.metrics.reads_in != reads_in {
            tracing::warn!(
                tool = record.context.tool,
                reads_in = record.metrics.reads_in,
                "reads_in differs from baseline"
            );
            reads_in = record.metrics.reads_in;
        }
        if record.metrics.bases_in != bases_in {
            tracing::warn!(
                tool = record.context.tool,
                bases_in = record.metrics.bases_in,
                "bases_in differs from baseline"
            );
            bases_in = record.metrics.bases_in;
        }
        if (record.metrics.mean_q_before - mean_q_before).abs() > 1e-6 {
            tracing::warn!(
                tool = record.context.tool,
                mean_q_before = record.metrics.mean_q_before,
                "mean_q_before differs from baseline"
            );
            mean_q_before = record.metrics.mean_q_before;
        }
        if record.metrics.reads_in > 0 {
            let loss = 1.0 - (record.metrics.reads_out as f64 / record.metrics.reads_in as f64);
            if loss < -1e-6 {
                tracing::warn!(
                    tool = record.context.tool,
                    reads_in = record.metrics.reads_in,
                    reads_out = record.metrics.reads_out,
                    "reads_out exceeds reads_in"
                );
            }
        }
    }
}
