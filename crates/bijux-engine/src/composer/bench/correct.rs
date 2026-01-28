use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use crate::planner::load_registry;
use anyhow::{anyhow, Context, Result};
use bijux_bench::{
    append_jsonl, fetch_fastq_correct_v1, insert_fastq_correct_v1, BenchmarkContext,
    BenchmarkRecord, ExecutionMetrics, FastqCorrectMetrics, MetricSet,
};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use uuid::Uuid;

use crate::image_qa::ensure_image_qa_passed;
use crate::{
    bench_base_dir, bench_tools_dir, docker_rm, docker_stats_mb, hash_file_sha256,
    input_fastq_stats, output_fastq_stats, run_tool_container, validate_execution_outputs,
    SeqkitMetrics,
};

use super::failure::{classify_failure, BenchmarkFailure};
use super::helpers::{
    compute_run_id, find_first_fastq, normalize_correct_tool_list, params_hash,
    prepare_tool_run_dirs, resolve_image_for_run, write_execution_logs, write_metrics_json,
    ExecutionManifest,
};
use super::report::write_correct_report;

pub fn bench_fastq_correct(
    catalog: &std::collections::HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::bench::args::BenchFastqCorrectArgs,
) -> Result<()> {
    let tools = normalize_correct_tool_list(&args.tools)?;
    ensure_image_qa_passed("fastq.correct", &tools, platform, catalog)?;
    let bench_inputs = prepare_correct_bench(catalog, platform, runner_override, args)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_bench::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let mut records: Vec<BenchmarkRecord<FastqCorrectMetrics>> = Vec::new();
    let mut new_records: Vec<BenchmarkRecord<FastqCorrectMetrics>> = Vec::new();
    let mut failures: Vec<BenchmarkFailure> = Vec::new();

    for tool in tools {
        let spec = catalog
            .get(&tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let image_digest = spec
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .to_string();
        let cached = fetch_fastq_correct_v1(
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
        match run_correct_tool(catalog, platform, args, &bench_inputs, &tool) {
            Ok(record) => new_records.push(record),
            Err(err) => failures.push(classify_failure("fastq.correct", &tool, &err)),
        }
    }

    records.extend(new_records.iter().cloned());

    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    for record in &new_records {
        append_jsonl(&bench_path, record).context("write bench.jsonl")?;
    }

    for record in &new_records {
        insert_fastq_correct_v1(&conn, record).context("insert bench sqlite")?;
    }

    write_correct_report(&bench_inputs.bench_dir, &records, &failures, args.explain)?;
    if !failures.is_empty() {
        return Err(anyhow!("benchmark failures: {}", failures.len()));
    }
    Ok(())
}

struct CorrectBenchInputs {
    runner: RunnerKind,
    r1: PathBuf,
    r1_dir: PathBuf,
    input_hash: String,
    input_stats: SeqkitMetrics,
    bench_dir: PathBuf,
    tools_root: PathBuf,
}

fn prepare_correct_bench(
    catalog: &std::collections::HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::bench::args::BenchFastqCorrectArgs,
) -> Result<CorrectBenchInputs> {
    let runner = runner_override.unwrap_or(platform.runner);
    if runner != RunnerKind::Docker {
        return Err(anyhow!("benchmarking supports docker only for now"));
    }
    let bench_dir = bench_base_dir(&args.out, "correct", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "correct", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    println!(
        "planned tools: {}",
        normalize_correct_tool_list(&args.tools)?.join(", ")
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

    Ok(CorrectBenchInputs {
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
fn run_correct_tool(
    catalog: &std::collections::HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    args: &crate::bench::args::BenchFastqCorrectArgs,
    bench_inputs: &CorrectBenchInputs,
    tool: &str,
) -> Result<BenchmarkRecord<FastqCorrectMetrics>> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;

    println!("→ correct {tool}");
    let params = serde_json::json!({
        "sample_id": args.sample_id,
        "r1": bench_inputs.r1,
    });
    let param_hash = params_hash(&params).unwrap_or_else(|_| Uuid::new_v4().to_string());
    let image_digest = spec
        .digest
        .as_ref()
        .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
        .to_string();
    let run_id = compute_run_id(
        "fastq.correct",
        tool,
        &image_digest,
        &bench_inputs.input_hash,
        &param_hash,
    );
    let run_dirs = prepare_tool_run_dirs(&bench_inputs.tools_root, tool, &run_id)?;
    let out_dir = run_dirs.artifacts_dir.clone();
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

    let seqkit_spec = catalog
        .get("seqkit")
        .ok_or_else(|| anyhow!("seqkit missing from images.yaml"))?;
    let seqkit_image = resolve_image_for_run(seqkit_spec, platform)?;
    let out_fastq = if let Some(path) = execution.output_fastq {
        path
    } else {
        find_first_fastq(&out_dir)?
    };
    let output_stats = output_fastq_stats(&seqkit_image, &out_dir, &out_fastq)?;

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tool_manifest = registry
        .tool_by_id("fastq.correct", tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from manifests"))?;
    validate_execution_outputs(&tool_manifest.execution_contract, &out_dir)?;

    let metrics = FastqCorrectMetrics {
        reads_in: bench_inputs.input_stats.reads,
        reads_out: output_stats.reads,
        bases_in: bench_inputs.input_stats.bases,
        bases_out: output_stats.bases,
        mean_q_before: bench_inputs.input_stats.mean_q,
        mean_q_after: output_stats.mean_q,
        kmer_fix_rate: 0.0,
    };
    let metric_set = MetricSet::new(metrics);
    metric_set.validate()?;

    let manifest = ExecutionManifest {
        run_id: run_id.clone(),
        stage: "fastq.correct".to_string(),
        tool: tool.to_string(),
        tool_version: spec.version.clone(),
        image_digest: image_digest.clone(),
        command: execution.command.clone(),
        input_hashes: vec![bench_inputs.input_hash.clone()],
        input_files: vec![bench_inputs.r1.display().to_string()],
        output_dir: out_dir.display().to_string(),
        runner: bench_inputs.runner.to_string(),
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
        tool: tool.to_string(),
        tool_version: spec.version.clone(),
        image_digest,
        runner: bench_inputs.runner.to_string(),
        platform: platform.name.clone(),
        input_hash: bench_inputs.input_hash.clone(),
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
}
