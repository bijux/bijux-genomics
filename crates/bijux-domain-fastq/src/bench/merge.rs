use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_bench::{
    append_jsonl, fetch_fastq_merge_v1, insert_fastq_merge_v1, BenchmarkContext, BenchmarkRecord,
    ExecutionMetrics, FastqMergeMetrics, MetricSet,
};
use bijux_engine::api::load_registry;
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use tracing::warn;
use uuid::Uuid;

use crate::image_qa::ensure_image_qa_passed;
use bijux_engine::api::validate_execution_outputs;
use bijux_engine::api::{bench_base_dir, bench_tools_dir};
use bijux_engine::api::{docker_rm, docker_stats_mb, run_merge_container};
use bijux_engine::api::{hash_file_sha256, input_fastq_stats, output_fastq_stats, SeqkitMetrics};

use super::analyze::failure::{classify_failure, BenchmarkFailure};
use super::analyze::report::write_merge_report;
use super::helpers::{
    compute_run_id, normalize_merge_tool_list, params_hash, prepare_tool_run_dirs, ratio_u64,
    resolve_image_for_run, write_execution_logs, write_explain_md, write_metrics_json,
    ExecutionManifest,
};

/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning, execution, or metric recording fails.
pub fn bench_fastq_merge<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::bench::args::BenchFastqMergeArgs,
) -> Result<()> {
    let tools = normalize_merge_tool_list(&args.tools)?;
    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let bench_inputs = prepare_merge_bench(catalog, platform, runner_override, args)?;
    let selected = tools.clone();
    let all_tools: Vec<String> = registry
        .tools_for_stage("fastq.merge")
        .iter()
        .map(|tool| tool.tool_id.clone())
        .collect();
    let excluded: Vec<String> = all_tools
        .into_iter()
        .filter(|tool| !selected.contains(tool))
        .collect();
    write_explain_md(
        &bench_inputs.bench_dir,
        "fastq.merge",
        &selected,
        &excluded,
        None,
    )?;
    ensure_image_qa_passed("fastq.merge", &tools, platform, catalog)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_bench::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let mut records: Vec<BenchmarkRecord<FastqMergeMetrics>> = Vec::new();
    let mut new_records: Vec<BenchmarkRecord<FastqMergeMetrics>> = Vec::new();
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
        let cached = fetch_fastq_merge_v1(
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
        match run_merge_tool(catalog, platform, args, &bench_inputs, &tool) {
            Ok(record) => new_records.push(record),
            Err(err) => failures.push(classify_failure("fastq.merge", &tool, &err)),
        }
    }

    records.extend(new_records.iter().cloned());

    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    for record in &new_records {
        append_jsonl(&bench_path, record).context("write bench.jsonl")?;
    }

    for record in &new_records {
        insert_fastq_merge_v1(&conn, record).context("insert bench sqlite")?;
    }

    check_fastq_merge_comparability(&records);
    write_merge_report(&bench_inputs.bench_dir, &records, &failures, args.explain)?;
    if !failures.is_empty() {
        return Err(anyhow!("benchmark failures: {}", failures.len()));
    }
    Ok(())
}

struct MergeBenchInputs {
    runner: RunnerKind,
    r1: PathBuf,
    r2: PathBuf,
    r1_dir: PathBuf,
    input_hash: String,
    input_hash_r1: String,
    input_hash_r2: String,
    input_stats_r1: SeqkitMetrics,
    input_stats_r2: SeqkitMetrics,
    bench_dir: PathBuf,
    tools_root: PathBuf,
}

fn prepare_merge_bench<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::bench::args::BenchFastqMergeArgs,
) -> Result<MergeBenchInputs> {
    let runner = runner_override.unwrap_or(platform.runner);
    if runner != RunnerKind::Docker {
        return Err(anyhow!("benchmarking supports docker only for now"));
    }
    let bench_dir = bench_base_dir(&args.out, "merge", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "merge", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    println!(
        "planned tools: {}",
        normalize_merge_tool_list(&args.tools)?.join(", ")
    );

    let r1 = args.r1.canonicalize().context("resolve r1 path")?;
    let r2 = args.r2.canonicalize().context("resolve r2 path")?;
    let r1_dir = r1
        .parent()
        .ok_or_else(|| anyhow!("r1 has no parent"))?
        .to_path_buf();

    let seqkit_spec = catalog
        .get("seqkit")
        .ok_or_else(|| anyhow!("seqkit missing from images.yaml"))?;
    let seqkit_image = resolve_image_for_run(seqkit_spec, platform)?;

    let input_hash_r1 = hash_file_sha256(&r1)?;
    let input_hash_r2 = hash_file_sha256(&r2)?;
    let input_hash = format!("{input_hash_r1},{input_hash_r2}");
    let input_stats_r1 = input_fastq_stats(&seqkit_image, &r1_dir, &r1)?;
    let input_stats_r2 = input_fastq_stats(&seqkit_image, &r1_dir, &r2)?;

    Ok(MergeBenchInputs {
        runner,
        r1,
        r2,
        r1_dir,
        input_hash,
        input_hash_r1,
        input_hash_r2,
        input_stats_r1,
        input_stats_r2,
        bench_dir,
        tools_root,
    })
}

#[allow(clippy::too_many_lines)]
fn run_merge_tool<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &crate::bench::args::BenchFastqMergeArgs,
    bench_inputs: &MergeBenchInputs,
    tool: &str,
) -> Result<BenchmarkRecord<FastqMergeMetrics>> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;

    println!("→ merge {tool}");
    let params = serde_json::json!({
        "sample_id": args.sample_id,
        "r1": bench_inputs.r1,
        "r2": bench_inputs.r2,
    });
    let param_hash = params_hash(&params).unwrap_or_else(|_| Uuid::new_v4().to_string());
    let image_digest = spec
        .digest
        .as_ref()
        .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
        .to_string();
    let run_id = compute_run_id(
        "fastq.merge",
        tool,
        &image_digest,
        &bench_inputs.input_hash,
        &param_hash,
    );
    let run_dirs = prepare_tool_run_dirs(&bench_inputs.tools_root, tool, &run_id)?;
    let out_dir = run_dirs.artifacts_dir.clone();
    let start = Instant::now();
    let container_name = format!("bijux-bench-{}-{}", args.sample_id, Uuid::new_v4());
    let execution = run_merge_container(
        tool,
        &image,
        &bench_inputs.r1_dir,
        &bench_inputs.r1,
        &bench_inputs.r2,
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

    let merged_stats = output_fastq_stats(&seqkit_image, &out_dir, &execution.merged_fastq)?;
    let unmerged_r1_stats = output_fastq_stats(&seqkit_image, &out_dir, &execution.unmerged_r1)?;
    let unmerged_r2_stats = output_fastq_stats(&seqkit_image, &out_dir, &execution.unmerged_r2)?;

    let reads_r1 = bench_inputs.input_stats_r1.reads;
    let reads_r2 = bench_inputs.input_stats_r2.reads;
    let reads_merged = merged_stats.reads;
    let reads_unmerged = unmerged_r1_stats.reads.min(unmerged_r2_stats.reads);
    if unmerged_r1_stats.reads != unmerged_r2_stats.reads {
        warn!(
            tool = tool,
            unmerged_r1 = unmerged_r1_stats.reads,
            unmerged_r2 = unmerged_r2_stats.reads,
            "unmerged read counts differ between r1 and r2"
        );
    }
    let min_reads = reads_r1.min(reads_r2);
    let merge_rate = if min_reads > 0 {
        ratio_u64(reads_merged, min_reads)
    } else {
        0.0
    };

    let metrics = FastqMergeMetrics {
        reads_r1,
        reads_r2,
        reads_merged,
        reads_unmerged,
        merge_rate,
    };
    let metric_set = MetricSet::new(metrics);
    metric_set.validate()?;

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tool_manifest = registry
        .tool_by_id("fastq.merge", tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from manifests"))?;
    validate_execution_outputs(&tool_manifest.execution_contract, &out_dir)?;
    let manifest = ExecutionManifest {
        run_id: run_id.clone(),
        stage: "fastq.merge".to_string(),
        tool: tool.to_string(),
        tool_version: spec.version.clone(),
        image_digest: image_digest.clone(),
        command: execution.command.clone(),
        input_hashes: vec![
            bench_inputs.input_hash_r1.clone(),
            bench_inputs.input_hash_r2.clone(),
        ],
        input_files: vec![
            bench_inputs.r1.display().to_string(),
            bench_inputs.r2.display().to_string(),
        ],
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

fn check_fastq_merge_comparability(records: &[BenchmarkRecord<FastqMergeMetrics>]) {
    if records.len() <= 1 {
        return;
    }
    let first = &records[0];
    let mut reads_r1 = first.metrics.metrics.reads_r1;
    let mut reads_r2 = first.metrics.metrics.reads_r2;

    for record in records.iter().skip(1) {
        if record.metrics.metrics.reads_r1 != reads_r1 {
            warn!(
                tool = record.context.tool,
                reads_r1 = record.metrics.metrics.reads_r1,
                "reads_r1 differs from baseline"
            );
            reads_r1 = record.metrics.metrics.reads_r1;
        }
        if record.metrics.metrics.reads_r2 != reads_r2 {
            warn!(
                tool = record.context.tool,
                reads_r2 = record.metrics.metrics.reads_r2,
                "reads_r2 differs from baseline"
            );
            reads_r2 = record.metrics.metrics.reads_r2;
        }
        let min_reads = record
            .metrics
            .metrics
            .reads_r1
            .min(record.metrics.metrics.reads_r2);
        if record.metrics.metrics.reads_merged > min_reads {
            warn!(
                tool = record.context.tool,
                reads_merged = record.metrics.metrics.reads_merged,
                min_reads = min_reads,
                "merge should not exceed input pairs"
            );
        }
    }
}
