use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::quality::{fetch_fastq_merge_v1, insert_fastq_merge_v1};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqMergeMetrics};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::params_hash;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::params::merge::UnmergedReadPolicy;
use bijux_dna_domain_fastq::MergePairsReportV1;
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

/// Benchmark FASTQ read-merging tools under governed stage contracts.
///
/// # Errors
/// Returns an error if planning, execution, report parsing, or persistence fails.
#[allow(clippy::too_many_lines)]
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
        if let Some(failure) = merge_tool_failure(tool, execution.exit_code) {
            failures.push(failure);
            continue;
        }
        let merged_reads = required_plan_output_path(&tool_plan.plan, "merged_reads")?;
        let report_json = required_plan_output_path(&tool_plan.plan, "report_json")?;
        let record = build_merge_record(
            catalog,
            platform,
            setup.runner,
            &setup.input_hash,
            &setup.r1_stats,
            &setup.r2_stats,
            &tool_plan.tool_spec,
            &tool_plan.plan.params,
            merged_reads,
            report_json,
            &execution,
        )?;
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
    let params_hash = params_hash(&plan.params).context("hash merge plan params")?;
    let image_digest = tool_spec
        .image
        .digest
        .as_ref()
        .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
        .clone();

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

fn merge_tool_failure(tool: &str, exit_code: i32) -> Option<RawFailure> {
    (exit_code != 0).then(|| RawFailure {
        stage: STAGE_MERGE_PAIRS.as_str().to_string(),
        tool: tool.to_string(),
        reason: format!("tool {tool} failed with status {exit_code}"),
        category: ErrorCategory::ToolError,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_merge_record<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner: RuntimeKind,
    input_hash: &str,
    r1_stats: &bijux_dna_core::prelude::measure::SeqkitMetrics,
    r2_stats: &bijux_dna_core::prelude::measure::SeqkitMetrics,
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    params: &serde_json::Value,
    merged_reads: &Path,
    report_path: &Path,
    execution: &StageResultV1,
) -> Result<BenchmarkRecord<FastqMergeMetrics>> {
    let merged_stats = observe_merge_stats(catalog, platform, runner, merged_reads)?;
    let mut report = load_governed_merge_report(report_path)?;
    report.runtime_s = Some(execution.runtime_s);
    report.memory_mb = Some(execution.memory_mb);
    write_governed_merge_report(report_path, &report)?;

    let pairs_in = report.reads_r1.min(report.reads_r2);
    let reads_merged = report.reads_merged.min(pairs_in);
    let reads_unmerged = report.reads_unmerged.min(pairs_in.saturating_sub(reads_merged));
    let metrics = FastqMergeMetrics {
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
    };
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;
    let out_dir = report_path.parent().ok_or_else(|| anyhow!("merge report has no parent"))?;
    let metrics_path = out_dir.join("metrics.json");
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&metrics_path, &metrics_json)
        .context("write merge metrics")?;
    prune_merge_tool_payload(out_dir, report_path, &metrics_path, &report)?;

    let context = build_benchmark_context(
        &report.tool_id,
        tool_spec.tool_version.clone(),
        benchmark_image_identity(tool_spec),
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
