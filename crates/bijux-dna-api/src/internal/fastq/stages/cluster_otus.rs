use std::collections::{BTreeSet, HashMap};

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_cluster_otus_v1, insert_fastq_cluster_otus_v1,
};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqClusterOtusMetrics};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_domain_fastq::params::edna::OtuClusteringEffectiveParams;
use bijux_dna_domain_fastq::{ClusterOtusReportV1, CLUSTER_OTUS_REPORT_SCHEMA_VERSION};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::stage_api::{
    bench_dir_name, inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind,
    RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;

use crate::internal::fastq::stages::preprocess::materialize_amplicon_stage_outputs_for_bench;
use crate::internal::fastq::stages::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context,
};
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};

const STAGE_ID: &str = "fastq.cluster_otus";
const LOCAL_CLUSTER_OTUS_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.cluster_otus.local_smoke.report.v1";

#[derive(Debug, Clone, serde::Serialize)]
struct LocalClusterOtusSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    planned_tool_id: String,
    report_tool_id: String,
    clustering_threshold: f64,
    otu_count: u64,
    sample_count: u64,
    representative_sequence_count: u64,
    otu_table_tsv: String,
    representative_sequences_fasta: String,
    otu_representatives_fasta: String,
    case_report_json: String,
    taxonomy_ready_fasta: String,
    taxonomy_ready_fastq: String,
    raw_backend_report: String,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ClusterOtusTableMetrics {
    pub otu_count: u64,
    pub sample_count: u64,
}

pub(crate) fn cluster_otus_options_from_args(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqClusterOtusArgs,
) -> bijux_dna_planner_fastq::ClusterOtusStageParams {
    let mut options = bijux_dna_planner_fastq::ClusterOtusStageParams::baseline();
    if let Some(otu_identity) = args.otu_identity {
        options.otu_identity = otu_identity;
    }
    options.threads = args.threads;
    options
}

pub(crate) fn read_cluster_otus_table_metrics(
    path: &std::path::Path,
) -> Result<ClusterOtusTableMetrics> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut samples = std::collections::BTreeSet::new();
    let mut otus = std::collections::BTreeSet::new();
    for line in raw.lines().skip(1) {
        let mut parts = line.split('\t');
        let Some(sample_id) = parts.next().map(str::trim) else {
            continue;
        };
        let Some(otu_id) = parts.next().map(str::trim) else {
            continue;
        };
        if !sample_id.is_empty() {
            samples.insert(sample_id.to_string());
        }
        if !otu_id.is_empty() {
            otus.insert(otu_id.to_string());
        }
    }
    Ok(ClusterOtusTableMetrics { otu_count: otus.len() as u64, sample_count: samples.len() as u64 })
}

pub(crate) fn count_cluster_otus_representatives(path: &std::path::Path) -> Result<u64> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(raw.lines().filter(|line| line.trim_start().starts_with('>')).count() as u64)
}

pub(crate) struct ClusterOtusReportInputs<'a> {
    pub tool_id: &'a str,
    pub input_reads: &'a std::path::Path,
    pub otu_table: &'a std::path::Path,
    pub otu_representatives: &'a std::path::Path,
    pub taxonomy_reference_fasta: &'a std::path::Path,
    pub taxonomy_reads_fastq: &'a std::path::Path,
    pub report_json: &'a std::path::Path,
    pub effective_params: &'a OtuClusteringEffectiveParams,
    pub table_metrics: ClusterOtusTableMetrics,
    pub representative_sequence_count: u64,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i32>,
    pub used_fallback: bool,
    pub raw_backend_report: Option<&'a std::path::Path>,
    pub backend_metrics: Option<serde_json::Value>,
}

struct ClusterOtusOutputs {
    otu_table: std::path::PathBuf,
    otu_representatives: std::path::PathBuf,
    taxonomy_reference_fasta: std::path::PathBuf,
    taxonomy_reads_fastq: std::path::PathBuf,
    report_json: std::path::PathBuf,
}

pub(crate) fn canonical_cluster_otus_report(
    inputs: ClusterOtusReportInputs<'_>,
) -> ClusterOtusReportV1 {
    ClusterOtusReportV1 {
        schema_version: CLUSTER_OTUS_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_ID.to_string(),
        stage_id: STAGE_ID.to_string(),
        tool_id: inputs.tool_id.to_string(),
        otu_identity: inputs.effective_params.identity_threshold,
        threads: inputs.effective_params.threads,
        input_reads: inputs.input_reads.display().to_string(),
        otu_table: inputs.otu_table.display().to_string(),
        otu_representatives: inputs.otu_representatives.display().to_string(),
        taxonomy_ready_fasta: inputs.taxonomy_reference_fasta.display().to_string(),
        taxonomy_ready_fastq: inputs.taxonomy_reads_fastq.display().to_string(),
        report_json: inputs.report_json.display().to_string(),
        otu_count: inputs.table_metrics.otu_count,
        sample_count: inputs.table_metrics.sample_count,
        representative_sequence_count: inputs.representative_sequence_count,
        output_table_kind: inputs.effective_params.output_table_kind.clone(),
        used_fallback: inputs.used_fallback,
        runtime_s: inputs.runtime_s,
        memory_mb: inputs.memory_mb,
        exit_code: inputs.exit_code,
        raw_backend_report: inputs.raw_backend_report.map(|path| path.display().to_string()),
        raw_backend_report_format: inputs.effective_params.raw_backend_report_format.clone(),
        backend_metrics: inputs.backend_metrics,
    }
}

/// Benchmark FASTQ OTU clustering tools against governed stage contracts.
///
/// # Errors
/// Returns an error if input discovery, planning, execution, report materialization,
/// or `SQLite` persistence fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_cluster_otus<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqClusterOtusArgs,
) -> Result<BenchOutcome<FastqClusterOtusMetrics>> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = bijux_dna_planner_fastq::select_cluster_otus_tools(&args.tools)?;
    let tools = filter_tools_by_role(STAGE_ID, &tools, &registry, false)?;
    let runner = ensure_bench_runner(platform, runner_override)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_ID, artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_ID, &header);
    let input_hash = if let Some(r2) = args.r2.as_deref() {
        format!(
            "{}+{}",
            hash_file_sha256(&args.r1).context("hash cluster otus input r1")?,
            hash_file_sha256(r2).context("hash cluster otus input r2")?
        )
    } else {
        hash_file_sha256(&args.r1).context("hash cluster otus input")?
    };
    let bench_dir_name = bench_dir_name(&bijux_dna_domain_fastq::stages::ids::STAGE_CLUSTER_OTUS)
        .ok_or_else(|| anyhow!("bench dir missing for {STAGE_ID}"))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir)?;
    bijux_dna_infra::ensure_dir(&tools_root)?;

    if args.explain {
        write_explain_md(&bench_dir, STAGE_ID, &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, STAGE_ID, &tools, &registry, None)?;
    }

    ensure_image_qa_passed(STAGE_ID, &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_ID, &tools, platform, catalog)?;

    let sqlite_path = bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path)?;
    let bench_path = bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::new();

    for tool in &tools {
        let out_dir = tools_root.join(tool);
        bijux_dna_infra::ensure_dir(&out_dir)?;
        let tool_spec = build_tool_execution_spec(STAGE_ID, tool, &registry, catalog, platform)?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = bijux_dna_planner_fastq::tool_adapters::fastq::cluster_otus::plan_with_options(
            &tool_spec,
            &args.r1,
            args.r2.as_deref(),
            &out_dir,
            &cluster_otus_options_from_args(args),
        )?;
        let params_hash = stable_params_hash(&plan.params);
        let image_digest = benchmark_image_identity(&tool_spec);
        if let Ok(Some(record)) = fetch_fastq_cluster_otus_v1(
            &conn,
            tool,
            &tool_spec.tool_version,
            &image_digest,
            &runner.to_string(),
            &platform.name,
            &input_hash,
            &params_hash,
        ) {
            records.push(record);
            continue;
        }
        let step = bijux_dna_stage_contract::execution_step_from_stage_plan(&plan);
        let execution = execute_plans_with_jobs(vec![step.clone()], runner, jobs)?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("missing execution result for {tool}"))?;
        if execution.exit_code != 0 {
            let stderr = execution.stderr.trim();
            let reason = if stderr.is_empty() {
                format!("tool {tool} failed with status {}", execution.exit_code)
            } else {
                format!("tool {tool} failed with status {}: {stderr}", execution.exit_code)
            };
            failures.push(RawFailure {
                stage: STAGE_ID.to_string(),
                tool: tool.clone(),
                reason,
                category: ErrorCategory::ToolError,
            });
            continue;
        }
        let payload = materialize_amplicon_stage_outputs_for_bench(&out_dir, &step)?;
        let outputs = resolve_cluster_otus_outputs(&plan)?;
        let table_metrics = read_cluster_otus_table_metrics(&outputs.otu_table)?;
        let representative_count =
            count_cluster_otus_representatives(&outputs.otu_representatives)?;
        let metrics =
            FastqClusterOtusMetrics { otu_count: table_metrics.otu_count, representative_count };
        let metric_set = metric_set(metrics);
        let effective_params: OtuClusteringEffectiveParams =
            serde_json::from_value(plan.effective_params.clone())
                .context("parse cluster_otus effective params")?;
        let used_fallback = cluster_otus_used_fallback(&payload)?;
        let raw_backend_report = out_dir.join("otu_clusters.uc");
        let report = canonical_cluster_otus_report(ClusterOtusReportInputs {
            tool_id: tool,
            input_reads: &args.r1,
            otu_table: &outputs.otu_table,
            otu_representatives: &outputs.otu_representatives,
            taxonomy_reference_fasta: &outputs.taxonomy_reference_fasta,
            taxonomy_reads_fastq: &outputs.taxonomy_reads_fastq,
            report_json: &outputs.report_json,
            effective_params: &effective_params,
            table_metrics,
            representative_sequence_count: representative_count,
            runtime_s: Some(execution.runtime_s),
            memory_mb: Some(execution.memory_mb),
            exit_code: Some(execution.exit_code),
            used_fallback,
            raw_backend_report: raw_backend_report.exists().then_some(raw_backend_report.as_path()),
            backend_metrics: Some(serde_json::json!({
                "tool_payload": payload,
            })),
        });
        validate_cluster_otus_report_identity(tool, &report)?;
        validate_cluster_otus_report_metrics(&report, &metric_set.metrics)?;
        validate_cluster_otus_report_execution(
            &report,
            execution.runtime_s,
            execution.memory_mb,
            execution.exit_code,
        )?;
        write_cluster_otus_artifacts(&out_dir, &outputs, &report, &metric_set)?;
        let record = BenchmarkRecord {
            context: build_benchmark_context(
                tool,
                tool_spec.tool_version.clone(),
                image_digest,
                runner,
                platform,
                input_hash.clone(),
                plan.params.clone(),
            ),
            execution: ExecutionMetrics {
                runtime_s: execution.runtime_s,
                memory_mb: execution.memory_mb,
                exit_code: execution.exit_code,
            },
            metrics: metric_set,
        };
        record.validate()?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_cluster_otus_v1(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome { records, failures, bench_dir, explain: args.explain })
}

fn resolve_cluster_otus_outputs(
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<ClusterOtusOutputs> {
    let outputs = ClusterOtusOutputs {
        otu_table: output_path(plan, "otu_table")?,
        otu_representatives: output_path(plan, "otu_representatives")?,
        taxonomy_reference_fasta: output_path(plan, "taxonomy_ready_fasta")?,
        taxonomy_reads_fastq: output_path(plan, "taxonomy_ready_fastq")?,
        report_json: output_path(plan, "report_json")?,
    };
    validate_cluster_otus_output_paths(&outputs)?;
    Ok(outputs)
}

fn validate_cluster_otus_output_paths(outputs: &ClusterOtusOutputs) -> Result<()> {
    let mut paths = BTreeSet::new();
    for path in [
        outputs.otu_table.as_path(),
        outputs.otu_representatives.as_path(),
        outputs.taxonomy_reference_fasta.as_path(),
        outputs.taxonomy_reads_fastq.as_path(),
        outputs.report_json.as_path(),
    ] {
        if !paths.insert(path) {
            return Err(anyhow!("cluster_otus output path reused: {}", path.display()));
        }
    }
    Ok(())
}

fn validate_cluster_otus_report_identity(tool: &str, report: &ClusterOtusReportV1) -> Result<()> {
    if report.schema_version != CLUSTER_OTUS_REPORT_SCHEMA_VERSION {
        return Err(anyhow!(
            "cluster_otus report schema mismatch: expected {}, observed {}",
            CLUSTER_OTUS_REPORT_SCHEMA_VERSION,
            report.schema_version
        ));
    }
    if report.stage != STAGE_ID || report.stage_id != STAGE_ID {
        return Err(anyhow!(
            "cluster_otus report stage mismatch: observed stage={} stage_id={}",
            report.stage,
            report.stage_id
        ));
    }
    if report.tool_id != tool {
        return Err(anyhow!(
            "cluster_otus report tool mismatch: expected {}, observed {}",
            tool,
            report.tool_id
        ));
    }
    Ok(())
}

fn validate_cluster_otus_report_metrics(
    report: &ClusterOtusReportV1,
    metrics: &FastqClusterOtusMetrics,
) -> Result<()> {
    if report.otu_count != metrics.otu_count {
        return Err(anyhow!(
            "cluster_otus report otu_count mismatch: expected {}, observed {}",
            metrics.otu_count,
            report.otu_count
        ));
    }
    if report.representative_sequence_count != metrics.representative_count {
        return Err(anyhow!(
            "cluster_otus report representative count mismatch: expected {}, observed {}",
            metrics.representative_count,
            report.representative_sequence_count
        ));
    }
    Ok(())
}

fn validate_cluster_otus_report_execution(
    report: &ClusterOtusReportV1,
    runtime_s: f64,
    memory_mb: f64,
    exit_code: i32,
) -> Result<()> {
    if report.runtime_s.is_none_or(|observed| (observed - runtime_s).abs() > f64::EPSILON) {
        return Err(anyhow!(
            "cluster_otus report runtime mismatch: expected {}, observed {:?}",
            runtime_s,
            report.runtime_s
        ));
    }
    if report.memory_mb.is_none_or(|observed| (observed - memory_mb).abs() > f64::EPSILON) {
        return Err(anyhow!(
            "cluster_otus report memory mismatch: expected {}, observed {:?}",
            memory_mb,
            report.memory_mb
        ));
    }
    if report.exit_code != Some(exit_code) {
        return Err(anyhow!(
            "cluster_otus report exit code mismatch: expected {}, observed {:?}",
            exit_code,
            report.exit_code
        ));
    }
    Ok(())
}

fn write_cluster_otus_artifacts(
    out_dir: &std::path::Path,
    outputs: &ClusterOtusOutputs,
    report: &ClusterOtusReportV1,
    metric_set: &bijux_dna_analyze::MetricSet<FastqClusterOtusMetrics>,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(&outputs.report_json, report)?;
    bijux_dna_infra::atomic_write_json(
        &out_dir.join("metrics.json"),
        &serde_json::to_value(metric_set)?,
    )?;
    validate_cluster_otus_written_artifacts(out_dir, outputs, report)
}

fn validate_cluster_otus_written_artifacts(
    out_dir: &std::path::Path,
    outputs: &ClusterOtusOutputs,
    report: &ClusterOtusReportV1,
) -> Result<()> {
    let metrics_json = out_dir.join("metrics.json");
    for path in [
        outputs.otu_table.as_path(),
        outputs.otu_representatives.as_path(),
        outputs.taxonomy_reference_fasta.as_path(),
        outputs.taxonomy_reads_fastq.as_path(),
        outputs.report_json.as_path(),
        metrics_json.as_path(),
    ] {
        validate_cluster_otus_artifact_exists(path)?;
    }
    validate_cluster_otus_nonempty_artifact(&outputs.otu_table)?;
    validate_cluster_otus_nonempty_artifact(&outputs.report_json)?;
    validate_cluster_otus_nonempty_artifact(&metrics_json)?;
    if report.representative_sequence_count > 0 {
        validate_cluster_otus_nonempty_artifact(&outputs.otu_representatives)?;
        validate_cluster_otus_nonempty_artifact(&outputs.taxonomy_reference_fasta)?;
        validate_cluster_otus_nonempty_artifact(&outputs.taxonomy_reads_fastq)?;
    }
    Ok(())
}

fn validate_cluster_otus_artifact_exists(path: &std::path::Path) -> Result<()> {
    std::fs::metadata(path)
        .with_context(|| format!("read cluster_otus artifact {}", path.display()))?;
    Ok(())
}

fn validate_cluster_otus_nonempty_artifact(path: &std::path::Path) -> Result<()> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("read cluster_otus artifact {}", path.display()))?;
    if metadata.len() == 0 {
        return Err(anyhow!("cluster_otus artifact is empty: {}", path.display()));
    }
    Ok(())
}

fn cluster_otus_used_fallback(payload: &serde_json::Value) -> Result<bool> {
    payload
        .get("used_fallback")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("cluster_otus payload missing boolean used_fallback"))
}

fn output_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    output_id: &str,
) -> Result<std::path::PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| anyhow!("cluster_otus plan missing {output_id} output"))
}

/// Materialize the governed local-smoke `fastq.cluster_otus` artifact bundle.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_cluster_otus_smoke_report() -> Result<std::path::PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_fastq::stage_api::local_cluster_otus_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "governed fastq.cluster_otus local smoke must resolve exactly one case"
        ));
    };

    let output_root = repo_root.join("target/local-smoke/fastq.cluster_otus");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let summary = materialize_local_cluster_otus_smoke_case(&repo_root, case, &output_root)?;
    let report_path = output_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    Ok(output_root.join("otu_table.tsv"))
}

fn materialize_local_cluster_otus_smoke_case(
    repo_root: &std::path::Path,
    case: &bijux_dna_planner_fastq::LocalClusterOtusSmokeCasePlan,
    output_root: &std::path::Path,
) -> Result<LocalClusterOtusSmokeReport> {
    let effective_params =
        serde_json::from_value::<OtuClusteringEffectiveParams>(case.plan.effective_params.clone())
            .map_err(|error| {
                anyhow!("decode cluster-otus local-smoke effective params: {error}")
            })?;

    let input_reads = repo_root.join(&case.reads);
    let outputs = resolve_cluster_otus_outputs(&case.plan)?;
    let case_otu_table = resolve_smoke_output_path(repo_root, &outputs.otu_table);
    let case_otu_representatives =
        resolve_smoke_output_path(repo_root, &outputs.otu_representatives);
    let case_taxonomy_fasta =
        resolve_smoke_output_path(repo_root, &outputs.taxonomy_reference_fasta);
    let case_taxonomy_fastq = resolve_smoke_output_path(repo_root, &outputs.taxonomy_reads_fastq);
    let case_report_json = resolve_smoke_output_path(repo_root, &outputs.report_json);
    let case_raw_backend_report =
        resolve_smoke_output_path(repo_root, &case.plan.out_dir.join("otu_clusters.uc"));

    for path in [
        &case_otu_table,
        &case_otu_representatives,
        &case_taxonomy_fasta,
        &case_taxonomy_fastq,
        &case_report_json,
        &case_raw_backend_report,
    ] {
        if let Some(parent) = path.parent() {
            bijux_dna_infra::ensure_dir(parent)?;
        }
    }

    bijux_dna_domain_fastq::stages::contract::cluster_otus(
        &input_reads,
        &effective_params,
        &case_otu_table,
        &case_otu_representatives,
        &case_taxonomy_fasta,
        &case_taxonomy_fastq,
        &case_report_json,
    )?;
    let table_metrics = read_cluster_otus_table_metrics(&case_otu_table)?;
    let representative_sequence_count =
        count_cluster_otus_representatives(&case_otu_representatives)?;
    write_smoke_cluster_otus_uc_report(&case_otu_representatives, &case_raw_backend_report)?;

    let mut report = canonical_cluster_otus_report(ClusterOtusReportInputs {
        tool_id: "bijux",
        input_reads: &input_reads,
        otu_table: &case_otu_table,
        otu_representatives: &case_otu_representatives,
        taxonomy_reference_fasta: &case_taxonomy_fasta,
        taxonomy_reads_fastq: &case_taxonomy_fastq,
        report_json: &case_report_json,
        effective_params: &effective_params,
        table_metrics,
        representative_sequence_count,
        runtime_s: None,
        memory_mb: None,
        exit_code: Some(0),
        used_fallback: false,
        raw_backend_report: Some(case_raw_backend_report.as_path()),
        backend_metrics: Some(serde_json::json!({
            "cluster_memberships": table_metrics.otu_count,
        })),
    });
    report.input_reads = case.reads.display().to_string();
    report.otu_table = path_relative_to_repo(repo_root, &case_otu_table);
    report.otu_representatives = path_relative_to_repo(repo_root, &case_otu_representatives);
    report.taxonomy_ready_fasta = path_relative_to_repo(repo_root, &case_taxonomy_fasta);
    report.taxonomy_ready_fastq = path_relative_to_repo(repo_root, &case_taxonomy_fastq);
    report.report_json = path_relative_to_repo(repo_root, &case_report_json);
    report.raw_backend_report = Some(path_relative_to_repo(repo_root, &case_raw_backend_report));
    bijux_dna_infra::atomic_write_json(&case_report_json, &report)?;

    let top_level_representatives = output_root.join("otu_representatives.fasta");
    copy_smoke_artifact(&case_otu_representatives, &top_level_representatives)?;
    let top_level_otu_table = output_root.join("otu_table.tsv");
    write_top_level_cluster_otus_table(
        repo_root,
        &case_otu_table,
        &top_level_otu_table,
        &top_level_representatives,
    )?;

    Ok(LocalClusterOtusSmokeReport {
        schema_version: LOCAL_CLUSTER_OTUS_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: STAGE_ID.to_string(),
        sample_id: case.sample_id.clone(),
        planned_tool_id: case.plan.tool_id.as_str().to_string(),
        report_tool_id: report.tool_id,
        clustering_threshold: report.otu_identity,
        otu_count: report.otu_count,
        sample_count: report.sample_count,
        representative_sequence_count: report.representative_sequence_count,
        otu_table_tsv: path_relative_to_repo(repo_root, &top_level_otu_table),
        representative_sequences_fasta: path_relative_to_repo(repo_root, &top_level_representatives),
        otu_representatives_fasta: path_relative_to_repo(repo_root, &top_level_representatives),
        case_report_json: path_relative_to_repo(repo_root, &case_report_json),
        taxonomy_ready_fasta: path_relative_to_repo(repo_root, &case_taxonomy_fasta),
        taxonomy_ready_fastq: path_relative_to_repo(repo_root, &case_taxonomy_fastq),
        raw_backend_report: path_relative_to_repo(repo_root, &case_raw_backend_report),
    })
}

fn write_top_level_cluster_otus_table(
    repo_root: &std::path::Path,
    case_otu_table: &std::path::Path,
    top_level_otu_table: &std::path::Path,
    top_level_representatives: &std::path::Path,
) -> Result<()> {
    let representative_path = path_relative_to_repo(repo_root, top_level_representatives);
    let raw = std::fs::read_to_string(case_otu_table)
        .with_context(|| format!("read {}", case_otu_table.display()))?;
    let mut rendered =
        String::from("sample_id\totu_id\tabundance\trepresentative_id\trepresentative_fasta\n");
    for line in raw.lines().skip(1).filter(|line| !line.trim().is_empty()) {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 3 {
            return Err(anyhow!(
                "cluster_otus local-smoke case table row must contain sample_id, otu_id, and abundance"
            ));
        }
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            fields[0], fields[1], fields[2], fields[1], representative_path
        ));
    }
    std::fs::write(top_level_otu_table, rendered)
        .with_context(|| format!("write {}", top_level_otu_table.display()))?;
    Ok(())
}

fn write_smoke_cluster_otus_uc_report(
    representatives_fasta: &std::path::Path,
    raw_backend_report: &std::path::Path,
) -> Result<()> {
    let raw = std::fs::read_to_string(representatives_fasta)
        .with_context(|| format!("read {}", representatives_fasta.display()))?;
    let mut report = String::new();
    let mut current_id = None::<String>;
    for line in raw.lines() {
        if let Some(id) = line.strip_prefix('>') {
            current_id = Some(id.trim().to_string());
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        if let Some(id) = current_id.take() {
            report.push_str(&format!("S\t0\t{}\t*\t*\t*\t*\t*\t{}\t*\n", line.len(), id));
        }
    }
    std::fs::write(raw_backend_report, report)
        .with_context(|| format!("write {}", raw_backend_report.display()))?;
    Ok(())
}

fn resolve_smoke_output_path(
    repo_root: &std::path::Path,
    path: &std::path::Path,
) -> std::path::PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &std::path::Path, path: &std::path::Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

fn copy_smoke_artifact(source: &std::path::Path, destination: &std::path::Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    std::fs::copy(source, destination).map(|_| ()).with_context(|| {
        format!(
            "copy local cluster-otus artifact {} -> {}",
            source.display(),
            destination.display()
        )
    })
}
