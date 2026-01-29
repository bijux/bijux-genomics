use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_analyze::{
    append_jsonl, fetch_fastq_correct_v1, insert_fastq_correct_v1, metric_set, BenchmarkContext,
    BenchmarkRecord, FastqCorrectMetrics,
};
use bijux_core::measure::ExecutionMetrics;
use bijux_engine::api::{ensure_bench_runner, load_registry};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use uuid::Uuid;

use bijux_engine::api::validate_execution_outputs;
use bijux_engine::api::{bench_base_dir, bench_tools_dir};
use bijux_engine::api::{cleanup_execution, execution_memory_mb, run_tool_execution};
use bijux_engine::api::{hash_file_sha256, input_fastq_stats, output_fastq_stats, SeqkitMetrics};
use bijux_environment::image_qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use bijux_stages_fastq::fastq::correct::{normalize_correct_tool_list, plan_correct};
use bijux_stages_fastq::{
    contract_for_stage, inspect_headers, log_header_warnings, normalize_outputs, preflight_stage,
    FastqArtifact, FastqArtifactKind, StagePlanJson,
};

use crate::fastq_exec::helpers::{
    compute_run_id, params_hash, prepare_tool_run_dirs, resolve_image_for_run,
    write_execution_logs, write_explain_md, write_explain_plan_json, write_metrics_json,
    write_retention_report_placeholder, write_run_manifest, write_stage_plan_json,
    ExecutionManifest,
};
use crate::fastq_exec::helpers::{filter_tools_by_role, BenchOutcome};
use bijux_stages_fastq::RawFailure;

/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning, execution, or metric recording fails.
pub fn bench_fastq_correct<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqCorrectArgs,
) -> Result<BenchOutcome<FastqCorrectMetrics>> {
    let tools = normalize_correct_tool_list(&args.tools)?;
    let r2 = args
        .r2
        .as_ref()
        .ok_or_else(|| anyhow!("r2 required for fastq.correct"))?;
    let _artifacts = FastqArtifact::paired_end(&args.r1, r2);
    preflight_stage("fastq.correct", FastqArtifactKind::PairedEnd)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings("fastq.correct", &header);
    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.correct", &tools, &registry, false)?;
    let bench_inputs = prepare_correct_bench(catalog, platform, runner_override, args)?;
    let selected = tools.clone();
    let all_tools: Vec<String> = registry
        .tools_for_stage("fastq.correct")
        .iter()
        .map(|tool| tool.tool_id.clone())
        .collect();
    let excluded: Vec<String> = all_tools
        .into_iter()
        .filter(|tool| !selected.contains(tool))
        .collect();
    write_explain_md(
        &bench_inputs.bench_dir,
        "fastq.correct",
        &selected,
        &excluded,
        None,
    )?;
    write_explain_plan_json(
        &bench_inputs.bench_dir,
        "fastq.correct",
        &selected,
        &registry,
        None,
    )?;
    ensure_image_qa_passed("fastq.correct", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.correct", &tools, platform, catalog)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let mut records: Vec<BenchmarkRecord<FastqCorrectMetrics>> = Vec::new();
    let mut new_records: Vec<BenchmarkRecord<FastqCorrectMetrics>> = Vec::new();
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
            Err(err) => failures.push(RawFailure {
                stage: "fastq.correct".to_string(),
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
        insert_fastq_correct_v1(&conn, record).context("insert bench sqlite")?;
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: bench_inputs.bench_dir,
        explain: args.explain,
    })
}

struct CorrectBenchInputs {
    runner: RunnerKind,
    r1: PathBuf,
    r2: PathBuf,
    r1_dir: PathBuf,
    input_hash: String,
    input_stats: SeqkitMetrics,
    bench_dir: PathBuf,
    tools_root: PathBuf,
}

fn prepare_correct_bench<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqCorrectArgs,
) -> Result<CorrectBenchInputs> {
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir = bench_base_dir(&args.out, "correct", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "correct", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    println!(
        "planned tools: {}",
        normalize_correct_tool_list(&args.tools)?.join(", ")
    );

    let r1 = args.r1.canonicalize().context("resolve r1 path")?;
    let r2 = args
        .r2
        .as_ref()
        .ok_or_else(|| anyhow!("r2 required for fastq.correct"))?
        .canonicalize()
        .context("resolve r2 path")?;
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
        r2,
        r1_dir,
        input_hash,
        input_stats,
        bench_dir,
        tools_root,
    })
}

#[allow(clippy::too_many_lines)]
fn run_correct_tool<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_stages_fastq::args::BenchFastqCorrectArgs,
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
    let plan = plan_correct(tool, &bench_inputs.r1, &bench_inputs.r2, &out_dir)?;
    let plan_json = StagePlanJson::from_plan(&plan);
    let _plan_path = write_stage_plan_json(&run_dirs, "fastq_correct.plan.json", &plan_json)?;
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

    let seqkit_spec = catalog
        .get("seqkit")
        .ok_or_else(|| anyhow!("seqkit missing from images.yaml"))?;
    let seqkit_image = resolve_image_for_run(seqkit_spec, platform)?;
    let contract = contract_for_stage("fastq.correct")
        .ok_or_else(|| anyhow!("missing fastq.correct contract"))?;
    let normalized = normalize_outputs("fastq.correct", &out_dir, contract.output_kind)?;
    let out_fastq = normalized
        .r1
        .as_ref()
        .ok_or_else(|| anyhow!("output fastq missing"))?;
    let output_stats = output_fastq_stats(&seqkit_image, &out_dir, out_fastq)?;

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
    let metric_set = metric_set(metrics);
    bijux_analyze::validate_metric_set(&metric_set)?;

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
        parameters: params.clone(),
    };
    let execution_metrics = ExecutionMetrics {
        runtime_s,
        memory_mb,
        exit_code: execution.exit_code,
    };
    let envelope = &metric_set;
    write_metrics_json(&run_dirs, &execution_metrics, envelope)?;
    write_retention_report_placeholder(&run_dirs, "fastq.correct", tool, &params)?;
    let adapter_bank_path = bijux_stages_fastq::adapter_bank_path();
    write_run_manifest(&run_dirs, "fastq.correct", tool, &adapter_bank_path, &[])?;
    let record = BenchmarkRecord {
        context,
        execution: execution_metrics,
        metrics: metric_set,
    };
    record.validate()?;
    Ok(record)
}
