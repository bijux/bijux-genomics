use std::collections::HashMap;
use std::path::PathBuf;

use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_registry};
use crate::{execution_kernel, execution_kernel::NetworkPolicy};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{fetch_fastq_stats_v1, insert_fastq_stats_v1};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkContext, BenchmarkRecord, FastqStatsMetrics,
    LengthHistogramBin,
};
use bijux_dna_core::metrics::MetricContextV1;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::params_hash;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runtime::{RunProvenanceV1, StageObservabilityContextV1};
use uuid::Uuid;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use bijux_dna_core::contract::validate_execution_outputs;
use bijux_dna_core::prelude::measure::SeqkitMetrics;
use bijux_dna_infra::hash_file_sha256;
use bijux_dna_infra::{bench_base_dir, bench_tools_dir};
use bijux_dna_planner_fastq::select_stats_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::stats_neutral::plan_stats_neutral;
use bijux_dna_planner_fastq::stage_api::observer::{
    input_fastq_stats, length_histogram_command, parse_length_histogram, parse_seqkit_stats,
};
use bijux_dna_planner_fastq::stage_api::StagePlanJson;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifact,
};
use bijux_dna_runner::backend::docker::executor::resolve_image_for_run;
use bijux_dna_runner::execute::execute_observer_command;
use bijux_dna_runtime::recording::{
    compute_run_id, prepare_tool_run_dirs, write_execution_logs, write_metrics_envelope,
    write_metrics_json, write_run_manifest, write_stage_plan_json, RunArtifactInput,
};

use crate::api_internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_STATS_NEUTRAL,
};
use bijux_dna_core::contract::{ContractVersion, ExecutionManifest};
use bijux_dna_planner_fastq::stage_api::RawFailure;

/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning, execution, or metric recording fails.
pub fn bench_fastq_stats_neutral<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqStatsArgs,
) -> Result<BenchOutcome<FastqStatsMetrics>> {
    let tools = select_stats_tools(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage(STAGE_STATS_NEUTRAL.as_str(), artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings(STAGE_STATS_NEUTRAL.as_str(), &header);
    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_STATS_NEUTRAL.as_str(), &tools, &registry, false)?;
    let bench_inputs = prepare_stats_bench(catalog, platform, runner_override, args)?;
    let selected = tools.clone();
    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_STATS_NEUTRAL.as_str());
    let all_tools: Vec<String> = registry
        .tools_for_stage(&stage_id)
        .iter()
        .map(|tool| tool.tool_id.to_string())
        .collect();
    let excluded: Vec<String> = all_tools
        .into_iter()
        .filter(|tool| !selected.contains(tool))
        .collect();
    write_explain_md(
        &bench_inputs.bench_dir,
        STAGE_STATS_NEUTRAL.as_str(),
        &selected,
        &excluded,
        None,
    )?;
    write_explain_plan_json(
        &bench_inputs.bench_dir,
        STAGE_STATS_NEUTRAL.as_str(),
        &selected,
        &registry,
        None,
    )?;
    ensure_image_qa_passed(STAGE_STATS_NEUTRAL.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_STATS_NEUTRAL.as_str(), &tools, platform, catalog)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let mut records: Vec<BenchmarkRecord<FastqStatsMetrics>> = Vec::new();
    let mut new_records: Vec<BenchmarkRecord<FastqStatsMetrics>> = Vec::new();
    let mut failures: Vec<RawFailure> = Vec::new();

    let runner = bench_inputs.runner.to_string();
    let platform_name = platform.name.clone();
    for tool in tools {
        let tool_spec = build_tool_execution_spec(
            STAGE_STATS_NEUTRAL.as_str(),
            &tool,
            &registry,
            catalog,
            platform,
        )?;
        let tool_dir = bench_inputs.tools_root.join(&tool);
        let plan = plan_stats_neutral(&tool_spec, &bench_inputs.r1, &tool_dir)?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
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
                stage: STAGE_STATS_NEUTRAL.as_str().to_string(),
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
    runner: RuntimeKind,
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
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqStatsArgs,
) -> Result<StatsBenchInputs> {
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir_name = bench_dir_name(&STAGE_STATS_NEUTRAL)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_STATS_NEUTRAL.as_str()))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_dna_infra::ensure_dir(&tools_root).context("create tools output dir")?;

    println!(
        "planned tools: {}",
        select_stats_tools(&args.tools)?.join(", ")
    );

    let r1 = args.r1.canonicalize().context("resolve r1 path")?;
    let r1_dir = r1
        .parent()
        .ok_or_else(|| anyhow!("r1 has no parent"))?
        .to_path_buf();

    let tool_id = bijux_dna_planner_fastq::stage_api::TOOL_SEQKIT;
    let tool_spec = catalog
        .get(tool_id)
        .ok_or_else(|| anyhow!("{tool_id} missing from images.toml"))?;
    let tool_image = resolve_image_for_run(tool_spec, platform)?;

    let input_hash = hash_file_sha256(&r1)?;
    let stats_spec = input_fastq_stats(&r1_dir, &r1)?;
    let stats_output = execute_observer_command(
        &tool_image.full_name,
        stats_spec.mount_dir.as_path(),
        &stats_spec.args,
        runner,
    )?;
    if stats_output.exit_code != 0 {
        return Err(anyhow!("seqkit stats failed: {}", stats_output.stderr));
    }
    let input_stats = parse_seqkit_stats(&stats_output.stdout)?;

    let hist_spec = length_histogram_command(&r1_dir, &r1)?;
    let hist_output = execute_observer_command(
        &tool_image.full_name,
        hist_spec.mount_dir.as_path(),
        &hist_spec.args,
        runner,
    )?;
    if hist_output.exit_code != 0 {
        return Err(anyhow!(
            "seqkit length histogram failed: {}",
            hist_output.stderr
        ));
    }
    let length_hist = parse_length_histogram(&hist_output.stdout)?
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
    _args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqStatsArgs,
    bench_inputs: &StatsBenchInputs,
    tool: &str,
) -> Result<BenchmarkRecord<FastqStatsMetrics>> {
    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tool_spec = build_tool_execution_spec(
        STAGE_STATS_NEUTRAL.as_str(),
        tool,
        &registry,
        catalog,
        platform,
    )?;

    println!("→ stats {tool}");
    let tool_dir = bench_inputs.tools_root.join(tool);
    let plan = plan_stats_neutral(&tool_spec, &bench_inputs.r1, &tool_dir)?;
    let plan_json = StagePlanJson::from_plan(&plan);
    let params = plan.params.clone();
    let param_hash = params_hash(&params).unwrap_or_else(|_| Uuid::new_v4().to_string());
    let image_digest = tool_spec
        .image
        .digest
        .as_ref()
        .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
        .clone();
    let run_id = compute_run_id(
        STAGE_STATS_NEUTRAL.as_str(),
        tool,
        &image_digest,
        &bench_inputs.input_hash,
        &param_hash,
    );
    let run_dirs = prepare_tool_run_dirs(&bench_inputs.tools_root, tool, &run_id)?;
    let out_dir = run_dirs.artifacts_dir.clone();
    let _plan_path = write_stage_plan_json(&run_dirs, "fastq_stats_neutral.plan.json", &plan_json)?;
    let step = bijux_dna_stage_contract::execution_step_from_stage_plan(&plan);
    let tool_context = execution_kernel::ToolContext {
        run_id: run_id.clone(),
        stage_id: STAGE_STATS_NEUTRAL.as_str().to_string(),
        tool_id: tool.to_string(),
        sample_id: None,
        stage_root: run_dirs.logs_dir.clone(),
        input_root: bench_inputs
            .r1
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| bench_inputs.bench_dir.clone()),
        output_root: out_dir.clone(),
        tmp_root: run_dirs.logs_dir.join("tmp"),
        threads: plan.resources.threads.max(1),
        memory_hint_mb: Some(u64::from(plan.resources.mem_gb).saturating_mul(1024)),
        seed: None,
        network_policy: NetworkPolicy::Allow,
    };
    let execution = execution_kernel::invoke_tool(&execution_kernel::ToolInvocationRequest {
        step: step.clone(),
        runner: bench_inputs.runner,
        context: tool_context,
        timeout: None,
    })?
    .stage_result;

    let metrics = FastqStatsMetrics {
        reads_total: bench_inputs.input_stats.reads,
        bases_total: bench_inputs.input_stats.bases,
        mean_q: bench_inputs.input_stats.mean_q,
        gc_percent: bench_inputs.input_stats.gc_percent,
        length_histogram: bench_inputs.length_hist.clone(),
    };
    let metric_set = metric_set(metrics);
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_STATS_NEUTRAL.as_str());
    let tool_manifest = registry
        .tool_by_id(&stage_id, &bijux_dna_core::ids::ToolId::new(tool))
        .ok_or_else(|| anyhow!("tool {tool} missing from manifests"))?;
    validate_execution_outputs(&tool_manifest.execution_contract, &out_dir)?;
    let manifest = ExecutionManifest {
        contract_version: ContractVersion::v1(),
        run_id: run_id.clone(),
        stage: STAGE_STATS_NEUTRAL.as_str().to_string(),
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
    bijux_dna_infra::atomic_write_json(&run_dirs.manifest_path, &manifest)
        .context("write execution manifest")?;
    write_execution_logs(&run_dirs.logs_dir, &execution.stdout, &execution.stderr)?;
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
    let parameters_json_normalized =
        bijux_dna_core::contract::canonical::parameters_json_canonicalization(&params);
    let stage_ctx = StageObservabilityContextV1 {
        stage_id: STAGE_STATS_NEUTRAL.as_str().to_string(),
        stage_version: plan.stage_version.0,
        tool_id: tool.to_string(),
        tool_version: tool_spec.tool_version.clone(),
        input_fingerprint: bench_inputs.input_hash.clone(),
        parameters_fingerprint: param_hash.clone(),
        parameters_json: params.clone(),
        parameters_json_normalized,
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
        &bijux_dna_runtime::recording::run_artifacts_dir_for_out(&out_dir),
        &stage_ctx,
        &metrics_json,
        std::slice::from_ref(&bench_inputs.input_hash),
    )?;
    let envelope = &metric_set;
    write_metrics_json(&run_dirs, &execution_metrics, envelope)?;
    let adapter_bank_path = bijux_dna_planner_fastq::stage_api::adapter_bank_path();
    let run_provenance = RunProvenanceV1 {
        schema_version: "bijux.run_provenance.v1".to_string(),
        tool_image_digest: Some(image_digest.clone()),
        tool_version: tool_spec.tool_version.clone(),
        params_hash: param_hash.clone(),
        input_hashes: vec![bench_inputs.input_hash.clone()],
        reference_genome: None,
        pipeline_id: STAGE_STATS_NEUTRAL.as_str().to_string(),
        git_commit: std::env::var("BIJUX_GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string()),
        build_profile: std::env::var("BIJUX_BUILD_PROFILE")
            .unwrap_or_else(|_| "unknown".to_string()),
        plan_hash: std::env::var("BIJUX_PLAN_HASH").ok(),
    };
    let extra_artifacts = [RunArtifactInput {
        name: "adapter_bank",
        path: adapter_bank_path,
    }];
    let stage_contract_hash =
        bijux_dna_domain_fastq::stage_contract_hash(STAGE_STATS_NEUTRAL.as_str())
            .and_then(std::result::Result::ok);
    write_run_manifest(
        &run_dirs,
        STAGE_STATS_NEUTRAL.as_str(),
        tool,
        &run_provenance,
        stage_contract_hash,
        &extra_artifacts,
    )?;
    let record = BenchmarkRecord {
        context,
        execution: execution_metrics,
        metrics: metric_set,
    };
    record.validate()?;
    Ok(record)
}
