use std::collections::HashMap;
use std::path::PathBuf;

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::internal::fastq::stages::trim_bench_common::{
    build_benchmark_context, require_existing_benchmark_output,
};
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_duplicates_v1, insert_fastq_duplicates_v1,
};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkRecord, FastqDuplicateMetrics, MetricSet,
};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_planner_fastq::tool_adapters::fastq::remove_duplicates::{
    dedup_mode_from_literal, plan_deduplicate_with_options, RemoveDuplicatesPlanOptions,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use bijux_dna_stage_contract::StagePlanV1;

const STAGE_ID: &str = "fastq.remove_duplicates";
const DEDUP_RATE_EPSILON: f64 = 1e-9;

#[derive(Debug, Clone, PartialEq)]
struct DuplicateReportCounts {
    reads_in: u64,
    reads_out: u64,
    duplicates_removed: u64,
    dedup_rate: f64,
    tool: Option<String>,
    paired_mode: Option<String>,
    dedup_mode: Option<String>,
    keep_order: Option<bool>,
    pair_count_match: Option<bool>,
    duplicate_class_count: Option<u64>,
    duplicate_provenance_json: Option<String>,
    raw_backend_report_format: Option<String>,
}

fn resolve_remove_duplicates_tools(
    requested_tools: &[String],
    tools_resolved_implicitly: bool,
    paired_mode: bool,
) -> Result<Vec<String>> {
    let tools = bijux_dna_planner_fastq::select_remove_duplicates_tools(requested_tools)?;
    let compatible = bijux_dna_planner_fastq::stage_api::filter_tools_for_input_layout(
        &StageId::new(STAGE_ID),
        tools.iter().cloned().map(ToolId::new).collect::<Vec<_>>(),
        paired_mode,
    )
    .into_iter()
    .map(|tool_id| tool_id.to_string())
    .collect::<Vec<_>>();
    if compatible.is_empty() {
        return Err(anyhow!(
            "fastq.remove_duplicates has no governed tools for {} input layout",
            if paired_mode { "paired-end" } else { "single-end" }
        ));
    }
    if !tools_resolved_implicitly && compatible.len() != tools.len() {
        let incompatible =
            tools.into_iter().filter(|tool_id| !compatible.contains(tool_id)).collect::<Vec<_>>();
        return Err(anyhow!(
            "fastq.remove_duplicates does not support {} inputs for tool(s): {}",
            if paired_mode { "paired-end" } else { "single-end" },
            incompatible.join(", "),
        ));
    }
    Ok(compatible)
}

/// Benchmark FASTQ duplicate-removal tools under governed contracts.
///
/// # Errors
/// Returns an error if planning, execution, report parsing, or persistence fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_remove_duplicates<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqRemoveDuplicatesArgs,
) -> Result<BenchOutcome<FastqDuplicateMetrics>> {
    let selected_tools = select_remove_duplicates_benchmark_tools(args)?;
    let setup = prepare_remove_duplicates_setup(platform, runner_override, args, &selected_tools)?;

    if args.explain {
        write_remove_duplicates_explain(&setup)?;
    }

    ensure_remove_duplicates_qa(catalog, platform, &setup.tools)?;

    let sqlite_path = setup.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path)?;
    let bench_path = setup.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::new();

    for tool in &setup.tools {
        let tool_plan = prepare_remove_duplicates_tool_plan(catalog, platform, args, &setup, tool)?;
        if let Ok(Some(record)) = fetch_fastq_duplicates_v1(
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
        let execution = execute_remove_duplicates_tool(&tool_plan, setup.runner, jobs, tool)?;
        if let Some(failure) = remove_duplicates_tool_failure(tool, execution.exit_code) {
            failures.push(failure);
            continue;
        }
        let outputs = resolve_remove_duplicates_outputs(&tool_plan.plan)?;
        let counts = load_deduplicate_report_counts(&outputs.report_path)?;
        let metrics = duplicate_metrics_from_counts(&counts);
        let metric_set = metric_set(metrics);
        write_remove_duplicates_metrics(&tool_plan.out_dir, &metric_set)?;
        let record = BenchmarkRecord {
            context: build_benchmark_context(
                tool,
                tool_plan.tool_spec.tool_version.clone(),
                tool_plan.image_digest.clone(),
                setup.runner,
                platform,
                setup.input_hash.clone(),
                tool_plan.bench_params.clone(),
            ),
            execution: ExecutionMetrics {
                runtime_s: execution.runtime_s,
                memory_mb: execution.memory_mb,
                exit_code: execution.exit_code,
            },
            metrics: metric_set,
        };
        record.validate()?;
        append_jsonl(&bench_path, &record)?;
        insert_fastq_duplicates_v1(&conn, &record)?;
        records.push(record);
    }

    Ok(BenchOutcome { records, failures, bench_dir: setup.bench_dir, explain: args.explain })
}

struct RemoveDuplicatesBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    runner: RuntimeKind,
    input_hash: String,
    bench_dir: std::path::PathBuf,
    tools_root: std::path::PathBuf,
    options: RemoveDuplicatesPlanOptions,
}

struct RemoveDuplicatesToolPlan {
    out_dir: PathBuf,
    tool_spec: ToolExecutionSpecV1,
    plan: StagePlanV1,
    bench_params: serde_json::Value,
    params_hash: String,
    image_digest: String,
}

struct RemoveDuplicatesOutputs {
    report_path: PathBuf,
}

fn select_remove_duplicates_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqRemoveDuplicatesArgs,
) -> Result<Vec<String>> {
    let tools = resolve_remove_duplicates_tools(
        &args.tools,
        args.tools_resolved_implicitly,
        args.r2.is_some(),
    )?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_ID, artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_ID, &header);
    Ok(tools)
}

fn prepare_remove_duplicates_setup(
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqRemoveDuplicatesArgs,
    selected_tools: &[String],
) -> Result<RemoveDuplicatesBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_ID, selected_tools, &registry, false)?;
    let runner = ensure_bench_runner(platform, runner_override)?;
    let input_hash = if let Some(r2) = args.r2.as_deref() {
        format!("{}+{}", hash_file_sha256(&args.r1)?, hash_file_sha256(r2)?)
    } else {
        hash_file_sha256(&args.r1)?
    };
    let bench_dir_name =
        bench_dir_name(&bijux_dna_domain_fastq::stages::ids::STAGE_REMOVE_DUPLICATES)
            .ok_or_else(|| anyhow!("bench dir missing for {STAGE_ID}"))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir)?;
    bijux_dna_infra::ensure_dir(&tools_root)?;
    let options = RemoveDuplicatesPlanOptions {
        threads_override: args.threads,
        dedup_mode: args
            .dedup_mode
            .as_deref()
            .map(dedup_mode_from_literal)
            .transpose()?
            .unwrap_or(bijux_dna_domain_fastq::params::remove_duplicates::DedupMode::Exact),
        keep_order: args.keep_order.unwrap_or(true),
    };

    Ok(RemoveDuplicatesBenchmarkSetup {
        registry,
        tools,
        runner,
        input_hash,
        bench_dir,
        tools_root,
        options,
    })
}

fn write_remove_duplicates_explain(setup: &RemoveDuplicatesBenchmarkSetup) -> Result<()> {
    write_explain_md(&setup.bench_dir, STAGE_ID, &setup.tools, &[], None)?;
    write_explain_plan_json(&setup.bench_dir, STAGE_ID, &setup.tools, &setup.registry, None)
}

fn ensure_remove_duplicates_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(STAGE_ID, tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_ID, tools, platform, catalog)
}

fn prepare_remove_duplicates_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqRemoveDuplicatesArgs,
    setup: &RemoveDuplicatesBenchmarkSetup,
    tool: &str,
) -> Result<RemoveDuplicatesToolPlan> {
    let out_dir = setup.tools_root.join(tool);
    bijux_dna_infra::ensure_dir(&out_dir)?;
    let tool_spec = build_tool_execution_spec(STAGE_ID, tool, &setup.registry, catalog, platform)?;
    let plan = plan_deduplicate_with_options(
        &tool_spec,
        &args.r1,
        args.r2.as_deref(),
        &out_dir,
        &setup.options,
    )?;
    let bench_params = benchmark_query_context()?.embed_in_parameters(&plan.params);
    let params_hash = stable_params_hash(&bench_params);
    let image_digest = tool_spec
        .image
        .digest
        .as_ref()
        .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
        .clone();

    Ok(RemoveDuplicatesToolPlan {
        out_dir,
        tool_spec,
        plan,
        bench_params,
        params_hash,
        image_digest,
    })
}

fn execute_remove_duplicates_tool(
    tool_plan: &RemoveDuplicatesToolPlan,
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

fn remove_duplicates_tool_failure(tool: &str, exit_code: i32) -> Option<RawFailure> {
    (exit_code != 0).then(|| RawFailure {
        stage: STAGE_ID.to_string(),
        tool: tool.to_string(),
        reason: format!("tool {tool} failed with status {exit_code}"),
        category: ErrorCategory::ToolError,
    })
}

fn resolve_remove_duplicates_outputs(plan: &StagePlanV1) -> Result<RemoveDuplicatesOutputs> {
    let report_path = required_plan_output_path(plan, "report_json")?;
    let duplicate_classes_tsv = required_plan_output_path(plan, "duplicate_classes_tsv")?;
    let duplicate_provenance_json = required_plan_output_path(plan, "duplicate_provenance_json")?;
    let report_path = require_existing_benchmark_output(&report_path, "report_json")?.to_path_buf();
    let _duplicate_classes_tsv =
        require_existing_benchmark_output(&duplicate_classes_tsv, "duplicate_classes_tsv")?;
    let _duplicate_provenance_json =
        require_existing_benchmark_output(&duplicate_provenance_json, "duplicate_provenance_json")?;

    Ok(RemoveDuplicatesOutputs { report_path })
}

fn duplicate_metrics_from_counts(counts: &DuplicateReportCounts) -> FastqDuplicateMetrics {
    FastqDuplicateMetrics {
        reads_in: counts.reads_in,
        reads_out: counts.reads_out,
        duplicates_removed: counts.duplicates_removed,
        dedup_rate: counts.dedup_rate,
        tool: counts.tool.clone(),
        paired_mode: counts.paired_mode.clone(),
        dedup_mode: counts.dedup_mode.clone(),
        keep_order: counts.keep_order,
        pair_count_match: counts.pair_count_match,
        duplicate_class_count: counts.duplicate_class_count,
        duplicate_provenance_json: counts.duplicate_provenance_json.clone(),
        raw_backend_report_format: counts.raw_backend_report_format.clone(),
    }
}

fn write_remove_duplicates_metrics(
    out_dir: &std::path::Path,
    metric_set: &MetricSet<FastqDuplicateMetrics>,
) -> Result<()> {
    Ok(bijux_dna_infra::atomic_write_json(
        &out_dir.join("metrics.json"),
        &serde_json::to_value(metric_set)?,
    )?)
}

fn benchmark_query_context() -> Result<bijux_dna_domain_fastq::BenchQueryContext> {
    bijux_dna_domain_fastq::governed_stage_bench_query_context(STAGE_ID)
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

#[cfg(test)]
fn deduplicate_report_counts(
    input_reads_r1: u64,
    input_reads_r2: Option<u64>,
    output_reads_r1: u64,
    output_reads_r2: Option<u64>,
) -> DuplicateReportCounts {
    let reads_in = input_reads_r1 + input_reads_r2.unwrap_or(0);
    let reads_out = output_reads_r1 + output_reads_r2.unwrap_or(0);
    let duplicates_removed = reads_in.saturating_sub(reads_out);
    let dedup_rate =
        if reads_in == 0 { 0.0 } else { u64_to_f64(duplicates_removed) / u64_to_f64(reads_in) };
    DuplicateReportCounts {
        reads_in,
        reads_out,
        duplicates_removed,
        dedup_rate,
        tool: None,
        paired_mode: None,
        dedup_mode: None,
        keep_order: None,
        pair_count_match: None,
        duplicate_class_count: None,
        duplicate_provenance_json: None,
        raw_backend_report_format: None,
    }
}

#[allow(clippy::too_many_lines)]
fn load_deduplicate_report_counts(report_path: &std::path::Path) -> Result<DuplicateReportCounts> {
    let raw = std::fs::read_to_string(report_path).map_err(|error| {
        anyhow!("read governed remove-duplicates report {}: {error}", report_path.display())
    })?;
    let report = bijux_dna_domain_fastq::observer::parse_remove_duplicates_report(&raw).map_err(
        |error| {
            anyhow!("parse governed remove-duplicates report {}: {error}", report_path.display())
        },
    )?;
    let reads_in = report.reads_in;
    let reads_out = report.reads_out;
    let duplicates_removed = report.duplicates_removed;
    let derived_duplicates_removed = reads_in.saturating_sub(reads_out);
    if duplicates_removed != derived_duplicates_removed {
        return Err(anyhow!(
            "governed remove-duplicates report {} is inconsistent: duplicates_removed={} but reads_in-reads_out={derived_duplicate_reads}",
            report_path.display(),
            duplicates_removed,
            derived_duplicate_reads = derived_duplicates_removed,
        ));
    }
    let dedup_rate =
        if reads_in == 0 { 0.0 } else { u64_to_f64(duplicates_removed) / u64_to_f64(reads_in) };
    if !report.dedup_rate.is_finite() || !(0.0..=1.0).contains(&report.dedup_rate) {
        return Err(anyhow!(
            "governed remove-duplicates report {} is inconsistent: dedup_rate={} must be within [0, 1]",
            report_path.display(),
            report.dedup_rate,
        ));
    }
    if (report.dedup_rate - dedup_rate).abs() > DEDUP_RATE_EPSILON {
        return Err(anyhow!(
            "governed remove-duplicates report {} is inconsistent: dedup_rate={} but reads_removed/reads_in={dedup_rate}",
            report_path.display(),
            report.dedup_rate,
        ));
    }
    if let Some(reads_in_r2) = report.reads_in_r2 {
        if reads_in_r2 != reads_in {
            return Err(anyhow!(
                "governed remove-duplicates report {} is inconsistent: reads_in_r2={} but reads_in={reads_in}",
                report_path.display(),
                reads_in_r2,
            ));
        }
    }
    if let Some(reads_out_r2) = report.reads_out_r2 {
        if reads_out_r2 != reads_out {
            return Err(anyhow!(
                "governed remove-duplicates report {} is inconsistent: reads_out_r2={} but reads_out={reads_out}",
                report_path.display(),
                reads_out_r2,
            ));
        }
    }
    if let Some(pairs_in) = report.pairs_in {
        if pairs_in != reads_in {
            return Err(anyhow!(
                "governed remove-duplicates report {} is inconsistent: pairs_in={} but reads_in={reads_in}",
                report_path.display(),
                pairs_in,
            ));
        }
    }
    if let Some(pairs_out) = report.pairs_out {
        if pairs_out != reads_out {
            return Err(anyhow!(
                "governed remove-duplicates report {} is inconsistent: pairs_out={} but reads_out={reads_out}",
                report_path.display(),
                pairs_out,
            ));
        }
    }
    if (report.pairs_in.is_some() || report.pairs_out.is_some())
        && report.pair_count_match != Some(true)
    {
        return Err(anyhow!(
                "governed remove-duplicates report {} is inconsistent: paired reports must set pair_count_match=true",
                report_path.display(),
            ));
    }
    let classified_duplicates_removed: u64 =
        report.duplicate_classes.iter().map(|entry| entry.reads_removed).sum();
    if !report.duplicate_classes.is_empty() && classified_duplicates_removed != duplicates_removed {
        return Err(anyhow!(
            "governed remove-duplicates report {} is inconsistent: duplicate_classes sum to {} but duplicates_removed={duplicate_reads}",
            report_path.display(),
            classified_duplicates_removed,
            duplicate_reads = duplicates_removed,
        ));
    }
    if !report.duplicate_classes.is_empty()
        && (report.duplicate_classes_tsv.is_none() || report.duplicate_provenance_json.is_none())
    {
        return Err(anyhow!(
            "governed remove-duplicates report {} is inconsistent: duplicate classes require duplicate_classes_tsv and duplicate_provenance_json",
            report_path.display(),
        ));
    }
    Ok(DuplicateReportCounts {
        reads_in,
        reads_out,
        duplicates_removed,
        dedup_rate: report.dedup_rate,
        tool: Some(report.tool_id),
        paired_mode: serde_json::to_value(report.paired_mode)?.as_str().map(ToString::to_string),
        dedup_mode: Some(serde_json::to_string(&report.dedup_mode)?.trim_matches('"').to_string()),
        keep_order: Some(report.keep_order),
        pair_count_match: report.pair_count_match,
        duplicate_class_count: Some(report.duplicate_classes.len() as u64),
        duplicate_provenance_json: report.duplicate_provenance_json,
        raw_backend_report_format: report.raw_backend_report_format,
    })
}

fn required_plan_output_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    output_id: &str,
) -> Result<std::path::PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| {
            anyhow!(
                "remove_duplicates plan is missing governed output `{output_id}` for tool {}",
                plan.tool_id.as_str()
            )
        })
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::path::PathBuf;

    use super::{
        deduplicate_report_counts, load_deduplicate_report_counts, required_plan_output_path,
        resolve_remove_duplicates_tools, DuplicateReportCounts,
    };
    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolConstraints, ToolId,
    };
    use bijux_dna_stage_contract::{ArtifactRef, PlanDecisionReason, StageIO, StagePlanV1};

    fn plan_with_outputs(paired: bool) -> StagePlanV1 {
        let mut outputs = vec![ArtifactRef::required(
            ArtifactId::from_static("dedup_reads_r1"),
            PathBuf::from("out/dedup_r1.fastq.gz"),
            ArtifactRole::Reads,
        )];
        if paired {
            outputs.push(ArtifactRef::required(
                ArtifactId::from_static("dedup_reads_r2"),
                PathBuf::from("out/dedup_r2.fastq.gz"),
                ArtifactRole::Reads,
            ));
        }
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("report_json"),
            PathBuf::from("out/deduplicate_report.json"),
            ArtifactRole::ReportJson,
        ));
        StagePlanV1 {
            stage_id: StageId::from_static("fastq.remove_duplicates"),
            stage_instance_id: None,
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("clumpify"),
            tool_version: "test".to_string(),
            image: serde_json::from_value(serde_json::json!({
                "image": "bijuxdna/clumpify",
                "digest": null,
            }))
            .expect("image"),
            command: CommandSpecV1 { template: vec!["echo".to_string(), "ok".to_string()] },
            resources: ToolConstraints::default(),
            io: StageIO { inputs: vec![], outputs },
            out_dir: PathBuf::from("out"),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            aux_images: std::collections::BTreeMap::new(),
            reason: PlanDecisionReason::default(),
        }
    }

    #[test]
    fn deduplicate_counts_cover_paired_inputs() {
        let counts = deduplicate_report_counts(100, Some(100), 70, Some(70));
        assert_eq!(
            counts,
            DuplicateReportCounts {
                reads_in: 200,
                reads_out: 140,
                duplicates_removed: 60,
                dedup_rate: 0.3,
                tool: None,
                paired_mode: None,
                dedup_mode: None,
                keep_order: None,
                pair_count_match: None,
                duplicate_class_count: None,
                duplicate_provenance_json: None,
                raw_backend_report_format: None,
            }
        );
    }

    #[test]
    fn deduplicate_metrics_load_from_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("deduplicate_report.json");
        bijux_dna_infra::write_bytes(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.remove_duplicates.report.v2",
                "stage": "fastq.remove_duplicates",
                "stage_id": "fastq.remove_duplicates",
                "tool_id": "clumpify",
                "paired_mode": "single_end",
                "threads": 4,
                "dedup_mode": "exact",
                "keep_order": true,
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "dedup.fastq.gz",
                "output_r2": null,
                "reads_in": 200,
                "reads_out": 160,
                "reads_in_r2": null,
                "reads_out_r2": null,
                "pairs_in": null,
                "pairs_out": null,
                "pair_count_match": null,
                "duplicates_removed": 40,
                "dedup_rate": 0.2,
                "duplicate_classes_tsv": "duplicate_classes.tsv",
                "duplicate_provenance_json": "duplicate_provenance.json",
                "duplicate_classes": [
                    {"class": "duplicate", "reads_removed": 40, "paired_mode": "single_end"}
                ],
                "raw_backend_report": "clumpify.log",
                "raw_backend_report_format": "clumpify_log",
                "runtime_s": null,
                "memory_mb": null
            })
            .to_string(),
        )
        .expect("write report");

        let counts =
            load_deduplicate_report_counts(&report_path).expect("load governed dedup report");
        assert_eq!(counts.reads_in, 200);
        assert_eq!(counts.reads_out, 160);
        assert_eq!(counts.duplicates_removed, 40);
        assert!((counts.dedup_rate - 0.2).abs() < f64::EPSILON);
        assert_eq!(counts.tool.as_deref(), Some("clumpify"));
        assert_eq!(counts.duplicate_class_count, Some(1));
    }

    #[test]
    fn deduplicate_metrics_accept_legacy_key_value_reports() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("deduplicate_report.txt");
        bijux_dna_infra::write_bytes(&report_path, "reads_in=200\nreads_out=160\n")
            .expect("write report");

        let counts =
            load_deduplicate_report_counts(&report_path).expect("load parser-backed dedup report");
        assert_eq!(counts.reads_in, 200);
        assert_eq!(counts.reads_out, 160);
        assert_eq!(counts.duplicates_removed, 40);
        assert!((counts.dedup_rate - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn deduplicate_metrics_reject_inconsistent_duplicate_counts() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("deduplicate_report.json");
        bijux_dna_infra::write_bytes(
            &report_path,
            serde_json::json!({
                "reads_in": 200,
                "reads_out": 160,
                "duplicates_removed": 35
            })
            .to_string(),
        )
        .expect("write report");

        let error = load_deduplicate_report_counts(&report_path)
            .expect_err("inconsistent duplicate counts must fail");
        assert!(error.to_string().contains("inconsistent"));
    }

    #[test]
    fn deduplicate_metrics_reject_inconsistent_dedup_rate() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("deduplicate_report.json");
        bijux_dna_infra::write_bytes(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.remove_duplicates.report.v2",
                "stage": "fastq.remove_duplicates",
                "stage_id": "fastq.remove_duplicates",
                "tool_id": "clumpify",
                "paired_mode": "single_end",
                "threads": 4,
                "dedup_mode": "exact",
                "keep_order": true,
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "dedup.fastq.gz",
                "output_r2": null,
                "reads_in": 100,
                "reads_out": 84,
                "reads_in_r2": null,
                "reads_out_r2": null,
                "pairs_in": null,
                "pairs_out": null,
                "pair_count_match": null,
                "duplicates_removed": 16,
                "dedup_rate": 0.10,
                "duplicate_classes_tsv": "duplicate_classes.tsv",
                "duplicate_provenance_json": "duplicate_provenance.json",
                "duplicate_classes": [
                    {"class": "duplicate", "reads_removed": 16, "paired_mode": "single_end"}
                ],
                "raw_backend_report": "clumpify.log",
                "raw_backend_report_format": "clumpify_log",
                "runtime_s": null,
                "memory_mb": null
            })
            .to_string(),
        )
        .expect("write report");

        let error = load_deduplicate_report_counts(&report_path)
            .expect_err("inconsistent dedup rate must fail");
        assert!(error.to_string().contains("dedup_rate"));
    }

    #[test]
    fn deduplicate_metrics_reject_inconsistent_pair_accounting() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("deduplicate_report.json");
        bijux_dna_infra::write_bytes(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.remove_duplicates.report.v2",
                "stage": "fastq.remove_duplicates",
                "stage_id": "fastq.remove_duplicates",
                "tool_id": "clumpify",
                "paired_mode": "paired_end",
                "threads": 6,
                "dedup_mode": "optical_aware",
                "keep_order": false,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "output_r1": "dedup_R1.fastq.gz",
                "output_r2": "dedup_R2.fastq.gz",
                "reads_in": 200,
                "reads_out": 172,
                "reads_in_r2": 200,
                "reads_out_r2": 170,
                "pairs_in": 200,
                "pairs_out": 172,
                "pair_count_match": false,
                "duplicates_removed": 28,
                "dedup_rate": 0.14,
                "duplicate_classes_tsv": "duplicate_classes.tsv",
                "duplicate_provenance_json": "duplicate_provenance.json",
                "duplicate_classes": [
                    {"class": "duplicate", "reads_removed": 20, "paired_mode": "paired_end"},
                    {"class": "optical_duplicate", "reads_removed": 8, "paired_mode": "paired_end"}
                ],
                "raw_backend_report": "clumpify.log",
                "raw_backend_report_format": "clumpify_log",
                "runtime_s": null,
                "memory_mb": null
            })
            .to_string(),
        )
        .expect("write report");

        let error = load_deduplicate_report_counts(&report_path)
            .expect_err("inconsistent pair accounting must fail");
        assert!(error.to_string().contains("reads_out_r2"));
    }

    #[test]
    fn deduplicate_metrics_reject_inconsistent_duplicate_class_breakdown() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("deduplicate_report.json");
        bijux_dna_infra::write_bytes(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.remove_duplicates.report.v2",
                "stage": "fastq.remove_duplicates",
                "stage_id": "fastq.remove_duplicates",
                "tool_id": "clumpify",
                "paired_mode": "single_end",
                "threads": 4,
                "dedup_mode": "exact",
                "keep_order": true,
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "dedup.fastq.gz",
                "output_r2": null,
                "reads_in": 100,
                "reads_out": 84,
                "reads_in_r2": null,
                "reads_out_r2": null,
                "pairs_in": null,
                "pairs_out": null,
                "pair_count_match": null,
                "duplicates_removed": 16,
                "dedup_rate": 0.16,
                "duplicate_classes_tsv": "duplicate_classes.tsv",
                "duplicate_provenance_json": null,
                "duplicate_classes": [
                    {"class": "duplicate", "reads_removed": 12, "paired_mode": "single_end"}
                ],
                "raw_backend_report": "clumpify.log",
                "raw_backend_report_format": "clumpify_log",
                "runtime_s": null,
                "memory_mb": null
            })
            .to_string(),
        )
        .expect("write report");

        let error = load_deduplicate_report_counts(&report_path)
            .expect_err("incomplete duplicate class evidence must fail");
        assert!(error.to_string().contains("duplicate_classes"));
    }

    #[test]
    fn required_plan_output_path_uses_governed_report_artifact() {
        let plan = plan_with_outputs(true);
        assert_eq!(
            required_plan_output_path(&plan, "report_json").expect("report path"),
            PathBuf::from("out/deduplicate_report.json")
        );
    }

    #[test]
    fn implicit_single_end_dedup_selection_filters_paired_only_tools() {
        let tools = resolve_remove_duplicates_tools(
            &["fastuniq".to_string(), "clumpify".to_string()],
            true,
            false,
        )
        .expect("single-end auto selection should keep only compatible tools");
        assert_eq!(tools, vec!["clumpify".to_string()]);
    }

    #[test]
    fn explicit_single_end_fastuniq_request_still_fails() {
        let error = resolve_remove_duplicates_tools(&["fastuniq".to_string()], false, false)
            .expect_err("explicit incompatible tool request must fail");
        assert!(error.to_string().contains("single-end"));
    }
}
