use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_analyze::{
    append_jsonl, fetch_fastq_stats_v1, insert_fastq_stats_v1, BenchmarkContext, BenchmarkRecord,
    FastqStatsMetrics, LengthHistogramBin, MetricSet,
};
use bijux_engine::api::load_registry;
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use bijux_measure::ExecutionMetrics;
use uuid::Uuid;

use crate::domain::{infer_input_kind, inspect_headers, log_header_warnings, preflight_stage};
use crate::image_qa::ensure_image_qa_passed;
use bijux_engine::api::validate_execution_outputs;
use bijux_engine::api::{bench_base_dir, bench_tools_dir};
use bijux_engine::api::{docker_rm, docker_stats_mb, run_validate_container};
use bijux_engine::api::{hash_file_sha256, input_fastq_stats, length_histogram, SeqkitMetrics};

use super::analyze::failure::{classify_failure, BenchmarkFailure};
use super::analyze::report::write_stats_report;
use super::helpers::{
    compute_run_id, normalize_stats_tool_list, params_hash, prepare_tool_run_dirs,
    resolve_image_for_run, write_execution_logs, write_explain_md, write_metrics_json,
    ExecutionManifest,
};

/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning, execution, or metric recording fails.
pub fn bench_fastq_stats<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::bench::args::BenchFastqStatsArgs,
) -> Result<()> {
    let tools = normalize_stats_tool_list(&args.tools)?;
    let input_kind = infer_input_kind(None);
    preflight_stage("fastq.stats", input_kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings("fastq.stats", &header);
    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let bench_inputs = prepare_stats_bench(catalog, platform, runner_override, args)?;
    let selected = tools.clone();
    let all_tools: Vec<String> = registry
        .tools_for_stage("fastq.stats")
        .iter()
        .map(|tool| tool.tool_id.clone())
        .collect();
    let excluded: Vec<String> = all_tools
        .into_iter()
        .filter(|tool| !selected.contains(tool))
        .collect();
    write_explain_md(
        &bench_inputs.bench_dir,
        "fastq.stats",
        &selected,
        &excluded,
        None,
    )?;
    ensure_image_qa_passed("fastq.stats", &tools, platform, catalog)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let mut records: Vec<BenchmarkRecord<FastqStatsMetrics>> = Vec::new();
    let mut new_records: Vec<BenchmarkRecord<FastqStatsMetrics>> = Vec::new();
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
        let cached = fetch_fastq_stats_v1(
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
        match run_stats_tool(catalog, platform, args, &bench_inputs, &tool) {
            Ok(record) => new_records.push(record),
            Err(err) => failures.push(classify_failure("fastq.stats", &tool, &err)),
        }
    }

    records.extend(new_records.iter().cloned());

    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    for record in &new_records {
        append_jsonl(&bench_path, record).context("write bench.jsonl")?;
    }

    for record in &new_records {
        insert_fastq_stats_v1(&conn, record).context("insert bench sqlite")?;
    }

    write_stats_report(&bench_inputs.bench_dir, &records, &failures, args.explain)?;
    if !failures.is_empty() {
        return Err(anyhow!("benchmark failures: {}", failures.len()));
    }
    Ok(())
}

struct StatsBenchInputs {
    runner: RunnerKind,
    r1: PathBuf,
    r1_dir: PathBuf,
    input_hash: String,
    input_stats: SeqkitMetrics,
    length_hist: Vec<LengthHistogramBin>,
    bench_dir: PathBuf,
    tools_root: PathBuf,
}

fn prepare_stats_bench<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::bench::args::BenchFastqStatsArgs,
) -> Result<StatsBenchInputs> {
    let runner = runner_override.unwrap_or(platform.runner);
    if runner != RunnerKind::Docker {
        return Err(anyhow!("benchmarking supports docker only for now"));
    }
    let bench_dir = bench_base_dir(&args.out, "stats", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "stats", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    println!(
        "planned tools: {}",
        normalize_stats_tool_list(&args.tools)?.join(", ")
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
    let length_hist = length_histogram(&seqkit_image, &r1_dir, &r1)?
        .into_iter()
        .map(|(length, count)| LengthHistogramBin { length, count })
        .collect();

    Ok(StatsBenchInputs {
        runner,
        r1,
        r1_dir,
        input_hash,
        input_stats,
        length_hist,
        bench_dir,
        tools_root,
    })
}

fn run_stats_tool<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &crate::bench::args::BenchFastqStatsArgs,
    bench_inputs: &StatsBenchInputs,
    tool: &str,
) -> Result<BenchmarkRecord<FastqStatsMetrics>> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;

    println!("→ stats {tool}");
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
        "fastq.stats",
        tool,
        &image_digest,
        &bench_inputs.input_hash,
        &param_hash,
    );
    let run_dirs = prepare_tool_run_dirs(&bench_inputs.tools_root, tool, &run_id)?;
    let out_dir = run_dirs.artifacts_dir.clone();
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

    let metrics = FastqStatsMetrics {
        reads_total: bench_inputs.input_stats.reads,
        bases_total: bench_inputs.input_stats.bases,
        mean_q: bench_inputs.input_stats.mean_q,
        gc_percent: bench_inputs.input_stats.gc_percent,
        length_histogram: bench_inputs.length_hist.clone(),
    };
    let metric_set = MetricSet::new(metrics);
    metric_set.validate()?;

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tool_manifest = registry
        .tool_by_id("fastq.stats", tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from manifests"))?;
    validate_execution_outputs(&tool_manifest.execution_contract, &out_dir)?;
    let manifest = ExecutionManifest {
        run_id: run_id.clone(),
        stage: "fastq.stats".to_string(),
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
