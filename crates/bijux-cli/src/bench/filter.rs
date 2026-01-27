use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_bench::{
    append_jsonl, fetch_fastq_filter_v1, insert_fastq_filter_v1, BenchmarkContext, BenchmarkRecord,
    ExecutionMetrics, FastqFilterMetrics, StageMetricSchema,
};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use uuid::Uuid;

use crate::image_qa::ensure_image_qa_passed;
use crate::utils::{
    bench_base_dir, bench_tools_dir, docker_rm, docker_stats_mb, hash_file_sha256,
    input_fastq_stats, output_fastq_stats, run_tool_container, SeqkitMetrics,
};

use super::helpers::{normalize_filter_tool_list, resolve_image_for_run, ExecutionManifest};
use super::report::write_filter_report;

pub fn bench_fastq_filter(
    catalog: &std::collections::HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::cli::BenchFastqFilterArgs,
) -> Result<()> {
    let tools = normalize_filter_tool_list(&args.tools)?;
    ensure_image_qa_passed("fastq.filter", &tools, platform, catalog)?;
    let bench_inputs = prepare_filter_bench(catalog, platform, runner_override, args)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_bench::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let mut records: Vec<BenchmarkRecord<FastqFilterMetrics>> = Vec::new();
    let mut new_records: Vec<BenchmarkRecord<FastqFilterMetrics>> = Vec::new();

    for tool in tools {
        let spec = catalog
            .get(&tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let image_digest = spec
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .to_string();
        let cached = fetch_fastq_filter_v1(
            &conn,
            &tool,
            &spec.version,
            &image_digest,
            &bench_inputs.input_hash,
        );
        if let Ok(Some(record)) = cached {
            records.push(record);
            continue;
        }
        let record = run_filter_tool(catalog, platform, args, &bench_inputs, &tool)?;
        new_records.push(record);
    }

    records.extend(new_records.iter().cloned());

    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    for record in &new_records {
        append_jsonl(&bench_path, record).context("write bench.jsonl")?;
    }

    for record in &new_records {
        insert_fastq_filter_v1(&conn, record).context("insert bench sqlite")?;
    }

    check_fastq_filter_comparability(&records);
    write_filter_report(&bench_inputs.bench_dir, &records, args.explain)?;
    Ok(())
}

struct FilterBenchInputs {
    runner: RunnerKind,
    r1: PathBuf,
    r1_dir: PathBuf,
    input_hash: String,
    input_stats: SeqkitMetrics,
    bench_dir: PathBuf,
    tools_root: PathBuf,
}

fn prepare_filter_bench(
    catalog: &std::collections::HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::cli::BenchFastqFilterArgs,
) -> Result<FilterBenchInputs> {
    let runner = runner_override.unwrap_or(platform.runner);
    if runner != RunnerKind::Docker {
        return Err(anyhow!("benchmarking supports docker only for now"));
    }
    let bench_dir = bench_base_dir(&args.out, "filter", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "filter", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    println!(
        "planned tools: {}",
        normalize_filter_tool_list(&args.tools)?.join(", ")
    );

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

    Ok(FilterBenchInputs {
        runner,
        r1,
        r1_dir,
        input_hash,
        input_stats,
        bench_dir,
        tools_root,
    })
}

#[allow(clippy::too_many_lines)]
fn run_filter_tool(
    catalog: &std::collections::HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    args: &crate::cli::BenchFastqFilterArgs,
    bench_inputs: &FilterBenchInputs,
    tool: &str,
) -> Result<BenchmarkRecord<FastqFilterMetrics>> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;

    println!("→ filter {tool}");
    let tool_dir = bench_inputs.tools_root.join(tool);
    let out_dir = tool_dir.join("out");
    fs::create_dir_all(&out_dir).context("create tool output dir")?;
    let start = Instant::now();
    let container_name = format!("bijux-bench-{}-{}", args.sample_id, Uuid::new_v4());
    let execution = run_tool_container(
        tool,
        &image,
        &bench_inputs.r1_dir,
        &bench_inputs.r1,
        &out_dir,
        &container_name,
    )?;
    let runtime_s = start.elapsed().as_secs_f64();
    let memory_mb = docker_stats_mb(&container_name)?;
    docker_rm(&container_name)?;

    let out_fastq = execution
        .output_fastq
        .as_ref()
        .ok_or_else(|| anyhow!("output fastq missing"))?;
    let out_fastq = if out_fastq.exists() {
        out_fastq.clone()
    } else {
        let alt = out_fastq.with_extension("");
        if alt.exists() {
            alt
        } else {
            return Err(anyhow!("output fastq missing"));
        }
    };
    let output_stats = output_fastq_stats(
        &resolve_image_for_run(
            catalog
                .get("seqkit")
                .ok_or_else(|| anyhow!("seqkit missing from images.yaml"))?,
            platform,
        )?,
        &out_dir,
        &out_fastq,
    )?;

    let reads_in = bench_inputs.input_stats.reads;
    let reads_out = output_stats.reads;
    let reads_dropped = reads_in.saturating_sub(reads_out);
    let metrics = FastqFilterMetrics {
        reads_in,
        reads_out,
        reads_dropped,
        mean_q_before: bench_inputs.input_stats.mean_q,
        mean_q_after: output_stats.mean_q,
    };
    metrics.validate()?;

    let image_digest = spec
        .digest
        .as_ref()
        .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
        .to_string();
    let manifest = ExecutionManifest {
        tool: tool.to_string(),
        tool_version: spec.version.clone(),
        image_digest: image_digest.clone(),
        command: execution.command.clone(),
        input_hashes: vec![bench_inputs.input_hash.clone()],
        input_files: vec![bench_inputs.r1.display().to_string()],
        runner: bench_inputs.runner.to_string(),
        platform: platform.name.clone(),
        arch: platform.arch.clone(),
    };
    let manifest_path = tool_dir.join("execution_manifest.json");
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)
        .context("write execution manifest")?;
    let context = BenchmarkContext {
        tool: tool.to_string(),
        tool_version: spec.version.clone(),
        image_digest,
        runner: bench_inputs.runner.to_string(),
        platform: platform.name.clone(),
        input_hash: bench_inputs.input_hash.clone(),
        parameters: serde_json::json!({
            "sample_id": args.sample_id,
            "r1": bench_inputs.r1,
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
    if execution.exit_code != 0 {
        return Err(anyhow!(
            "tool {tool} failed with status {} (stdout: {}, stderr: {})",
            execution.exit_code,
            execution.stdout.trim(),
            execution.stderr.trim()
        ));
    }
    Ok(record)
}

fn check_fastq_filter_comparability(records: &[BenchmarkRecord<FastqFilterMetrics>]) {
    if records.len() <= 1 {
        return;
    }
    let first = &records[0];
    let mut reads_in = first.metrics.reads_in;
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
        if (record.metrics.mean_q_before - mean_q_before).abs() > 1e-6 {
            tracing::warn!(
                tool = record.context.tool,
                mean_q_before = record.metrics.mean_q_before,
                "mean_q_before differs from baseline"
            );
            mean_q_before = record.metrics.mean_q_before;
        }
        if record.metrics.reads_out > record.metrics.reads_in {
            tracing::warn!(
                tool = record.context.tool,
                reads_in = record.metrics.reads_in,
                reads_out = record.metrics.reads_out,
                "reads_out exceeds reads_in"
            );
        }
    }
}
