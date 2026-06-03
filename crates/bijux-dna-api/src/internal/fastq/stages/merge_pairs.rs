use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::quality::{fetch_fastq_merge_v1, insert_fastq_merge_v1};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqMergeMetrics, MetricSet};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::params_hash;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::metrics::ratio_u64;
use bijux_dna_domain_fastq::params::merge::{MergeEffectiveParams, UnmergedReadPolicy};
use bijux_dna_domain_fastq::{MergePairsReportV1, MERGE_PAIRS_REPORT_SCHEMA_VERSION};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, ensure_dir, hash_file_sha256};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::select_merge_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::merge_pairs::{
    plan_merge_with_options, MergePlanOptions,
};
use bijux_dna_planner_fastq::stage_api::FastqArtifactKind;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use bijux_dna_stage_contract::StagePlanV1;

use crate::internal::fastq::stages::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context, observe_fastq_stats,
};
use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_MERGE_PAIRS,
};
use serde::Serialize;

const LOCAL_MERGE_PAIRS_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.merge_pairs.local_smoke.report.v1";

#[derive(Debug, Clone, Serialize)]
struct LocalMergePairsSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    planned_tool_id: String,
    report_tool_id: String,
    merge_overlap: u32,
    min_length: u32,
    input_pair_count: u64,
    merged_count: u64,
    unmerged_r1_count: u64,
    unmerged_r2_count: u64,
    discarded_count: u64,
    merged_fastq_gz: String,
    unmerged_r1_fastq_gz: Option<String>,
    unmerged_r2_fastq_gz: Option<String>,
    case_report_json: String,
    raw_backend_report: Option<String>,
}

fn parse_unmerged_read_policy(raw: Option<&str>) -> Result<UnmergedReadPolicy> {
    match raw.unwrap_or("emit_unmerged_pairs") {
        "emit_unmerged_pairs" => Ok(UnmergedReadPolicy::EmitUnmergedPairs),
        "omit_unmerged_pairs" => Ok(UnmergedReadPolicy::OmitUnmergedPairs),
        other => Err(anyhow!("unsupported fastq.merge_pairs unmerged_read_policy `{other}`")),
    }
}

fn merge_plan_options(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqMergeArgs,
) -> Result<MergePlanOptions> {
    Ok(MergePlanOptions {
        threads: args.threads,
        merge_overlap: args.merge_overlap,
        min_length: args.min_length,
        unmerged_read_policy: parse_unmerged_read_policy(args.unmerged_read_policy.as_deref())?,
    })
}

fn load_governed_merge_report(report_path: &Path) -> Result<MergePairsReportV1> {
    let raw = std::fs::read_to_string(report_path)
        .with_context(|| format!("read governed merge report {}", report_path.display()))?;
    bijux_dna_domain_fastq::observer::parse_merge_pairs_report(&raw)
        .with_context(|| format!("parse governed merge report {}", report_path.display()))
}

fn write_governed_merge_report(report_path: &Path, report: &MergePairsReportV1) -> Result<()> {
    bijux_dna_infra::atomic_write_json(report_path, report)
        .with_context(|| format!("write governed merge report {}", report_path.display()))
}

fn required_plan_output_path<'a>(
    plan: &'a bijux_dna_stage_contract::StagePlanV1,
    artifact_name: &str,
) -> Result<&'a Path> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_name)
        .map(|artifact| artifact.path.as_path())
        .ok_or_else(|| anyhow!("planned merge output `{artifact_name}` missing"))
}

fn resolve_merge_outputs(plan: &StagePlanV1) -> Result<MergePlanOutputs<'_>> {
    let outputs = MergePlanOutputs {
        merged_reads: required_plan_output_path(plan, "merged_reads")?,
        report_json: required_plan_output_path(plan, "report_json")?,
    };
    validate_merge_output_paths(&outputs)?;
    Ok(outputs)
}

fn validate_merge_output_paths(outputs: &MergePlanOutputs<'_>) -> Result<()> {
    let mut paths = BTreeSet::new();
    for path in [outputs.merged_reads, outputs.report_json] {
        if !paths.insert(path) {
            return Err(anyhow!("merge output path reused: {}", path.display()));
        }
    }
    Ok(())
}

fn materialize_local_merge_pairs_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_fastq::LocalMergePairsSmokeCasePlan,
    output_root: &Path,
) -> Result<LocalMergePairsSmokeReport> {
    let effective_params =
        serde_json::from_value::<MergeEffectiveParams>(case.plan.effective_params.clone())
            .context("decode merge pairs local-smoke effective params")?;
    let input_r1 = repo_root.join(&case.r1);
    let input_r2 = repo_root.join(&case.r2);
    let case_merged_reads =
        resolve_output_path(repo_root, required_plan_output_path(&case.plan, "merged_reads")?);
    let case_report_json =
        resolve_output_path(repo_root, required_plan_output_path(&case.plan, "report_json")?);
    let case_unmerged_r1 = optional_plan_output_path(&case.plan, "unmerged_reads_r1")
        .map(|path| resolve_output_path(repo_root, path));
    let case_unmerged_r2 = optional_plan_output_path(&case.plan, "unmerged_reads_r2")
        .map(|path| resolve_output_path(repo_root, path));
    let case_raw_backend_report = optional_plan_param_path(&case.plan, "raw_backend_report_txt")
        .map(|path| resolve_output_path(repo_root, &path));

    for path in [&case_merged_reads, &case_report_json] {
        if let Some(parent) = path.parent() {
            bijux_dna_infra::ensure_dir(parent)?;
        }
    }
    for path in
        [case_unmerged_r1.as_ref(), case_unmerged_r2.as_ref(), case_raw_backend_report.as_ref()]
            .into_iter()
            .flatten()
    {
        if let Some(parent) = path.parent() {
            bijux_dna_infra::ensure_dir(parent)?;
        }
    }

    let mut report = bijux_dna_domain_fastq::stages::contract::merge_pairs(
        &input_r1,
        &input_r2,
        &effective_params,
        &case_merged_reads,
        case_unmerged_r1.as_deref(),
        case_unmerged_r2.as_deref(),
        &case_report_json,
        case_raw_backend_report.as_deref(),
    )?;

    report.input_r1 = case.r1.display().to_string();
    report.input_r2 = case.r2.display().to_string();
    report.merged_reads = path_relative_to_repo(repo_root, &case_merged_reads);
    report.unmerged_reads_r1 =
        case_unmerged_r1.as_ref().map(|path| path_relative_to_repo(repo_root, path));
    report.unmerged_reads_r2 =
        case_unmerged_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path));
    report.raw_backend_report =
        case_raw_backend_report.as_ref().map(|path| path_relative_to_repo(repo_root, path));
    write_governed_merge_report(&case_report_json, &report)?;

    let top_level_merged = output_root.join("merged.fastq.gz");
    copy_fastq_artifact(&case_merged_reads, &top_level_merged)?;
    let top_level_unmerged_r1 =
        case_unmerged_r1.as_ref().map(|_| output_root.join("unmerged/R1.fastq.gz"));
    let top_level_unmerged_r2 =
        case_unmerged_r2.as_ref().map(|_| output_root.join("unmerged/R2.fastq.gz"));
    if let (Some(source), Some(destination)) =
        (case_unmerged_r1.as_ref(), top_level_unmerged_r1.as_ref())
    {
        copy_fastq_artifact(source, destination)?;
    }
    if let (Some(source), Some(destination)) =
        (case_unmerged_r2.as_ref(), top_level_unmerged_r2.as_ref())
    {
        copy_fastq_artifact(source, destination)?;
    }

    let input_pair_count = report.reads_r1.min(report.reads_r2);
    let merged_count = report.reads_merged.min(input_pair_count);
    let unmerged_count = report.reads_unmerged.min(input_pair_count.saturating_sub(merged_count));
    let discarded_count = input_pair_count.saturating_sub(merged_count + unmerged_count);

    Ok(LocalMergePairsSmokeReport {
        schema_version: LOCAL_MERGE_PAIRS_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: STAGE_MERGE_PAIRS.as_str().to_string(),
        sample_id: case.sample_id.clone(),
        planned_tool_id: case.plan.tool_id.as_str().to_string(),
        report_tool_id: report.tool_id,
        merge_overlap: effective_params.merge_overlap.unwrap_or(case.merge_overlap),
        min_length: effective_params.min_len.unwrap_or(case.min_length),
        input_pair_count,
        merged_count,
        unmerged_r1_count: unmerged_count,
        unmerged_r2_count: unmerged_count,
        discarded_count,
        merged_fastq_gz: path_relative_to_repo(repo_root, &top_level_merged),
        unmerged_r1_fastq_gz: top_level_unmerged_r1
            .as_ref()
            .map(|path| path_relative_to_repo(repo_root, path)),
        unmerged_r2_fastq_gz: top_level_unmerged_r2
            .as_ref()
            .map(|path| path_relative_to_repo(repo_root, path)),
        case_report_json: path_relative_to_repo(repo_root, &case_report_json),
        raw_backend_report: case_raw_backend_report
            .as_ref()
            .map(|path| path_relative_to_repo(repo_root, path)),
    })
}

fn optional_plan_output_path<'a>(plan: &'a StagePlanV1, artifact_name: &str) -> Option<&'a Path> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_name)
        .map(|artifact| artifact.path.as_path())
}

fn optional_plan_param_path(plan: &StagePlanV1, key: &str) -> Option<PathBuf> {
    plan.params.get(key).and_then(serde_json::Value::as_str).map(PathBuf::from)
}

fn resolve_output_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

fn copy_fastq_artifact(source: &Path, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    let source_is_gz = is_gzip_path(source);
    let destination_is_gz = is_gzip_path(destination);
    if source_is_gz == destination_is_gz {
        std::fs::copy(source, destination).with_context(|| {
            format!(
                "copy local merge smoke artifact from {} to {}",
                source.display(),
                destination.display()
            )
        })?;
        return Ok(());
    }

    let input = std::fs::File::open(source)
        .with_context(|| format!("open local merge smoke artifact {}", source.display()))?;
    let output = std::fs::File::create(destination)
        .with_context(|| format!("create local merge smoke artifact {}", destination.display()))?;

    if destination_is_gz {
        let mut reader: Box<dyn std::io::Read> = if source_is_gz {
            Box::new(flate2::read::MultiGzDecoder::new(input))
        } else {
            Box::new(std::io::BufReader::new(input))
        };
        let mut writer = flate2::write::GzEncoder::new(output, flate2::Compression::default());
        std::io::copy(&mut reader, &mut writer).with_context(|| {
            format!(
                "compress local merge smoke artifact from {} to {}",
                source.display(),
                destination.display()
            )
        })?;
        writer.finish()?;
    } else {
        let mut reader = flate2::read::MultiGzDecoder::new(input);
        let mut writer = std::io::BufWriter::new(output);
        std::io::copy(&mut reader, &mut writer).with_context(|| {
            format!(
                "decompress local merge smoke artifact from {} to {}",
                source.display(),
                destination.display()
            )
        })?;
    }
    Ok(())
}

fn is_gzip_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("gz"))
}

/// Benchmark FASTQ read-merging tools under governed stage contracts.
///
/// # Errors
/// Returns an error if planning, execution, report parsing, or persistence fails.
pub fn bench_fastq_merge<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqMergeArgs,
) -> Result<BenchOutcome<FastqMergeMetrics>> {
    let selected_tools = select_merge_benchmark_tools(args)?;
    let setup =
        prepare_merge_benchmark_setup(catalog, platform, runner_override, args, &selected_tools)?;

    if args.explain {
        write_merge_benchmark_explain(&setup)?;
    }

    ensure_merge_benchmark_qa(catalog, platform, &setup.tools)?;

    let sqlite_path = setup.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = setup.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::<BenchmarkRecord<FastqMergeMetrics>>::new();

    for tool in &setup.tools {
        let tool_plan = prepare_merge_tool_plan(catalog, platform, args, &setup, jobs, tool)?;
        if let Ok(Some(record)) = fetch_fastq_merge_v1(
            &conn,
            tool,
            &tool_plan.tool_spec.tool_version,
            &tool_plan.image_digest,
            &setup.runner.to_string(),
            &platform.name,
            &setup.input_hash,
            &tool_plan.params_hash,
        ) {
            records.push(record);
            continue;
        }
        let execution = execute_merge_tool(&tool_plan, setup.runner, jobs, tool)?;
        if let Some(failure) = merge_tool_failure(tool, &execution) {
            failures.push(failure);
            continue;
        }
        let outputs = resolve_merge_outputs(&tool_plan.plan)?;
        let record = build_merge_record(&MergeRecordInputs {
            catalog,
            platform,
            runner: setup.runner,
            input_hash: &setup.input_hash,
            r1_stats: &setup.r1_stats,
            r2_stats: &setup.r2_stats,
            tool_spec: &tool_plan.tool_spec,
            image_digest: &tool_plan.image_digest,
            params: &tool_plan.plan.params,
            merged_reads: outputs.merged_reads,
            report_path: outputs.report_json,
            execution: &execution,
        })?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_merge_v1(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome { records, failures, bench_dir: setup.bench_dir, explain: args.explain })
}

struct MergeBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    runner: RuntimeKind,
    bench_dir: std::path::PathBuf,
    tools_root: std::path::PathBuf,
    input_hash: String,
    r1_stats: SeqkitMetrics,
    r2_stats: SeqkitMetrics,
    options: MergePlanOptions,
}

struct MergeToolPlan {
    tool_spec: ToolExecutionSpecV1,
    plan: StagePlanV1,
    params_hash: String,
    image_digest: String,
}

struct MergePlanOutputs<'a> {
    merged_reads: &'a Path,
    report_json: &'a Path,
}

/// Materialize the governed local-smoke `fastq.merge_pairs` artifacts.
///
/// The written summary artifact lives at `target/local-smoke/fastq.merge_pairs/report.json`
/// under the active repository root, alongside top-level `merged.fastq.gz` and `unmerged/`
/// outputs.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_merge_pairs_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_fastq::stage_api::local_merge_pairs_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "local-smoke fastq.merge_pairs expects exactly one governed case, found {}",
            cases.len()
        ));
    };

    let output_root = repo_root.join("target/local-smoke/fastq.merge_pairs");
    let unmerged_root = output_root.join("unmerged");
    bijux_dna_infra::ensure_dir(&output_root)?;
    bijux_dna_infra::ensure_dir(&unmerged_root)?;

    let summary = materialize_local_merge_pairs_smoke_case(&repo_root, case, &output_root)?;
    let report_path = output_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    Ok(report_path)
}

struct MergeRecordInputs<'a, S: ::std::hash::BuildHasher> {
    catalog: &'a HashMap<String, ToolImageSpec, S>,
    platform: &'a PlatformSpec,
    runner: RuntimeKind,
    input_hash: &'a str,
    r1_stats: &'a SeqkitMetrics,
    r2_stats: &'a SeqkitMetrics,
    tool_spec: &'a ToolExecutionSpecV1,
    image_digest: &'a str,
    params: &'a serde_json::Value,
    merged_reads: &'a Path,
    report_path: &'a Path,
    execution: &'a StageResultV1,
}

fn select_merge_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqMergeArgs,
) -> Result<Vec<String>> {
    let tools = select_merge_tools(&args.tools)?;
    preflight_stage(STAGE_MERGE_PAIRS.as_str(), FastqArtifactKind::PairedEnd)?;
    let header = inspect_headers(&args.r1, Some(&args.r2), false)?;
    log_header_warnings(STAGE_MERGE_PAIRS.as_str(), &header);
    Ok(tools)
}

fn prepare_merge_benchmark_setup<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqMergeArgs,
    selected_tools: &[String],
) -> Result<MergeBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_MERGE_PAIRS.as_str(), selected_tools, &registry, false)?;
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir_name = bench_dir_name(&STAGE_MERGE_PAIRS)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_MERGE_PAIRS.as_str()))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    ensure_dir(&bench_dir).context("create bench output dir")?;
    ensure_dir(&tools_root).context("create tools output dir")?;
    let input_hash = merge_input_hash(&args.r1, &args.r2)?;
    let r1_stats = observe_fastq_stats(catalog, platform, runner, &args.r1)?;
    let r2_stats = observe_fastq_stats(catalog, platform, runner, &args.r2)?;
    let options = merge_plan_options(args)?;

    Ok(MergeBenchmarkSetup {
        registry,
        tools,
        runner,
        bench_dir,
        tools_root,
        input_hash,
        r1_stats,
        r2_stats,
        options,
    })
}

fn write_merge_benchmark_explain(setup: &MergeBenchmarkSetup) -> Result<()> {
    write_explain_md(&setup.bench_dir, STAGE_MERGE_PAIRS.as_str(), &setup.tools, &[], None)?;
    write_explain_plan_json(
        &setup.bench_dir,
        STAGE_MERGE_PAIRS.as_str(),
        &setup.tools,
        &setup.registry,
        None,
    )
}

fn ensure_merge_benchmark_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(STAGE_MERGE_PAIRS.as_str(), tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_MERGE_PAIRS.as_str(), tools, platform, catalog)
}

fn prepare_merge_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqMergeArgs,
    setup: &MergeBenchmarkSetup,
    jobs: usize,
    tool: &str,
) -> Result<MergeToolPlan> {
    let out_dir = setup.tools_root.join(tool);
    ensure_dir(&out_dir).context("create tool output dir")?;
    let tool_spec = build_tool_execution_spec(
        STAGE_MERGE_PAIRS.as_str(),
        tool,
        &setup.registry,
        catalog,
        platform,
    )?;
    let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
    let plan = plan_merge_with_options(&tool_spec, &args.r1, &args.r2, &out_dir, &setup.options)?;
    let params_hash = stable_params_hash(&plan.params);
    let image_digest = benchmark_image_identity(&tool_spec);

    Ok(MergeToolPlan { tool_spec, plan, params_hash, image_digest })
}

fn execute_merge_tool(
    tool_plan: &MergeToolPlan,
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

fn merge_tool_failure(tool: &str, execution: &StageResultV1) -> Option<RawFailure> {
    (execution.exit_code != 0).then(|| {
        let stderr = execution.stderr.trim();
        let reason = if stderr.is_empty() {
            format!("tool {tool} failed with status {}", execution.exit_code)
        } else {
            format!("tool {tool} failed with status {}: {stderr}", execution.exit_code)
        };
        RawFailure {
            stage: STAGE_MERGE_PAIRS.as_str().to_string(),
            tool: tool.to_string(),
            reason,
            category: ErrorCategory::ToolError,
        }
    })
}

fn build_merge_record<S: ::std::hash::BuildHasher>(
    inputs: &MergeRecordInputs<'_, S>,
) -> Result<BenchmarkRecord<FastqMergeMetrics>> {
    let catalog = inputs.catalog;
    let platform = inputs.platform;
    let runner = inputs.runner;
    let input_hash = inputs.input_hash;
    let r1_stats = inputs.r1_stats;
    let r2_stats = inputs.r2_stats;
    let tool_spec = inputs.tool_spec;
    let image_digest = inputs.image_digest;
    let params = inputs.params;
    let merged_reads = inputs.merged_reads;
    let report_path = inputs.report_path;
    let execution = inputs.execution;
    let merged_stats = observe_merge_stats(catalog, platform, runner, merged_reads)?;
    let report = merge_report_with_execution(report_path, execution)?;
    validate_merge_report_identity(&tool_spec.tool_id.0, &report)?;
    validate_merge_report_execution(&report, execution.runtime_s, execution.memory_mb)?;
    validate_merge_report_paths(&report, merged_reads)?;
    validate_merge_report_observed_counts(&report, r1_stats, r2_stats, &merged_stats)?;
    validate_merge_report_rate(&report)?;

    let metrics = merge_metrics_from_report(&report, r1_stats, r2_stats, &merged_stats);
    let metric_set = metric_set(metrics.clone());
    validate_merge_report_metrics(&report, &metrics)?;
    bijux_dna_analyze::validate_metric_set(&metric_set)?;
    let out_dir = report_path.parent().ok_or_else(|| anyhow!("merge report has no parent"))?;
    write_merge_artifacts(out_dir, report_path, &report, &metric_set)?;

    let context = build_benchmark_context(
        &report.tool_id,
        tool_spec.tool_version.clone(),
        image_digest.to_string(),
        runner,
        platform,
        input_hash.to_string(),
        params.clone(),
    );
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

fn write_merge_artifacts(
    out_dir: &Path,
    report_path: &Path,
    report: &MergePairsReportV1,
    metric_set: &MetricSet<FastqMergeMetrics>,
) -> Result<()> {
    let metrics_path = out_dir.join("metrics.json");
    let metrics_json = serde_json::to_value(metric_set)?;
    bijux_dna_infra::atomic_write_json(&metrics_path, &metrics_json)
        .context("write merge metrics")?;
    validate_merge_pre_prune_artifacts(report_path, &metrics_path, report)?;
    prune_merge_tool_payload(out_dir, report_path, &metrics_path, report)?;
    validate_merge_retained_artifacts(report_path, &metrics_path, report)
}

fn merge_metrics_from_report(
    report: &MergePairsReportV1,
    r1_stats: &SeqkitMetrics,
    r2_stats: &SeqkitMetrics,
    merged_stats: &SeqkitMetrics,
) -> FastqMergeMetrics {
    let pairs_in = report.reads_r1.min(report.reads_r2);
    let reads_merged = report.reads_merged.min(pairs_in);
    let reads_unmerged = report.reads_unmerged.min(pairs_in.saturating_sub(reads_merged));

    FastqMergeMetrics {
        reads_in: report.reads_r1 + report.reads_r2,
        reads_out: reads_merged,
        bases_in: r1_stats.bases + r2_stats.bases,
        bases_out: merged_stats.bases,
        pairs_in,
        pairs_out: reads_merged,
        reads_r1: report.reads_r1,
        reads_r2: report.reads_r2,
        reads_merged,
        reads_unmerged,
        merge_rate: report.merge_rate,
    }
}

fn merge_report_with_execution(
    report_path: &Path,
    execution: &StageResultV1,
) -> Result<MergePairsReportV1> {
    let mut report = load_governed_merge_report(report_path)?;
    report.runtime_s = Some(execution.runtime_s);
    report.memory_mb = Some(execution.memory_mb);
    write_governed_merge_report(report_path, &report)?;
    Ok(report)
}

fn validate_merge_report_identity(tool: &str, report: &MergePairsReportV1) -> Result<()> {
    if report.schema_version != MERGE_PAIRS_REPORT_SCHEMA_VERSION {
        return Err(anyhow!(
            "merge_pairs report schema mismatch: expected {}, observed {}",
            MERGE_PAIRS_REPORT_SCHEMA_VERSION,
            report.schema_version
        ));
    }
    if report.stage != STAGE_MERGE_PAIRS.as_str() || report.stage_id != STAGE_MERGE_PAIRS.as_str() {
        return Err(anyhow!(
            "merge_pairs report stage mismatch: observed stage={} stage_id={}",
            report.stage,
            report.stage_id
        ));
    }
    if report.tool_id != tool {
        return Err(anyhow!(
            "merge_pairs report tool mismatch: expected {}, observed {}",
            tool,
            report.tool_id
        ));
    }
    Ok(())
}

fn validate_merge_report_paths(report: &MergePairsReportV1, merged_reads: &Path) -> Result<()> {
    let report_merged_reads = Path::new(&report.merged_reads);
    if report_merged_reads != merged_reads {
        return Err(anyhow!(
            "merge_pairs report merged_reads mismatch: expected {}, observed {}",
            merged_reads.display(),
            report_merged_reads.display()
        ));
    }
    Ok(())
}

fn validate_merge_report_execution(
    report: &MergePairsReportV1,
    runtime_s: f64,
    memory_mb: f64,
) -> Result<()> {
    if report.runtime_s.is_none_or(|observed| (observed - runtime_s).abs() > f64::EPSILON) {
        return Err(anyhow!(
            "merge_pairs report runtime mismatch: expected {}, observed {:?}",
            runtime_s,
            report.runtime_s
        ));
    }
    if report.memory_mb.is_none_or(|observed| (observed - memory_mb).abs() > f64::EPSILON) {
        return Err(anyhow!(
            "merge_pairs report memory mismatch: expected {}, observed {:?}",
            memory_mb,
            report.memory_mb
        ));
    }
    Ok(())
}

fn validate_merge_report_observed_counts(
    report: &MergePairsReportV1,
    r1_stats: &SeqkitMetrics,
    r2_stats: &SeqkitMetrics,
    merged_stats: &SeqkitMetrics,
) -> Result<()> {
    if report.reads_r1 != r1_stats.reads {
        return Err(anyhow!(
            "merge_pairs report r1 read count mismatch: expected {}, observed {}",
            r1_stats.reads,
            report.reads_r1
        ));
    }
    if report.reads_r2 != r2_stats.reads {
        return Err(anyhow!(
            "merge_pairs report r2 read count mismatch: expected {}, observed {}",
            r2_stats.reads,
            report.reads_r2
        ));
    }
    if report.reads_merged != merged_stats.reads {
        return Err(anyhow!(
            "merge_pairs report merged read count mismatch: expected {}, observed {}",
            merged_stats.reads,
            report.reads_merged
        ));
    }
    Ok(())
}

fn validate_merge_report_rate(report: &MergePairsReportV1) -> Result<()> {
    let pairs_in = report.reads_r1.min(report.reads_r2);
    let expected = ratio_u64(report.reads_merged, pairs_in);
    if (report.merge_rate - expected).abs() > 0.000_001 {
        return Err(anyhow!(
            "merge_pairs report merge_rate arithmetic mismatch: expected {}, observed {}",
            expected,
            report.merge_rate
        ));
    }
    Ok(())
}

fn validate_merge_report_metrics(
    report: &MergePairsReportV1,
    metrics: &FastqMergeMetrics,
) -> Result<()> {
    if report.reads_r1 != metrics.reads_r1 {
        return Err(anyhow!(
            "merge_pairs report reads_r1 mismatch: expected {}, observed {}",
            metrics.reads_r1,
            report.reads_r1
        ));
    }
    if report.reads_r2 != metrics.reads_r2 {
        return Err(anyhow!(
            "merge_pairs report reads_r2 mismatch: expected {}, observed {}",
            metrics.reads_r2,
            report.reads_r2
        ));
    }
    if report.reads_merged != metrics.reads_merged {
        return Err(anyhow!(
            "merge_pairs report reads_merged mismatch: expected {}, observed {}",
            metrics.reads_merged,
            report.reads_merged
        ));
    }
    if report.reads_unmerged != metrics.reads_unmerged {
        return Err(anyhow!(
            "merge_pairs report reads_unmerged mismatch: expected {}, observed {}",
            metrics.reads_unmerged,
            report.reads_unmerged
        ));
    }
    if (report.merge_rate - metrics.merge_rate).abs() > f64::EPSILON {
        return Err(anyhow!(
            "merge_pairs report merge_rate mismatch: expected {}, observed {}",
            metrics.merge_rate,
            report.merge_rate
        ));
    }
    Ok(())
}

fn validate_merge_pre_prune_artifacts(
    report_path: &Path,
    metrics_path: &Path,
    report: &MergePairsReportV1,
) -> Result<()> {
    validate_merge_nonempty_artifact(report_path)?;
    validate_merge_nonempty_artifact(metrics_path)?;
    if report.reads_merged > 0 {
        validate_merge_nonempty_artifact(Path::new(&report.merged_reads))?;
    }
    if let Some(raw_backend_report) = report.raw_backend_report.as_ref() {
        validate_merge_artifact_exists(Path::new(raw_backend_report))?;
    }
    Ok(())
}

fn validate_merge_retained_artifacts(
    report_path: &Path,
    metrics_path: &Path,
    report: &MergePairsReportV1,
) -> Result<()> {
    validate_merge_nonempty_artifact(report_path)?;
    validate_merge_nonempty_artifact(metrics_path)?;
    if let Some(raw_backend_report) = report.raw_backend_report.as_ref() {
        validate_merge_artifact_exists(Path::new(raw_backend_report))?;
    }
    Ok(())
}

fn validate_merge_artifact_exists(path: &Path) -> Result<()> {
    fs::metadata(path).with_context(|| format!("read merge artifact {}", path.display()))?;
    Ok(())
}

fn validate_merge_nonempty_artifact(path: &Path) -> Result<()> {
    let metadata =
        fs::metadata(path).with_context(|| format!("read merge artifact {}", path.display()))?;
    if metadata.len() == 0 {
        return Err(anyhow!("merge artifact is empty: {}", path.display()));
    }
    Ok(())
}

fn prune_merge_tool_payload(
    out_dir: &Path,
    report_path: &Path,
    metrics_path: &Path,
    report: &MergePairsReportV1,
) -> Result<()> {
    let run_artifacts_dir = out_dir.join("run_artifacts");
    let mut keep = HashSet::new();
    keep.insert(report_path.to_path_buf());
    keep.insert(metrics_path.to_path_buf());
    if let Some(raw_backend_report) = report.raw_backend_report.as_ref() {
        keep.insert(Path::new(raw_backend_report).to_path_buf());
    }

    let mut dirs = vec![out_dir.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        for entry in
            fs::read_dir(&dir).with_context(|| format!("read merge tool dir {}", dir.display()))?
        {
            let path = entry.with_context(|| format!("read entry in {}", dir.display()))?.path();
            if path == run_artifacts_dir || path.starts_with(&run_artifacts_dir) {
                continue;
            }
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            if keep.contains(&path) {
                continue;
            }
            fs::remove_file(&path)
                .with_context(|| format!("prune merge payload {}", path.display()))?;
        }
    }

    Ok(())
}

fn merge_input_hash(r1: &Path, r2: &Path) -> Result<String> {
    let r1_hash = hash_file_sha256(r1).context("hash merge r1")?;
    let r2_hash = hash_file_sha256(r2).context("hash merge r2")?;
    params_hash(&serde_json::json!({ "r1": r1_hash, "r2": r2_hash }))
        .context("combine paired merge input hashes")
}

fn observe_merge_stats<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner: RuntimeKind,
    merged_reads: &Path,
) -> Result<bijux_dna_core::prelude::measure::SeqkitMetrics> {
    observe_fastq_stats(catalog, platform, runner, merged_reads)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::{merge_plan_options, parse_unmerged_read_policy, prune_merge_tool_payload};
    use bijux_dna_domain_fastq::params::merge::MergeEngine;
    use bijux_dna_domain_fastq::params::PairedMode;
    use bijux_dna_domain_fastq::MergePairsReportV1;
    use bijux_dna_planner_fastq::stage_api::args::BenchFastqMergeArgs;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn merge_plan_options_carry_declared_policy_surface() {
        let args = BenchFastqMergeArgs {
            sample_id: "sample-a".to_string(),
            r1: PathBuf::from("reads_R1.fastq.gz"),
            r2: PathBuf::from("reads_R2.fastq.gz"),
            out: PathBuf::from("out"),
            tools: vec!["pear".to_string()],
            explain: false,
            threads: Some(7),
            merge_overlap: Some(22),
            min_length: Some(120),
            unmerged_read_policy: Some("omit_unmerged_pairs".to_string()),
            replicates: 1,
            jobs: 1,
            ci_bootstrap: None,
        };

        let options = merge_plan_options(&args).expect("merge plan options");
        assert_eq!(options.threads, Some(7));
        assert_eq!(options.merge_overlap, Some(22));
        assert_eq!(options.min_length, Some(120));
        assert_eq!(
            serde_json::to_value(options.unmerged_read_policy).expect("policy json"),
            serde_json::json!("omit_unmerged_pairs")
        );
    }

    #[test]
    fn unsupported_unmerged_policy_is_rejected() {
        let err = parse_unmerged_read_policy(Some("maybe"))
            .expect_err("invalid unmerged policy must fail");
        assert!(err
            .to_string()
            .contains("unsupported fastq.merge_pairs unmerged_read_policy `maybe`"));
    }

    #[test]
    fn prune_merge_tool_payload_keeps_reports_and_run_artifacts() {
        let temp = tempdir().expect("tempdir");
        let out_dir = temp.path().join("pear");
        let run_artifacts = out_dir.join("run_artifacts");
        fs::create_dir_all(&run_artifacts).expect("mkdir");

        let merged_reads = out_dir.join("pear.assembled.fastq");
        let unmerged_r1 = out_dir.join("pear.unassembled.forward.fastq");
        let unmerged_r2 = out_dir.join("pear.unassembled.reverse.fastq");
        let discarded = out_dir.join("pear.discarded.fastq");
        let raw_backend_report = out_dir.join("pear.log");
        let report_path = out_dir.join("merge_report.json");
        let metrics_path = out_dir.join("metrics.json");
        let stage_report = run_artifacts.join("stage_report.json");

        for path in [
            &merged_reads,
            &unmerged_r1,
            &unmerged_r2,
            &discarded,
            &raw_backend_report,
            &report_path,
            &metrics_path,
            &stage_report,
        ] {
            fs::write(path, "{}").expect("write fixture");
        }

        let report = MergePairsReportV1 {
            schema_version: "bijux.fastq.merge_pairs.report.v2".to_string(),
            stage: "fastq.merge_pairs".to_string(),
            stage_id: "fastq.merge_pairs".to_string(),
            tool_id: "pear".to_string(),
            paired_mode: PairedMode::PairedEnd,
            merge_engine: MergeEngine::Pear,
            threads: 1,
            merge_overlap: None,
            min_len: None,
            unmerged_read_policy:
                bijux_dna_domain_fastq::params::merge::UnmergedReadPolicy::EmitUnmergedPairs,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: "reads_R2.fastq.gz".to_string(),
            merged_reads: merged_reads.display().to_string(),
            unmerged_reads_r1: Some(unmerged_r1.display().to_string()),
            unmerged_reads_r2: Some(unmerged_r2.display().to_string()),
            reads_r1: 10,
            reads_r2: 10,
            reads_merged: 8,
            reads_unmerged: 2,
            merge_rate: 0.8,
            runtime_s: Some(1.0),
            memory_mb: Some(8.0),
            raw_backend_report: Some(raw_backend_report.display().to_string()),
            raw_backend_report_format: Some("pear_log".to_string()),
        };

        prune_merge_tool_payload(&out_dir, &report_path, &metrics_path, &report)
            .expect("prune merge payload");

        assert!(report_path.is_file());
        assert!(metrics_path.is_file());
        assert!(raw_backend_report.is_file());
        assert!(stage_report.is_file());
        assert!(!merged_reads.exists());
        assert!(!unmerged_r1.exists());
        assert!(!unmerged_r2.exists());
        assert!(!discarded.exists());
    }
}
