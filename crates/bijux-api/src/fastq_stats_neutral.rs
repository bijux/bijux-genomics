use std::collections::HashMap;
use std::path::PathBuf;

use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_registry};
use anyhow::{anyhow, Context, Result};
use bijux_analyze::{
    append_jsonl, fetch_fastq_stats_v1, insert_fastq_stats_v1, metric_set, BenchmarkContext,
    BenchmarkRecord, FastqStatsMetrics, LengthHistogramBin,
};
use bijux_core::measure::ExecutionMetrics;
use bijux_core::ErrorCategory;
use bijux_core::{MetricContextV1, RunProvenanceV1, StageObservabilityContextV1};
use bijux_env_runtime::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use bijux_runner_docker::primitives::build_tool_execution_spec;
use uuid::Uuid;

use bijux_core::measure::SeqkitMetrics;
use bijux_core::validate_execution_outputs;
use bijux_engine::services::run_artifacts::{
    compute_run_id, prepare_tool_run_dirs, write_execution_logs, write_metrics_envelope,
    write_metrics_json, write_retention_report_placeholder, write_run_manifest,
    write_stage_plan_json,
};
use bijux_env_builder::image_qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use bijux_exec::primitives::execute_stage_plan;
use bijux_exec::primitives::hash_file_sha256;
use bijux_infra::{bench_base_dir, bench_tools_dir};
use bijux_planner_fastq::normalize_stats_tool_list;
use bijux_runner_docker::primitives::resolve_image_for_run;
use bijux_stages_fastq::fastq::stats_neutral::plan_stats_neutral;
use bijux_stages_fastq::observer::{input_fastq_stats, length_histogram};
use bijux_stages_fastq::StagePlanJson;
use bijux_stages_fastq::{inspect_headers, log_header_warnings, preflight_stage, FastqArtifact};

use crate::fastq_router::{write_explain_md, write_explain_plan_json, BenchOutcome};
use bijux_core::ExecutionManifest;
use bijux_stages_fastq::RawFailure;

/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning, execution, or metric recording fails.
pub fn bench_fastq_stats_neutral<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqStatsArgs,
) -> Result<BenchOutcome<FastqStatsMetrics>> {
    let tools = normalize_stats_tool_list(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage("fastq.stats_neutral", artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings("fastq.stats_neutral", &header);
    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.stats_neutral", &tools, &registry, false)?;
    let bench_inputs = prepare_stats_bench(catalog, platform, runner_override, args)?;
    let selected = tools.clone();
    let all_tools: Vec<String> = registry
        .tools_for_stage("fastq.stats_neutral")
        .iter()
        .map(|tool| tool.tool_id.clone())
        .collect();
    let excluded: Vec<String> = all_tools
        .into_iter()
        .filter(|tool| !selected.contains(tool))
        .collect();
    write_explain_md(
        &bench_inputs.bench_dir,
        "fastq.stats_neutral",
        &selected,
        &excluded,
        None,
    )?;
    write_explain_plan_json(
        &bench_inputs.bench_dir,
        "fastq.stats_neutral",
        &selected,
        &registry,
        None,
    )?;
    ensure_image_qa_passed("fastq.stats_neutral", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.stats_neutral", &tools, platform, catalog)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let mut records: Vec<BenchmarkRecord<FastqStatsMetrics>> = Vec::new();
    let mut new_records: Vec<BenchmarkRecord<FastqStatsMetrics>> = Vec::new();
    let mut failures: Vec<RawFailure> = Vec::new();

    let runner = bench_inputs.runner.to_string();
    let platform_name = platform.name.clone();
    for tool in tools {
        let tool_spec =
            build_tool_execution_spec("fastq.stats_neutral", &tool, &registry, catalog, platform)?;
        let tool_dir = bench_inputs.tools_root.join(&tool);
        let plan = plan_stats_neutral(&tool_spec, &bench_inputs.r1, &tool_dir)?;
        let params_hash =
            bijux_core::params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        let cached = fetch_fastq_stats_v1(
            &conn,
            &tool,
            &tool_spec.tool_version,
            &image_digest,
            &runner,
            &platform_name,
            &bench_inputs.input_hash,
            &params_hash,
        );
        if let Ok(Some(record)) = cached {
            records.push(record);
            continue;
        }
        match run_stats_tool(catalog, platform, args, &bench_inputs, &tool) {
            Ok(record) => new_records.push(record),
            Err(err) => failures.push(RawFailure {
                stage: "fastq.stats_neutral".to_string(),
                tool: tool.clone(),
                reason: err.to_string(),
                category: ErrorCategory::ToolError,
            }),
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

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: bench_inputs.bench_dir,
        explain: args.explain,
    })
}

struct StatsBenchInputs {
    runner: RunnerKind,
    r1: PathBuf,
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
    args: &bijux_stages_fastq::args::BenchFastqStatsArgs,
) -> Result<StatsBenchInputs> {
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir = bench_base_dir(&args.out, "stats", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "stats", &args.sample_id);
    bijux_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_infra::ensure_dir(&tools_root).context("create tools output dir")?;

    println!(
        "planned tools: {}",
        normalize_stats_tool_list(&args.tools)?.join(", ")
    );

    let r1 = args.r1.canonicalize().context("resolve r1 path")?;
    let r1_dir = r1
        .parent()
        .ok_or_else(|| anyhow!("r1 has no parent"))?
        .to_path_buf();

    let tool_id = bijux_stages_fastq::TOOL_SEQKIT;
    let tool_spec = catalog
        .get(tool_id)
        .ok_or_else(|| anyhow!("{tool_id} missing from images.toml"))?;
    let tool_image = resolve_image_for_run(tool_spec, platform)?;

    let input_hash = hash_file_sha256(&r1)?;
    let input_stats = input_fastq_stats(&tool_image, &r1_dir, &r1)?;
    let length_hist = length_histogram(&tool_image, &r1_dir, &r1)?
        .into_iter()
        .map(|(length, count)| LengthHistogramBin { length, count })
        .collect();

    Ok(StatsBenchInputs {
        runner,
        r1,
        input_hash,
        input_stats,
        length_hist,
        bench_dir,
        tools_root,
    })
}

#[allow(clippy::too_many_lines)]
fn run_stats_tool<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    _args: &bijux_stages_fastq::args::BenchFastqStatsArgs,
    bench_inputs: &StatsBenchInputs,
    tool: &str,
) -> Result<BenchmarkRecord<FastqStatsMetrics>> {
    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tool_spec =
        build_tool_execution_spec("fastq.stats_neutral", tool, &registry, catalog, platform)?;

    println!("→ stats {tool}");
    let tool_dir = bench_inputs.tools_root.join(tool);
    let plan = plan_stats_neutral(&tool_spec, &bench_inputs.r1, &tool_dir)?;
    let plan_json = StagePlanJson::from_plan(&plan);
    let params = plan.params.clone();
    let param_hash =
        bijux_core::params_hash(&params).unwrap_or_else(|_| Uuid::new_v4().to_string());
    let image_digest = tool_spec
        .image
        .digest
        .as_ref()
        .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
        .clone();
    let run_id = compute_run_id(
        "fastq.stats_neutral",
        tool,
        &image_digest,
        &bench_inputs.input_hash,
        &param_hash,
    );
    let run_dirs = prepare_tool_run_dirs(&bench_inputs.tools_root, tool, &run_id)?;
    let out_dir = run_dirs.artifacts_dir.clone();
    let _plan_path = write_stage_plan_json(&run_dirs, "fastq_stats_neutral.plan.json", &plan_json)?;
    let execution = execute_stage_plan(&plan, bench_inputs.runner, None)?;

    let metrics = FastqStatsMetrics {
        reads_total: bench_inputs.input_stats.reads,
        bases_total: bench_inputs.input_stats.bases,
        mean_q: bench_inputs.input_stats.mean_q,
        gc_percent: bench_inputs.input_stats.gc_percent,
        length_histogram: bench_inputs.length_hist.clone(),
    };
    let metric_set = metric_set(metrics);
    bijux_analyze::validate_metric_set(&metric_set)?;

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tool_manifest = registry
        .tool_by_id("fastq.stats_neutral", tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from manifests"))?;
    validate_execution_outputs(&tool_manifest.execution_contract, &out_dir)?;
    let manifest = ExecutionManifest {
        run_id: run_id.clone(),
        stage: "fastq.stats_neutral".to_string(),
        tool: tool.to_string(),
        tool_version: tool_spec.tool_version.clone(),
        image_digest: image_digest.clone(),
        command: execution.command.clone(),
        input_hashes: vec![bench_inputs.input_hash.clone()],
        input_files: vec![bench_inputs.r1.display().to_string()],
        output_dir: out_dir.display().to_string(),
        runner: bench_inputs.runner.to_string(),
        platform: platform.name.clone(),
        arch: platform.arch.clone(),
    };
    bijux_infra::atomic_write_json(&run_dirs.manifest_path, &manifest)
        .context("write execution manifest")?;
    write_execution_logs(&run_dirs, &execution.stdout, &execution.stderr)?;
    let context = BenchmarkContext {
        tool: tool.to_string(),
        tool_version: tool_spec.tool_version.clone(),
        image_digest: image_digest.clone(),
        runner: bench_inputs.runner.to_string(),
        platform: platform.name.clone(),
        input_hash: bench_inputs.input_hash.clone(),
        parameters: params.clone().into(),
    };
    let execution_metrics = ExecutionMetrics {
        runtime_s: execution.runtime_s,
        memory_mb: execution.memory_mb,
        exit_code: execution.exit_code,
    };
    let metrics_json = serde_json::to_value(&metric_set)?;
    let stage_ctx = StageObservabilityContextV1 {
        stage_id: "fastq.stats_neutral".to_string(),
        stage_version: i32::try_from(plan.stage_version.0).unwrap_or(i32::MAX),
        tool_id: tool.to_string(),
        tool_version: tool_spec.tool_version.clone(),
        input_hash: bench_inputs.input_hash.clone(),
        params_hash: param_hash.clone(),
        parameters_json: params.clone(),
        metric_context: MetricContextV1 {
            tool_id: tool.to_string(),
            tool_version: tool_spec.tool_version.clone(),
            image_digest: Some(image_digest.clone()),
            runner: bench_inputs.runner.to_string(),
            platform: platform.name.clone(),
            input_hash: bench_inputs.input_hash.clone(),
            params_hash: param_hash.clone(),
            presets: std::collections::BTreeMap::new(),
            banks: std::collections::BTreeMap::new(),
        },
    };
    let _metrics_envelope_path = write_metrics_envelope(
        &out_dir.join("run_artifacts"),
        &stage_ctx,
        &execution_metrics,
        &metrics_json,
        &[],
    )?;
    let envelope = &metric_set;
    write_metrics_json(&run_dirs, &execution_metrics, envelope)?;
    write_retention_report_placeholder(&run_dirs, "fastq.stats_neutral", tool, &params)?;
    let adapter_bank_path = bijux_stages_fastq::adapter_bank_path();
    let run_provenance = RunProvenanceV1 {
        schema_version: "bijux.run_provenance.v1".to_string(),
        tool_image_digest: Some(image_digest.clone()),
        tool_version: tool_spec.tool_version.clone(),
        params_hash: param_hash.clone(),
        input_hashes: vec![bench_inputs.input_hash.clone()],
        reference_genome: None,
        pipeline_id: "fastq.stats_neutral".to_string(),
        git_commit: std::env::var("BIJUX_GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string()),
        build_profile: std::env::var("BIJUX_BUILD_PROFILE")
            .unwrap_or_else(|_| "unknown".to_string()),
    };
    write_run_manifest(
        &run_dirs,
        "fastq.stats_neutral",
        tool,
        &adapter_bank_path,
        &run_provenance,
        &[],
    )?;
    let record = BenchmarkRecord {
        context,
        execution: execution_metrics,
        metrics: metric_set,
    };
    record.validate()?;
    Ok(record)
}
