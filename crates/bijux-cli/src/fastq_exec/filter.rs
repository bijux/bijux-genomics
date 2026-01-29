use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_analyze::{
    append_jsonl, fetch_fastq_filter_v2, insert_fastq_filter_v2, metric_set, BenchmarkContext,
    BenchmarkRecord, FastqDeltaMetrics, FastqFilterMetrics,
};
use bijux_core::measure::ExecutionMetrics;
use bijux_engine::api::{ensure_bench_runner, load_registry};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use uuid::Uuid;

use bijux_domain_fastq::{
    contract_for_stage, inspect_headers, log_header_warnings, normalize_outputs, preflight_stage,
    FastqArtifact,
};
use bijux_engine::api::validate_execution_outputs;
use bijux_engine::api::{bench_base_dir, bench_tools_dir};
use bijux_engine::api::{cleanup_execution, execution_memory_mb, run_tool_execution};
use bijux_engine::api::{hash_file_sha256, input_fastq_stats, output_fastq_stats, SeqkitMetrics};
use bijux_environment::image_qa::{ensure_image_qa_passed, ensure_tool_qa_passed};

use crate::fastq_exec::helpers::{
    compute_run_id, normalize_filter_tool_list, params_hash, prepare_tool_run_dirs,
    resolve_image_for_run, write_execution_logs, write_explain_md, write_explain_plan_json,
    write_metrics_json, write_retention_report_placeholder, write_run_manifest, ExecutionManifest,
};
use crate::fastq_exec::helpers::{filter_tools_by_role, BenchOutcome};
use bijux_domain_fastq::RawFailure;

/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning, execution, or metric recording fails.
pub fn bench_fastq_filter<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_domain_fastq::args::BenchFastqFilterArgs,
) -> Result<BenchOutcome<FastqFilterMetrics>> {
    let tools = normalize_filter_tool_list(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage("fastq.filter", artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings("fastq.filter", &header);
    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.filter", &tools, &registry, false)?;
    let bench_inputs = prepare_filter_bench(catalog, platform, runner_override, args)?;
    let selected = tools.clone();
    let all_tools: Vec<String> = registry
        .tools_for_stage("fastq.filter")
        .iter()
        .map(|tool| tool.tool_id.clone())
        .collect();
    let excluded: Vec<String> = all_tools
        .into_iter()
        .filter(|tool| !selected.contains(tool))
        .collect();
    write_explain_md(
        &bench_inputs.bench_dir,
        "fastq.filter",
        &selected,
        &excluded,
        None,
    )?;
    write_explain_plan_json(
        &bench_inputs.bench_dir,
        "fastq.filter",
        &selected,
        &registry,
        None,
    )?;
    ensure_image_qa_passed("fastq.filter", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.filter", &tools, platform, catalog)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let mut records: Vec<BenchmarkRecord<FastqFilterMetrics>> = Vec::new();
    let mut new_records: Vec<BenchmarkRecord<FastqFilterMetrics>> = Vec::new();
    let mut failures: Vec<RawFailure> = Vec::new();

    for tool in tools {
        let spec = catalog
            .get(&tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let image_digest = spec
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .to_string();
        let cached = fetch_fastq_filter_v2(
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
        match run_filter_tool(catalog, platform, args, &bench_inputs, &tool) {
            Ok(record) => new_records.push(record),
            Err(err) => failures.push(RawFailure {
                stage: "fastq.filter".to_string(),
                tool: tool.to_string(),
                reason: err.to_string(),
            }),
        }
    }

    records.extend(new_records.iter().cloned());

    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    for record in &new_records {
        append_jsonl(&bench_path, record).context("write bench.jsonl")?;
    }

    for record in &new_records {
        insert_fastq_filter_v2(&conn, record).context("insert bench sqlite")?;
    }

    check_fastq_filter_comparability(&records);
    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: bench_inputs.bench_dir,
        explain: args.explain,
    })
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

fn prepare_filter_bench<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_domain_fastq::args::BenchFastqFilterArgs,
) -> Result<FilterBenchInputs> {
    let runner = ensure_bench_runner(platform, runner_override)?;
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
fn run_filter_tool<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_domain_fastq::args::BenchFastqFilterArgs,
    bench_inputs: &FilterBenchInputs,
    tool: &str,
) -> Result<BenchmarkRecord<FastqFilterMetrics>> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;

    println!("→ filter {tool}");
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
        "fastq.filter",
        tool,
        &image_digest,
        &bench_inputs.input_hash,
        &param_hash,
    );
    let run_dirs = prepare_tool_run_dirs(&bench_inputs.tools_root, tool, &run_id)?;
    let out_dir = run_dirs.artifacts_dir.clone();
    let start = Instant::now();
    let container_name = format!("bijux-bench-{}-{}", args.sample_id, Uuid::new_v4());
    let execution = run_tool_execution(
        tool,
        &image,
        &bench_inputs.r1_dir,
        &bench_inputs.r1,
        &out_dir,
        &container_name,
    )?;
    let runtime_s = start.elapsed().as_secs_f64();
    let memory_mb = execution_memory_mb(&container_name)?;
    cleanup_execution(&container_name)?;

    let contract = contract_for_stage("fastq.filter")
        .ok_or_else(|| anyhow!("missing fastq.filter contract"))?;
    let normalized = normalize_outputs("fastq.filter", &out_dir, contract.output_kind)?;
    let out_fastq = normalized
        .r1
        .as_ref()
        .ok_or_else(|| anyhow!("output fastq missing"))?;
    let output_stats = output_fastq_stats(
        &resolve_image_for_run(
            catalog
                .get("seqkit")
                .ok_or_else(|| anyhow!("seqkit missing from images.yaml"))?,
            platform,
        )?,
        &out_dir,
        out_fastq,
    )?;

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tool_manifest = registry
        .tool_by_id("fastq.filter", tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from manifests"))?;
    validate_execution_outputs(&tool_manifest.execution_contract, &out_dir)?;

    let reads_in = bench_inputs.input_stats.reads;
    let reads_out = output_stats.reads;
    let reads_dropped = reads_in.saturating_sub(reads_out);
    let delta = bijux_domain_fastq::compute_delta(bench_inputs.input_stats, output_stats);
    let metrics = FastqFilterMetrics {
        reads_in,
        reads_out,
        reads_dropped,
        mean_q_before: bench_inputs.input_stats.mean_q,
        mean_q_after: output_stats.mean_q,
        delta_metrics: FastqDeltaMetrics {
            read_retention: delta.read_retention,
            base_retention: delta.base_retention,
            mean_q_delta: delta.delta_mean_q,
            gc_delta: delta.delta_gc,
        },
    };
    let metric_set = metric_set(metrics);
    bijux_analyze::validate_metric_set(&metric_set)?;

    let manifest = ExecutionManifest {
        run_id: run_id.clone(),
        stage: "fastq.filter".to_string(),
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
        parameters: params.clone(),
    };
    let execution_metrics = ExecutionMetrics {
        runtime_s,
        memory_mb,
        exit_code: execution.exit_code,
    };
    let envelope = &metric_set;
    write_metrics_json(&run_dirs, &execution_metrics, envelope)?;
    write_retention_report_placeholder(&run_dirs, "fastq.filter", tool, &params)?;
    let adapter_bank_path = bijux_domain_fastq::adapter_bank_path();
    write_run_manifest(&run_dirs, "fastq.filter", tool, &adapter_bank_path, &[])?;
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

fn check_fastq_filter_comparability(records: &[BenchmarkRecord<FastqFilterMetrics>]) {
    if records.len() <= 1 {
        return;
    }
    let first = &records[0];
    let mut reads_in = first.metrics.metrics.reads_in;
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
        if (record.metrics.metrics.mean_q_before - mean_q_before).abs() > 1e-6 {
            tracing::warn!(
                tool = record.context.tool,
                mean_q_before = record.metrics.metrics.mean_q_before,
                "mean_q_before differs from baseline"
            );
            mean_q_before = record.metrics.metrics.mean_q_before;
        }
        if record.metrics.metrics.reads_out > record.metrics.metrics.reads_in {
            tracing::warn!(
                tool = record.context.tool,
                reads_in = record.metrics.metrics.reads_in,
                reads_out = record.metrics.metrics.reads_out,
                "reads_out exceeds reads_in"
            );
        }
    }
}
