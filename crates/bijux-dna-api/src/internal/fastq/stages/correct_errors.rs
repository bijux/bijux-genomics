use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::internal::fastq::stages::trim_bench_common::{
    benchmark_image_identity, require_existing_benchmark_output,
};
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::quality::{fetch_fastq_correct_v1, insert_fastq_correct_v1};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkContext, BenchmarkRecord, FastqCorrectMetrics,
};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::select_correct_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::correct_errors::{
    plan_correct_with_options, project_correct_options_for_tool, CorrectPlanOptions,
};
use bijux_dna_planner_fastq::stage_api::observer::{input_fastq_stats, parse_seqkit_stats};
use bijux_dna_planner_fastq::stage_api::FastqArtifactKind;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::backend::docker::executor::resolve_image_for_run;
use bijux_dna_runner::step_runner::{execute_observer_command, StageResultV1};

use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_CORRECT_ERRORS,
};
use bijux_dna_domain_fastq::{
    params::correct::FastqCorrectParams, CorrectErrorsReportV1,
    CORRECT_ERRORS_REPORT_SCHEMA_VERSION,
};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_stage_contract::StagePlanV1;

mod support;

use self::support::{
    apply_memory_override, apply_thread_override, benchmark_query_context, build_correction_report,
    correct_metrics_from_observed_stats, input_output_content_changed, observe_fastq_stats,
    optional_plan_output_path, parse_quality_encoding, required_plan_output_path,
};

/// # Errors
/// Returns an error if planning or execution fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_correct<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqCorrectArgs,
) -> Result<BenchOutcome<bijux_dna_analyze::FastqCorrectMetrics>> {
    let selected_tools = select_correct_benchmark_tools(args)?;
    let setup =
        prepare_correct_benchmark_setup(catalog, platform, runner_override, args, &selected_tools)?;

    if args.explain {
        write_correct_benchmark_explain(&setup)?;
    }

    ensure_correct_benchmark_qa(catalog, platform, &setup.tools)?;

    let store = CorrectBenchmarkStore::from_inputs(&setup.bench_inputs);
    let conn = bijux_dna_analyze::open_sqlite(&store.sqlite_path).context("open bench sqlite")?;
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::<BenchmarkRecord<FastqCorrectMetrics>>::new();
    for tool in &setup.tools {
        let tool_plan = prepare_correct_tool_plan(catalog, platform, args, &setup, jobs, tool)?;
        let cache_identity = CorrectCacheIdentity::from_plan(platform, &setup, tool, &tool_plan);
        if let Ok(Some(record)) = fetch_fastq_correct_v1(
            &conn,
            cache_identity.tool,
            &cache_identity.tool_version,
            &cache_identity.image_digest,
            &cache_identity.runner,
            &cache_identity.platform,
            &cache_identity.input_hash,
            &cache_identity.params_hash,
        ) {
            records.push(record);
            continue;
        }
        let execution = execute_correct_tool(&tool_plan, setup.bench_inputs.runner, jobs, tool)?;
        let record = build_correct_record(
            platform,
            &setup.bench_inputs,
            tool,
            &tool_plan.tool_spec,
            &tool_plan.bench_params,
            &tool_plan.plan,
            &execution,
        )?;
        append_jsonl(&store.jsonl_path, &record).context("write bench.jsonl")?;
        insert_fastq_correct_v1(&conn, &record).context("insert bench sqlite")?;
        if let Some(failure) = correct_tool_failure(tool, execution.exit_code) {
            failures.push(failure);
        }
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: setup.bench_inputs.bench_dir,
        explain: args.explain,
    })
}

fn select_correct_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqCorrectArgs,
) -> Result<Vec<String>> {
    let allow_experimental = std::env::var("BIJUX_EXPERIMENTAL_TOOLS").is_ok();
    let tools = select_correct_tools(&args.tools, allow_experimental)?;
    let artifact =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_CORRECT_ERRORS.as_str(), artifact)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_CORRECT_ERRORS.as_str(), &header);
    Ok(tools)
}

#[derive(Debug, Clone)]
struct CorrectBenchInputs {
    runner: RuntimeKind,
    r1: PathBuf,
    r2: Option<PathBuf>,
    input_hash: String,
    input_stats_r1: SeqkitMetrics,
    input_stats_r2: Option<SeqkitMetrics>,
    bench_dir: PathBuf,
    tools_root: PathBuf,
    seqkit_image: String,
}

struct CorrectBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    excluded_tools: Vec<String>,
    bench_inputs: CorrectBenchInputs,
}

struct CorrectBenchmarkStore {
    sqlite_path: PathBuf,
    jsonl_path: PathBuf,
}

struct CorrectToolPlan {
    tool_spec: ToolExecutionSpecV1,
    plan: StagePlanV1,
    bench_params: serde_json::Value,
    params_hash: String,
    image_digest: String,
}

struct CorrectRecordOutputs {
    output_r1: PathBuf,
    output_r2: Option<PathBuf>,
    report_path: PathBuf,
}

struct CorrectOutputObservation {
    output_stats_r1: SeqkitMetrics,
    output_stats_r2: Option<SeqkitMetrics>,
    outputs_changed: bool,
}

struct CorrectCacheIdentity<'a> {
    tool: &'a str,
    tool_version: String,
    image_digest: String,
    runner: String,
    platform: String,
    input_hash: String,
    params_hash: String,
}

impl<'a> CorrectCacheIdentity<'a> {
    fn from_plan(
        platform: &PlatformSpec,
        setup: &CorrectBenchmarkSetup,
        tool: &'a str,
        tool_plan: &CorrectToolPlan,
    ) -> Self {
        Self {
            tool,
            tool_version: tool_plan.tool_spec.tool_version.clone(),
            image_digest: tool_plan.image_digest.clone(),
            runner: setup.bench_inputs.runner.to_string(),
            platform: platform.name.clone(),
            input_hash: setup.bench_inputs.input_hash.clone(),
            params_hash: tool_plan.params_hash.clone(),
        }
    }
}

impl CorrectBenchmarkStore {
    fn from_inputs(inputs: &CorrectBenchInputs) -> Self {
        Self {
            sqlite_path: inputs.bench_dir.join("bench.sqlite"),
            jsonl_path: inputs.bench_dir.join("bench.jsonl"),
        }
    }
}

fn prepare_correct_benchmark_setup<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqCorrectArgs,
    selected_tools: &[String],
) -> Result<CorrectBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools =
        filter_tools_by_role(STAGE_CORRECT_ERRORS.as_str(), selected_tools, &registry, false)?;
    let bench_inputs = prepare_correct_bench(catalog, platform, runner_override, args)?;
    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_CORRECT_ERRORS.as_str());
    let all_tools: Vec<String> =
        registry.tools_for_stage(&stage_id).iter().map(|tool| tool.tool_id.to_string()).collect();
    let excluded_tools = all_tools.into_iter().filter(|tool| !tools.contains(tool)).collect();
    Ok(CorrectBenchmarkSetup { registry, tools, excluded_tools, bench_inputs })
}

fn write_correct_benchmark_explain(setup: &CorrectBenchmarkSetup) -> Result<()> {
    write_explain_md(
        &setup.bench_inputs.bench_dir,
        STAGE_CORRECT_ERRORS.as_str(),
        &setup.tools,
        &setup.excluded_tools,
        None,
    )?;
    write_explain_plan_json(
        &setup.bench_inputs.bench_dir,
        STAGE_CORRECT_ERRORS.as_str(),
        &setup.tools,
        &setup.registry,
        None,
    )
}

fn ensure_correct_benchmark_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(STAGE_CORRECT_ERRORS.as_str(), tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_CORRECT_ERRORS.as_str(), tools, platform, catalog)
}

fn prepare_correct_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqCorrectArgs,
    setup: &CorrectBenchmarkSetup,
    jobs: usize,
    tool: &str,
) -> Result<CorrectToolPlan> {
    let out_dir = setup.bench_inputs.tools_root.join(tool);
    bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
    let tool_spec = build_tool_execution_spec(
        STAGE_CORRECT_ERRORS.as_str(),
        tool,
        &setup.registry,
        catalog,
        platform,
    )?;
    let tool_spec = apply_thread_override(&tool_spec, args.threads);
    let tool_spec = apply_memory_override(&tool_spec, args.max_memory_gb);
    let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
    let projected_options = project_correct_options_for_tool(tool, &correct_plan_options(args)?);
    let plan = plan_correct_with_options(
        &tool_spec,
        &setup.bench_inputs.r1,
        setup.bench_inputs.r2.as_deref(),
        &out_dir,
        &projected_options,
    )?;
    let bench_params = benchmark_query_context()?.embed_in_parameters(&plan.params);
    let params_hash = stable_params_hash(&bench_params);
    let image_digest = benchmark_image_identity(&tool_spec);
    Ok(CorrectToolPlan { tool_spec, plan, bench_params, params_hash, image_digest })
}

fn correct_plan_options(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqCorrectArgs,
) -> Result<CorrectPlanOptions> {
    Ok(CorrectPlanOptions {
        threads: args.threads,
        quality_encoding: parse_quality_encoding(args.quality_encoding.as_deref())?,
        kmer_size: args.kmer_size,
        musket_kmer_budget: args.musket_kmer_budget,
        genome_size: args.genome_size,
        max_memory_gb: args.max_memory_gb,
        trusted_kmer_artifact: args.trusted_kmer_artifact.clone(),
        conservative_mode: args.conservative_mode.unwrap_or(false),
    })
}

fn execute_correct_tool(
    tool_plan: &CorrectToolPlan,
    runner: RuntimeKind,
    jobs: usize,
    tool: &str,
) -> Result<StageResultV1> {
    execute_plans_with_jobs(
        vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&tool_plan.plan)],
        runner,
        jobs,
    )?
    .into_iter()
    .next()
    .ok_or_else(|| anyhow!("missing execution result for {tool}"))
}

fn correct_tool_failure(tool: &str, exit_code: i32) -> Option<RawFailure> {
    (exit_code != 0).then(|| RawFailure {
        stage: STAGE_CORRECT_ERRORS.as_str().to_string(),
        tool: tool.to_string(),
        reason: format!("tool {tool} failed with status {exit_code}"),
        category: ErrorCategory::ToolError,
    })
}

fn prepare_correct_bench<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqCorrectArgs,
) -> Result<CorrectBenchInputs> {
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir_name = bench_dir_name(&STAGE_CORRECT_ERRORS)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_CORRECT_ERRORS.as_str()))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_dna_infra::ensure_dir(&tools_root).context("create tools output dir")?;

    let r1 = args.r1.canonicalize().context("resolve r1 path")?;
    let r1_dir = r1.parent().ok_or_else(|| anyhow!("r1 has no parent"))?.to_path_buf();

    let seqkit_tool = catalog
        .get(bijux_dna_planner_fastq::stage_api::TOOL_SEQKIT)
        .ok_or_else(|| anyhow!("seqkit missing from images catalog"))?;
    let seqkit_image = resolve_image_for_run(seqkit_tool, platform)?;
    let stats_spec = input_fastq_stats(&r1_dir, &r1)?;
    let stats_output = execute_observer_command(
        &seqkit_image.full_name,
        stats_spec.mount_dir.as_path(),
        &stats_spec.args,
        runner,
    )?;
    if stats_output.exit_code != 0 {
        return Err(anyhow!("seqkit correction observer failed: {}", stats_output.stderr));
    }
    let input_stats_r1 = parse_seqkit_stats(&stats_output.stdout)?;

    let (r2, input_stats_r2, input_hash) = if let Some(r2) = args.r2.as_deref() {
        let r2 = r2.canonicalize().context("resolve r2 path")?;
        let r2_dir = r2.parent().ok_or_else(|| anyhow!("r2 has no parent"))?.to_path_buf();
        let stats_spec_r2 = input_fastq_stats(&r2_dir, &r2)?;
        let stats_output_r2 = execute_observer_command(
            &seqkit_image.full_name,
            stats_spec_r2.mount_dir.as_path(),
            &stats_spec_r2.args,
            runner,
        )?;
        if stats_output_r2.exit_code != 0 {
            return Err(anyhow!(
                "seqkit correction observer failed for r2: {}",
                stats_output_r2.stderr
            ));
        }
        let r2_hash = hash_file_sha256(&r2).context("hash correction input r2")?;
        (
            Some(r2),
            Some(parse_seqkit_stats(&stats_output_r2.stdout)?),
            format!("{}+{}", hash_file_sha256(&r1).context("hash correction input r1")?, r2_hash),
        )
    } else {
        (None, None, hash_file_sha256(&r1).context("hash correction input")?)
    };

    Ok(CorrectBenchInputs {
        runner,
        r1,
        r2,
        input_hash,
        input_stats_r1,
        input_stats_r2,
        bench_dir,
        tools_root,
        seqkit_image: seqkit_image.full_name,
    })
}

fn build_correct_record(
    platform: &PlatformSpec,
    bench_inputs: &CorrectBenchInputs,
    tool: &str,
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    params: &serde_json::Value,
    plan: &StagePlanV1,
    execution: &StageResultV1,
) -> Result<BenchmarkRecord<FastqCorrectMetrics>> {
    let out_dir = &plan.out_dir;
    let outputs = resolve_correct_record_outputs(plan)?;
    let observation = observe_correct_outputs(bench_inputs, &outputs)?;
    let metrics = correct_metrics_from_observed_stats(
        &bench_inputs.input_stats_r1,
        bench_inputs.input_stats_r2.as_ref(),
        &observation.output_stats_r1,
        observation.output_stats_r2.as_ref(),
        observation.outputs_changed,
    );
    let metric_set = correct_metric_set(metrics.clone())?;

    let report = build_correction_report(
        tool,
        &bench_inputs.r1,
        bench_inputs.r2.as_deref(),
        &outputs.output_r1,
        outputs.output_r2.as_deref(),
        &outputs.report_path,
        &plan.effective_params,
        &metrics,
        execution,
        observation.outputs_changed,
    )?;
    write_correct_artifacts(out_dir, &outputs, &report, &metric_set)?;

    let context = BenchmarkContext {
        tool: tool.to_string(),
        tool_version: tool_spec.tool_version.clone(),
        image_digest: benchmark_image_identity(tool_spec),
        runner: bench_inputs.runner.to_string(),
        platform: platform.name.clone(),
        input_hash: bench_inputs.input_hash.clone(),
        parameters: params.clone().into(),
    };
    let record = BenchmarkRecord {
        context,
        execution: ExecutionMetrics {
            runtime_s: execution.runtime_s,
            memory_mb: execution.memory_mb,
            exit_code: execution.exit_code,
        },
        metrics: metric_set,
    };
    record.validate()?;
    Ok(record)
}

fn resolve_correct_record_outputs(plan: &StagePlanV1) -> Result<CorrectRecordOutputs> {
    let output_r1 = required_plan_output_path(plan, "corrected_reads_r1")?;
    let output_r1 = require_existing_benchmark_output(&output_r1, "corrected_reads_r1")?;
    let output_r2 = optional_plan_output_path(plan, "corrected_reads_r2")
        .map(|path| -> Result<PathBuf> {
            require_existing_benchmark_output(&path, "corrected_reads_r2")?;
            Ok(path)
        })
        .transpose()?;
    let report_path = required_plan_output_path(plan, "report_json")?;
    Ok(CorrectRecordOutputs { output_r1: output_r1.to_path_buf(), output_r2, report_path })
}

fn observe_correct_outputs(
    bench_inputs: &CorrectBenchInputs,
    outputs: &CorrectRecordOutputs,
) -> Result<CorrectOutputObservation> {
    let output_stats_r1 =
        observe_fastq_stats(&bench_inputs.seqkit_image, bench_inputs.runner, &outputs.output_r1)?;
    let output_stats_r2 = outputs
        .output_r2
        .as_deref()
        .map(|path| observe_fastq_stats(&bench_inputs.seqkit_image, bench_inputs.runner, path))
        .transpose()?;
    let outputs_changed = input_output_content_changed(
        &bench_inputs.r1,
        bench_inputs.r2.as_deref(),
        &outputs.output_r1,
        outputs.output_r2.as_deref(),
    )?;
    Ok(CorrectOutputObservation { output_stats_r1, output_stats_r2, outputs_changed })
}

fn correct_metric_set(
    metrics: FastqCorrectMetrics,
) -> Result<bijux_dna_analyze::MetricSet<FastqCorrectMetrics>> {
    let metric_set = metric_set(metrics);
    bijux_dna_analyze::validate_metric_set(&metric_set)?;
    Ok(metric_set)
}

fn write_correct_artifacts(
    out_dir: &Path,
    outputs: &CorrectRecordOutputs,
    report: &CorrectErrorsReportV1,
    metric_set: &bijux_dna_analyze::MetricSet<FastqCorrectMetrics>,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(&outputs.report_path, report)
        .context("write correction report")?;
    let metrics_json = serde_json::to_value(metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write correction metrics")
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::{
        apply_thread_override, build_correction_report, correct_metrics_from_observed_stats,
        optional_plan_output_path, required_plan_output_path,
    };
    use bijux_dna_core::contract::{ArtifactRole, StageIO};
    use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, ToolId};
    use bijux_dna_core::prelude::measure::SeqkitMetrics;
    use bijux_dna_core::prelude::{
        CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1,
    };
    use bijux_dna_planner_fastq::stage_api::fastq::correct_errors::{
        project_correct_options_for_tool, CorrectPlanOptions,
    };
    use bijux_dna_runner::step_runner::StageResultV1;
    use bijux_dna_stage_contract::{PlanDecisionReason, StagePlanV1};
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    #[test]
    fn correct_record_paths_follow_plan_outputs() {
        let plan = StagePlanV1 {
            stage_id: StageId::from_static("fastq.correct_errors"),
            stage_instance_id: None,
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("musket"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
            command: CommandSpecV1 { template: vec!["musket".to_string()] },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: Vec::new(),
                outputs: vec![
                    bijux_dna_core::prelude::ArtifactRef::required(
                        ArtifactId::from_static("corrected_reads_r1"),
                        PathBuf::from("custom/r1.corrected.fastq.gz"),
                        ArtifactRole::Reads,
                    ),
                    bijux_dna_core::prelude::ArtifactRef::required(
                        ArtifactId::from_static("corrected_reads_r2"),
                        PathBuf::from("custom/r2.corrected.fastq.gz"),
                        ArtifactRole::Reads,
                    ),
                ],
            },
            out_dir: PathBuf::from("custom"),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            aux_images: BTreeMap::new(),
            reason: PlanDecisionReason::default(),
        };

        assert_eq!(
            required_plan_output_path(&plan, "corrected_reads_r1").expect("r1 path"),
            PathBuf::from("custom/r1.corrected.fastq.gz")
        );
        assert_eq!(
            required_plan_output_path(&plan, "corrected_reads_r2").expect("r2 path"),
            PathBuf::from("custom/r2.corrected.fastq.gz")
        );
    }

    #[test]
    fn missing_corrected_output_is_rejected_before_metrics() {
        let plan = StagePlanV1 {
            stage_id: StageId::from_static("fastq.correct_errors"),
            stage_instance_id: None,
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("musket"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
            command: CommandSpecV1 { template: vec!["musket".to_string()] },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: Vec::new(),
                outputs: vec![bijux_dna_core::prelude::ArtifactRef::required(
                    ArtifactId::from_static("corrected_reads_r1"),
                    PathBuf::from("custom/r1.corrected.fastq.gz"),
                    ArtifactRole::Reads,
                )],
            },
            out_dir: PathBuf::from("custom"),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            aux_images: BTreeMap::new(),
            reason: PlanDecisionReason::default(),
        };

        let error = required_plan_output_path(&plan, "corrected_reads_r2")
            .expect_err("missing governed output must be rejected");
        assert!(error.to_string().contains("missing governed output `corrected_reads_r2`"));
    }

    fn seqkit_metrics(reads: u64, bases: u64, mean_q: f64) -> SeqkitMetrics {
        SeqkitMetrics { reads, bases, mean_q, gc_percent: 0.0 }
    }

    #[test]
    fn correct_metrics_aggregate_both_mates() {
        let metrics = correct_metrics_from_observed_stats(
            &seqkit_metrics(100, 10_000, 30.0),
            Some(&seqkit_metrics(100, 9_000, 31.0)),
            &seqkit_metrics(100, 9_800, 33.0),
            Some(&seqkit_metrics(100, 8_900, 34.0)),
            true,
        );
        assert_eq!(metrics.reads_in, 200);
        assert_eq!(metrics.reads_out, 200);
        assert_eq!(metrics.bases_in, 19_000);
        assert_eq!(metrics.bases_out, 18_700);
        assert_eq!(metrics.pairs_in, Some(100));
        assert_eq!(metrics.pairs_out, Some(100));
        assert!(metrics.mean_q_after > metrics.mean_q_before);
        assert!(metrics.kmer_fix_rate > 0.0);
    }

    #[test]
    fn unchanged_corrected_outputs_zero_the_fix_rate_proxy() {
        let metrics = correct_metrics_from_observed_stats(
            &seqkit_metrics(100, 10_000, 30.0),
            Some(&seqkit_metrics(100, 10_000, 30.0)),
            &seqkit_metrics(100, 10_000, 30.0),
            Some(&seqkit_metrics(100, 10_000, 30.0)),
            false,
        );
        assert!((metrics.kmer_fix_rate - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn single_end_metrics_do_not_invent_pair_counts() {
        let metrics = correct_metrics_from_observed_stats(
            &seqkit_metrics(100, 10_000, 30.0),
            None,
            &seqkit_metrics(100, 9_700, 33.0),
            None,
            true,
        );
        assert_eq!(metrics.reads_in, 100);
        assert_eq!(metrics.reads_out, 100);
        assert_eq!(metrics.pairs_in, None);
        assert_eq!(metrics.pairs_out, None);
        assert!(metrics.mean_q_after > metrics.mean_q_before);
    }

    #[test]
    fn optional_corrected_mate_output_can_be_absent() {
        let plan = StagePlanV1 {
            stage_id: StageId::from_static("fastq.correct_errors"),
            stage_instance_id: None,
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("rcorrector"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
            command: CommandSpecV1 { template: vec!["rcorrector".to_string()] },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: Vec::new(),
                outputs: vec![bijux_dna_core::prelude::ArtifactRef::required(
                    ArtifactId::from_static("corrected_reads_r1"),
                    PathBuf::from("custom/r1.corrected.fastq.gz"),
                    ArtifactRole::Reads,
                )],
            },
            out_dir: PathBuf::from("custom"),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            aux_images: BTreeMap::new(),
            reason: PlanDecisionReason::default(),
        };

        assert_eq!(optional_plan_output_path(&plan, "corrected_reads_r2"), None);
    }

    #[test]
    fn thread_override_replaces_governed_tool_threads() {
        let spec = ToolExecutionSpecV1 {
            tool_id: ToolId::from_static("musket"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
            command: CommandSpecV1 { template: vec!["musket".to_string()] },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 4,
                tmp_gb: 1,
                threads: 12,
            },
        };
        let overridden = apply_thread_override(&spec, Some(7));
        assert_eq!(overridden.resources.threads, 7);
    }

    #[test]
    fn benchmark_projection_strips_lighter_only_fields_for_musket() {
        let projected = project_correct_options_for_tool(
            "musket",
            &CorrectPlanOptions {
                threads: Some(8),
                kmer_size: Some(31),
                musket_kmer_budget: Some(536_870_912),
                genome_size: Some(3_200_000),
                max_memory_gb: Some(24),
                trusted_kmer_artifact: Some(Path::new("trusted.kmers").to_path_buf()),
                ..CorrectPlanOptions::baseline()
            },
        );

        assert_eq!(projected.threads, Some(8));
        assert_eq!(projected.kmer_size, Some(31));
        assert_eq!(projected.musket_kmer_budget, Some(536_870_912));
        assert_eq!(projected.genome_size, None);
        assert_eq!(projected.max_memory_gb, None);
        assert_eq!(projected.trusted_kmer_artifact, None);
    }

    #[test]
    fn correction_report_carries_effective_params_and_effect_deltas() {
        let metrics = correct_metrics_from_observed_stats(
            &seqkit_metrics(100, 10_000, 30.0),
            None,
            &seqkit_metrics(100, 9_700, 33.0),
            None,
            true,
        );
        let report = build_correction_report(
            "lighter",
            std::path::Path::new("reads.fastq.gz"),
            None,
            std::path::Path::new("corrected.fastq.gz"),
            None,
            std::path::Path::new("correct_report.json"),
            &serde_json::json!({
                "schema_version": bijux_dna_domain_fastq::params::correct::CORRECT_SCHEMA_VERSION,
                "paired_mode": "single_end",
                "threads": 8,
                "correction_engine": "lighter",
                "quality_encoding": "phred33",
                "musket_kmer_budget": null,
                "genome_size": 3_200_000_u64,
                "trusted_kmer_artifact": "trusted.kmers",
                "conservative_mode": false,
            }),
            &metrics,
            &StageResultV1 {
                run_id: "run-1".to_string(),
                runtime_s: 12.5,
                memory_mb: 768.0,
                exit_code: 0,
                outputs: Vec::new(),
                metrics_path: None,
                stdout: String::new(),
                stderr: String::new(),
                command: "lighter".to_string(),
            },
            true,
        )
        .expect("correction report should build");

        assert_eq!(report.tool_id, "lighter");
        assert_eq!(report.threads, 8);
        assert_eq!(
            report.correction_engine,
            bijux_dna_domain_fastq::params::correct::CorrectionEngine::Lighter
        );
        assert_eq!(report.trusted_kmer_artifact, Some(std::path::PathBuf::from("trusted.kmers")));
        assert_eq!(
            report
                .correction_effect
                .as_ref()
                .and_then(|effect| effect.get("outputs_changed"))
                .cloned(),
            Some(serde_json::json!(true))
        );
        assert_eq!(
            report.correction_effect.as_ref().and_then(|effect| effect.get("bases_delta")).cloned(),
            Some(serde_json::json!(-300_i128))
        );
        assert_eq!(report.exit_code, Some(0));
    }
}
