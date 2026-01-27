use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_bench::{
    append_jsonl, fetch_fastq_qc2_v1, insert_fastq_qc2_v1, BenchmarkContext, BenchmarkRecord,
    ExecutionMetrics, FastqQc2Metrics, StageMetricSchema,
};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use uuid::Uuid;

use crate::image_qa::ensure_image_qa_passed;
use crate::utils::{
    bench_base_dir, bench_tools_dir, docker_rm, docker_stats_mb, hash_file_sha256,
    input_fastq_stats, run_validate_container, SeqkitMetrics,
};

use super::helpers::{normalize_qc2_tool_list, resolve_image_for_run, ExecutionManifest};
use super::report::write_qc2_report;

pub fn bench_fastq_qc2(
    catalog: &std::collections::HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::cli::BenchFastqQc2Args,
) -> Result<()> {
    let tools = normalize_qc2_tool_list(&args.tools)?;
    ensure_image_qa_passed("fastq.qc2", &tools, platform, catalog)?;
    let bench_inputs = prepare_qc2_bench(catalog, platform, runner_override, args)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_bench::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let mut records: Vec<BenchmarkRecord<FastqQc2Metrics>> = Vec::new();
    let mut new_records: Vec<BenchmarkRecord<FastqQc2Metrics>> = Vec::new();

    for tool in tools {
        let spec = catalog
            .get(&tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let image_digest = spec
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .to_string();
        let cached = fetch_fastq_qc2_v1(
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
        let record = run_qc2_tool(catalog, platform, args, &bench_inputs, &tool)?;
        new_records.push(record);
    }

    records.extend(new_records.iter().cloned());

    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    for record in &new_records {
        append_jsonl(&bench_path, record).context("write bench.jsonl")?;
    }

    for record in &new_records {
        insert_fastq_qc2_v1(&conn, record).context("insert bench sqlite")?;
    }

    write_qc2_report(&bench_inputs.bench_dir, &records, args.explain)?;
    Ok(())
}

struct Qc2BenchInputs {
    runner: RunnerKind,
    r1: PathBuf,
    r1_dir: PathBuf,
    input_hash: String,
    input_stats: SeqkitMetrics,
    bench_dir: PathBuf,
    tools_root: PathBuf,
}

fn prepare_qc2_bench(
    catalog: &std::collections::HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::cli::BenchFastqQc2Args,
) -> Result<Qc2BenchInputs> {
    let runner = runner_override.unwrap_or(platform.runner);
    if runner != RunnerKind::Docker {
        return Err(anyhow!("benchmarking supports docker only for now"));
    }
    let bench_dir = bench_base_dir(&args.out, "qc2", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "qc2", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    println!(
        "planned tools: {}",
        normalize_qc2_tool_list(&args.tools)?.join(", ")
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

    Ok(Qc2BenchInputs {
        runner,
        r1,
        r1_dir,
        input_hash,
        input_stats,
        bench_dir,
        tools_root,
    })
}

fn run_qc2_tool(
    catalog: &std::collections::HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    args: &crate::cli::BenchFastqQc2Args,
    bench_inputs: &Qc2BenchInputs,
    tool: &str,
) -> Result<BenchmarkRecord<FastqQc2Metrics>> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;

    println!("→ qc2 {tool}");
    let tool_dir = bench_inputs.tools_root.join(tool);
    let out_dir = tool_dir.join("out");
    fs::create_dir_all(&out_dir).context("create tool output dir")?;
    let start = Instant::now();
    let container_name = format!("bijux-bench-{}-{}", args.sample_id, Uuid::new_v4());
    let execution = run_validate_container(
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

    let metrics = FastqQc2Metrics {
        reads_in: bench_inputs.input_stats.reads,
        bases_in: bench_inputs.input_stats.bases,
        mean_q: bench_inputs.input_stats.mean_q,
        contamination_rate: 0.0,
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
    Ok(record)
}
