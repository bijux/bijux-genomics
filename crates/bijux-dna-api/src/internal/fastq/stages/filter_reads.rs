use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::internal::fastq::stages::trim_bench_common::{
    build_benchmark_context, derive_trim_delta, observe_fastq_stats, prepare_trim_bench,
};
use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_FILTER_READS,
};
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::quality::{fetch_fastq_filter_v2, insert_fastq_filter_v2};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqFilterMetrics, MetricSet};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::{
    params::{filter::FilterEffectiveParams, PairedMode},
    FilterReadsReportV1, FILTER_READS_REPORT_SCHEMA_VERSION,
};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::select_filter_tools;
use bijux_dna_planner_fastq::stage_api::fastq::filter_reads::{plan_filter, FilterPlanOptions};
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::step_runner::StageResultV1;
use bijux_dna_stage_contract::StagePlanV1;
use serde::Serialize;

use crate::internal::fastq::stages::trim_bench_common::TrimBenchInputs;

const LOCAL_FILTER_READS_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.filter_reads.local_smoke.report.v1";

#[derive(Debug, Clone, Serialize)]
struct LocalFilterReadsSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    tool_id: String,
    max_n_count: Option<u32>,
    low_complexity_threshold: Option<f64>,
    input_reads: u64,
    output_reads: u64,
    reads_dropped: u64,
    reads_removed_by_n: u64,
    reads_removed_by_entropy: u64,
    reads_removed_low_complexity: u64,
    reads_removed_by_kmer: u64,
    reads_removed_contaminant_kmer: u64,
    reads_removed_by_length: u64,
    filtered_fastq_gz: String,
    report_json: String,
    raw_backend_report: String,
    used_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalFastqRecord {
    header: String,
    sequence: String,
    plus: String,
    quality: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalFilterDropReason {
    TooManyN,
    LowComplexity,
}

#[derive(Debug, Clone)]
struct LocalFilterDecision {
    record: LocalFastqRecord,
    drop_reason: Option<LocalFilterDropReason>,
}

fn apply_thread_override(
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    threads: Option<u32>,
) -> bijux_dna_core::prelude::ToolExecutionSpecV1 {
    let mut spec = tool_spec.clone();
    if let Some(threads) = threads {
        spec.resources.threads = threads.max(1);
    }
    spec
}

/// Materialize the governed local-smoke `fastq.filter_reads` report bundle.
///
/// The written summary artifact lives at `runs/bench/local-smoke/fastq.filter_reads/report.json`
/// under the active repository root, alongside the top-level `filtered.fastq.gz`.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_filter_reads_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_fastq::stage_api::local_filter_reads_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "local-smoke fastq.filter_reads expects exactly one governed case, found {}",
            cases.len()
        ));
    };
    if case.r2.is_some() {
        return Err(anyhow!(
            "local-smoke fastq.filter_reads currently supports only the governed single-end case"
        ));
    }

    let output_root = repo_root.join("runs/bench/local-smoke/fastq.filter_reads");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let summary = materialize_local_filter_reads_smoke_case(&repo_root, case, &output_root)?;
    let report_path = output_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    Ok(report_path)
}

fn materialize_local_filter_reads_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_fastq::LocalFilterReadsSmokeCasePlan,
    output_root: &Path,
) -> Result<LocalFilterReadsSmokeReport> {
    let effective_params =
        serde_json::from_value::<FilterEffectiveParams>(case.plan.effective_params.clone())
            .context("decode filter reads local-smoke effective params")?;
    let input_r1 = repo_root.join(&case.r1);
    let output_r1 = resolve_output_path(
        repo_root,
        &required_plan_output_path(&case.plan, "filtered_reads_r1")?,
    );
    let report_path =
        resolve_output_path(repo_root, &required_plan_output_path(&case.plan, "report_json")?);
    let raw_backend_report = resolve_output_path(
        repo_root,
        &required_plan_param_path(&case.plan, "raw_backend_report")?,
    );

    for path in [&output_r1, &report_path, &raw_backend_report] {
        if let Some(parent) = path.parent() {
            bijux_dna_infra::ensure_dir(parent)?;
        }
    }

    let input_records = read_local_fastq_records(&input_r1)?;
    let decisions = input_records
        .iter()
        .cloned()
        .map(|record| LocalFilterDecision {
            drop_reason: local_filter_drop_reason(&record, &effective_params),
            record,
        })
        .collect::<Vec<_>>();
    let retained_records = decisions
        .iter()
        .filter(|decision| decision.drop_reason.is_none())
        .map(|decision| decision.record.clone())
        .collect::<Vec<_>>();
    write_local_fastq_records(&output_r1, &retained_records)?;

    let reads_removed_by_n = decisions
        .iter()
        .filter(|decision| decision.drop_reason == Some(LocalFilterDropReason::TooManyN))
        .count() as u64;
    let reads_removed_low_complexity = decisions
        .iter()
        .filter(|decision| decision.drop_reason == Some(LocalFilterDropReason::LowComplexity))
        .count() as u64;
    let reads_in = input_records.len() as u64;
    let reads_out = retained_records.len() as u64;
    let reads_dropped = reads_in.saturating_sub(reads_out);
    let bases_in = total_bases(&input_records);
    let bases_out = total_bases(&retained_records);
    let mean_q_before = mean_quality(&input_records);
    let mean_q_after = mean_quality(&retained_records);
    let backend_metrics = serde_json::json!({
        "passed_filter_reads": reads_out,
        "too_many_n_reads": reads_removed_by_n,
        "low_complexity_reads": reads_removed_low_complexity,
        "too_short_reads": 0_u64,
    });
    let report = FilterReadsReportV1 {
        schema_version: FILTER_READS_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_FILTER_READS.as_str().to_string(),
        stage_id: STAGE_FILTER_READS.as_str().to_string(),
        tool_id: case.plan.tool_id.as_str().to_string(),
        paired_mode: effective_params.paired_mode,
        threads: effective_params.threads,
        input_r1: case.r1.display().to_string(),
        input_r2: None,
        output_r1: path_relative_to_repo(repo_root, &output_r1),
        output_r2: None,
        report_json: path_relative_to_repo(repo_root, &report_path),
        max_n: effective_params.max_n,
        max_n_fraction: effective_params.max_n_fraction,
        max_n_count: effective_params.max_n_count,
        low_complexity_threshold: effective_params.low_complexity_threshold,
        entropy_threshold: effective_params.entropy_threshold,
        n_policy: Some("drop".to_string()),
        polyx_policy: effective_params.polyx_policy.clone(),
        contaminant_db: effective_params.contaminant_db.clone(),
        reads_in,
        reads_out,
        reads_dropped,
        reads_removed_by_n,
        reads_removed_by_entropy: 0,
        reads_removed_low_complexity,
        reads_removed_by_kmer: 0,
        reads_removed_contaminant_kmer: 0,
        reads_removed_by_length: 0,
        bases_in,
        bases_out,
        pairs_in: None,
        pairs_out: None,
        mean_q_before,
        mean_q_after,
        runtime_s: None,
        memory_mb: None,
        exit_code: Some(0),
        raw_backend_report: Some(path_relative_to_repo(repo_root, &raw_backend_report)),
        raw_backend_report_format: Some("fastp_json".to_string()),
        backend_metrics: Some(backend_metrics.clone()),
    };
    write_governed_filter_report(&report_path, &report)?;
    write_local_filter_backend_report(
        &raw_backend_report,
        case.plan.tool_id.as_str(),
        &backend_metrics,
    )?;

    let top_level_filtered = output_root.join("filtered.fastq.gz");
    std::fs::copy(&output_r1, &top_level_filtered).with_context(|| {
        format!(
            "copy local filter smoke output from {} to {}",
            output_r1.display(),
            top_level_filtered.display()
        )
    })?;

    Ok(LocalFilterReadsSmokeReport {
        schema_version: LOCAL_FILTER_READS_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: STAGE_FILTER_READS.as_str().to_string(),
        sample_id: case.sample_id.clone(),
        tool_id: case.plan.tool_id.as_str().to_string(),
        max_n_count: effective_params.max_n_count,
        low_complexity_threshold: effective_params.low_complexity_threshold,
        input_reads: reads_in,
        output_reads: reads_out,
        reads_dropped,
        reads_removed_by_n,
        reads_removed_by_entropy: 0,
        reads_removed_low_complexity,
        reads_removed_by_kmer: 0,
        reads_removed_contaminant_kmer: 0,
        reads_removed_by_length: 0,
        filtered_fastq_gz: path_relative_to_repo(repo_root, &top_level_filtered),
        report_json: path_relative_to_repo(repo_root, &report_path),
        raw_backend_report: path_relative_to_repo(repo_root, &raw_backend_report),
        used_fallback: true,
    })
}

fn write_governed_filter_report(report_path: &Path, report: &FilterReadsReportV1) -> Result<()> {
    bijux_dna_infra::atomic_write_json(report_path, report)
        .with_context(|| format!("write governed filter report {}", report_path.display()))
}

fn local_filter_drop_reason(
    record: &LocalFastqRecord,
    effective_params: &FilterEffectiveParams,
) -> Option<LocalFilterDropReason> {
    let max_n_count = effective_params.max_n_count.or(effective_params.max_n);
    if let Some(limit) = max_n_count {
        let n_count =
            record.sequence.bytes().filter(|base| matches!(*base, b'N' | b'n')).count() as u32;
        if n_count > limit {
            return Some(LocalFilterDropReason::TooManyN);
        }
    }

    if let Some(threshold) =
        effective_params.low_complexity_threshold.or(effective_params.entropy_threshold)
    {
        let complexity = local_complexity_score(&record.sequence);
        if complexity < threshold {
            return Some(LocalFilterDropReason::LowComplexity);
        }
    }

    None
}

fn local_complexity_score(sequence: &str) -> f64 {
    let bytes = sequence.as_bytes();
    if bytes.len() <= 1 {
        return 0.0;
    }
    let transitions = bytes.windows(2).filter(|window| window[0] != window[1]).count() as f64;
    (transitions / (bytes.len() as f64 - 1.0)) * 100.0
}

fn total_bases(records: &[LocalFastqRecord]) -> u64 {
    records.iter().map(|record| record.sequence.len() as u64).sum()
}

fn mean_quality(records: &[LocalFastqRecord]) -> f64 {
    let total_bases = total_bases(records);
    if total_bases == 0 {
        return 0.0;
    }
    let total_quality = records
        .iter()
        .flat_map(|record| record.quality.bytes())
        .map(|value| u64::from(value.saturating_sub(33)))
        .sum::<u64>();
    total_quality as f64 / total_bases as f64
}

fn read_local_fastq_records(path: &Path) -> Result<Vec<LocalFastqRecord>> {
    let reader: Box<dyn BufRead> = if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
    {
        let file = std::fs::File::open(path)?;
        let decoder = flate2::read::MultiGzDecoder::new(file);
        Box::new(BufReader::new(decoder))
    } else {
        Box::new(BufReader::new(std::fs::File::open(path)?))
    };

    let mut lines = reader.lines();
    let mut records = Vec::new();
    while let Some(header) = lines.next() {
        let header = header?;
        let sequence =
            lines.next().ok_or_else(|| anyhow!("truncated FASTQ at {}", path.display()))??;
        let plus =
            lines.next().ok_or_else(|| anyhow!("truncated FASTQ at {}", path.display()))??;
        let quality =
            lines.next().ok_or_else(|| anyhow!("truncated FASTQ at {}", path.display()))??;
        records.push(LocalFastqRecord { header, sequence, plus, quality });
    }
    Ok(records)
}

fn write_local_fastq_records(path: &Path, records: &[LocalFastqRecord]) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }

    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
    {
        let file = std::fs::File::create(path)?;
        let mut encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        for record in records {
            writeln!(encoder, "{}", record.header)?;
            writeln!(encoder, "{}", record.sequence)?;
            writeln!(encoder, "{}", record.plus)?;
            writeln!(encoder, "{}", record.quality)?;
        }
        encoder.finish()?;
    } else {
        let file = std::fs::File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);
        for record in records {
            writeln!(writer, "{}", record.header)?;
            writeln!(writer, "{}", record.sequence)?;
            writeln!(writer, "{}", record.plus)?;
            writeln!(writer, "{}", record.quality)?;
        }
        writer.flush()?;
    }

    Ok(())
}

fn write_local_filter_backend_report(
    path: &Path,
    tool_id: &str,
    backend_metrics: &serde_json::Value,
) -> Result<()> {
    match tool_id {
        "fastp" => bijux_dna_infra::write_bytes(
            path,
            serde_json::json!({
                "filtering_result": {
                    "passed_filter_reads": backend_metrics["passed_filter_reads"],
                    "low_quality_reads": 0_u64,
                    "too_many_N_reads": backend_metrics["too_many_n_reads"],
                    "too_short_reads": 0_u64,
                    "low_complexity_reads": backend_metrics["low_complexity_reads"],
                }
            })
            .to_string(),
        )
        .with_context(|| format!("write local filter backend report {}", path.display())),
        _ => Err(anyhow!(
            "local-smoke fastq.filter_reads does not support backend report materialization for tool `{tool_id}`"
        )),
    }
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

fn required_plan_output_path(plan: &StagePlanV1, output_id: &str) -> Result<PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| {
            anyhow!(
                "filter_reads plan is missing governed output `{output_id}` for tool {}",
                plan.tool_id.as_str()
            )
        })
}

fn required_plan_param_path(plan: &StagePlanV1, param_name: &str) -> Result<PathBuf> {
    plan.params.get(param_name).and_then(serde_json::Value::as_str).map(PathBuf::from).ok_or_else(
        || {
            anyhow!(
                "filter_reads plan is missing governed parameter path `{param_name}` for tool {}",
                plan.tool_id.as_str()
            )
        },
    )
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_filter<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqFilterArgs,
) -> Result<BenchOutcome<bijux_dna_analyze::FastqFilterMetrics>> {
    let selected_tools = select_filter_benchmark_tools(args)?;
    let setup =
        prepare_filter_benchmark_setup(catalog, platform, runner_override, args, &selected_tools)?;

    if args.explain {
        write_filter_benchmark_explain(&setup)?;
    }

    ensure_filter_benchmark_qa(catalog, platform, &setup.tools)?;

    let store = FilterBenchmarkStore::from_setup(&setup);
    let conn = bijux_dna_analyze::open_sqlite(&store.sqlite_path).context("open bench sqlite")?;
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::<BenchmarkRecord<FastqFilterMetrics>>::new();
    for tool in &setup.tools {
        let tool_plan = prepare_filter_tool_plan(catalog, platform, args, &setup, jobs, tool)?;
        let cache_identity = FilterCacheIdentity::from_plan(platform, &setup, &tool_plan);
        if let Ok(Some(record)) = fetch_fastq_filter_v2(
            &conn,
            &cache_identity.tool,
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
        let execution = execute_filter_tool(&tool_plan, setup.bench_inputs.runner, jobs)?;
        if let Some(failure) = filter_tool_failure(&tool_plan, execution.result.exit_code) {
            failures.push(failure);
            continue;
        }
        let record = build_filter_record(&FilterRecordInputs {
            catalog,
            platform,
            bench_inputs: &setup.bench_inputs,
            input_stats_r2: setup.input_stats_r2.as_ref(),
            input_hash: &setup.input_hash,
            tool_plan: &tool_plan,
            execution: &execution,
        })?;
        append_jsonl(&store.jsonl_path, &record).context("write bench.jsonl")?;
        insert_fastq_filter_v2(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: setup.bench_inputs.bench_dir,
        explain: args.explain,
    })
}

struct FilterBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    bench_inputs: TrimBenchInputs,
    input_hash: String,
    input_stats_r2: Option<SeqkitMetrics>,
    options: FilterPlanOptions,
}

struct FilterBenchmarkStore {
    sqlite_path: PathBuf,
    jsonl_path: PathBuf,
}

impl FilterBenchmarkStore {
    fn from_setup(setup: &FilterBenchmarkSetup) -> Self {
        Self {
            sqlite_path: setup.bench_inputs.bench_dir.join("bench.sqlite"),
            jsonl_path: setup.bench_inputs.bench_dir.join("bench.jsonl"),
        }
    }
}

struct FilterToolPlan {
    tool: String,
    tool_spec: ToolExecutionSpecV1,
    plan: StagePlanV1,
    outputs: FilterPlanOutputs,
    params_hash: String,
    image_digest: String,
}

struct FilterPlanOutputs {
    reads: PathBuf,
    reads_r2: Option<PathBuf>,
}

struct FilterToolExecution {
    result: StageResultV1,
}

struct FilterCacheIdentity {
    tool: String,
    tool_version: String,
    image_digest: String,
    runner: String,
    platform: String,
    input_hash: String,
    params_hash: String,
}

impl FilterCacheIdentity {
    fn from_plan(
        platform: &PlatformSpec,
        setup: &FilterBenchmarkSetup,
        tool_plan: &FilterToolPlan,
    ) -> Self {
        Self {
            tool: tool_plan.tool.clone(),
            tool_version: tool_plan.tool_spec.tool_version.clone(),
            image_digest: tool_plan.image_digest.clone(),
            runner: setup.bench_inputs.runner.to_string(),
            platform: platform.name.clone(),
            input_hash: setup.input_hash.clone(),
            params_hash: tool_plan.params_hash.clone(),
        }
    }
}

struct FilterRecordInputs<'a, S: ::std::hash::BuildHasher> {
    catalog: &'a HashMap<String, ToolImageSpec, S>,
    platform: &'a PlatformSpec,
    bench_inputs: &'a TrimBenchInputs,
    input_stats_r2: Option<&'a SeqkitMetrics>,
    input_hash: &'a str,
    tool_plan: &'a FilterToolPlan,
    execution: &'a FilterToolExecution,
}

struct FilterReportBuildInputs<'a> {
    bench_inputs: &'a TrimBenchInputs,
    tool_plan: &'a FilterToolPlan,
    report_path: &'a Path,
    accounting: &'a FilterReadAccounting,
    output_stats_r1: &'a SeqkitMetrics,
    execution: &'a FilterToolExecution,
}

struct FilterBenchmarkRecordInputs<'a> {
    platform: &'a PlatformSpec,
    bench_inputs: &'a TrimBenchInputs,
    input_hash: &'a str,
    tool_plan: &'a FilterToolPlan,
    execution: &'a FilterToolExecution,
}

struct FilterReadAccounting {
    reads_in: u64,
    reads_out: u64,
    reads_dropped: u64,
    bases_in: u64,
    bases_out: u64,
    pairs_in: Option<u64>,
    pairs_out: Option<u64>,
}

struct FilterObservedOutputs {
    r1: SeqkitMetrics,
    r2: Option<SeqkitMetrics>,
}

struct FilterBackendReport {
    raw_backend_report: Option<String>,
    raw_backend_report_format: Option<String>,
    metrics: Option<serde_json::Value>,
    removal_counts: FilterRemovalCounts,
}

struct FilterReportParams {
    input_r1: String,
    input_r2: Option<String>,
    max_n: Option<u32>,
    max_n_fraction: Option<f64>,
    max_n_count: Option<u32>,
    low_complexity_threshold: Option<f64>,
    entropy_threshold: Option<f64>,
    polyx_policy: Option<String>,
    contaminant_db: Option<String>,
}

struct FilterReportOutputs {
    paired_mode: PairedMode,
    output_r1: String,
    output_r2: Option<String>,
    report_json: String,
}

struct FilterReportMeasurements {
    reads_in: u64,
    reads_out: u64,
    reads_dropped: u64,
    reads_removed_by_n: u64,
    reads_removed_by_entropy: u64,
    reads_removed_low_complexity: u64,
    reads_removed_by_kmer: u64,
    reads_removed_contaminant_kmer: u64,
    reads_removed_by_length: u64,
    bases_in: u64,
    bases_out: u64,
    pairs_in: Option<u64>,
    pairs_out: Option<u64>,
}

impl FilterReportMeasurements {
    fn from_accounting(
        accounting: &FilterReadAccounting,
        removal_counts: FilterRemovalCounts,
    ) -> Self {
        Self {
            reads_in: accounting.reads_in,
            reads_out: accounting.reads_out,
            reads_dropped: accounting.reads_dropped,
            reads_removed_by_n: removal_counts.reads_removed_by_n,
            reads_removed_by_entropy: removal_counts.reads_removed_by_entropy,
            reads_removed_low_complexity: removal_counts.reads_removed_low_complexity,
            reads_removed_by_kmer: removal_counts.reads_removed_by_kmer,
            reads_removed_contaminant_kmer: removal_counts.reads_removed_contaminant_kmer,
            reads_removed_by_length: removal_counts.reads_removed_by_length,
            bases_in: accounting.bases_in,
            bases_out: accounting.bases_out,
            pairs_in: accounting.pairs_in,
            pairs_out: accounting.pairs_out,
        }
    }
}

impl FilterReportOutputs {
    fn from_paths(output_reads: &Path, output_reads_r2: Option<&Path>, report_path: &Path) -> Self {
        Self {
            paired_mode: filter_paired_mode(output_reads_r2),
            output_r1: output_reads.display().to_string(),
            output_r2: output_reads_r2.map(|path| path.display().to_string()),
            report_json: report_path.display().to_string(),
        }
    }
}

impl FilterReportParams {
    fn from_params(params: &serde_json::Value, fallback_r1: &Path) -> Self {
        Self {
            input_r1: params
                .get("input_r1")
                .and_then(serde_json::Value::as_str)
                .map_or_else(|| fallback_r1.display().to_string(), ToString::to_string),
            input_r2: string_param(params, "input_r2"),
            max_n: u32_param(params, "max_n"),
            max_n_fraction: params.get("max_n_fraction").and_then(serde_json::Value::as_f64),
            max_n_count: u32_param(params, "max_n_count"),
            low_complexity_threshold: params
                .get("low_complexity_threshold")
                .and_then(serde_json::Value::as_f64),
            entropy_threshold: params.get("entropy_threshold").and_then(serde_json::Value::as_f64),
            polyx_policy: string_param(params, "polyx_policy"),
            contaminant_db: string_param(params, "kmer_ref"),
        }
    }
}

fn string_param(params: &serde_json::Value, name: &str) -> Option<String> {
    params.get(name).and_then(serde_json::Value::as_str).map(ToString::to_string)
}

fn u32_param(params: &serde_json::Value, name: &str) -> Option<u32> {
    params.get(name).and_then(serde_json::Value::as_u64).and_then(|value| u32::try_from(value).ok())
}

fn select_filter_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqFilterArgs,
) -> Result<Vec<String>> {
    let tools = select_filter_tools(&args.tools)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_FILTER_READS.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_FILTER_READS.as_str(), &header);
    Ok(tools)
}

fn prepare_filter_benchmark_setup<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqFilterArgs,
    selected_tools: &[String],
) -> Result<FilterBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools =
        filter_tools_by_role(STAGE_FILTER_READS.as_str(), selected_tools, &registry, false)?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_FILTER_READS,
    )?;
    let input_hash = if let Some(r2) = args.r2.as_deref() {
        format!("{}+{}", bench_inputs.input_hash, bijux_dna_infra::hash_file_sha256(r2)?)
    } else {
        bench_inputs.input_hash.clone()
    };
    let input_stats_r2 = if let Some(r2) = args.r2.as_deref() {
        Some(observe_fastq_stats(catalog, platform, bench_inputs.runner, r2)?)
    } else {
        None
    };
    let options = FilterPlanOptions {
        threads: args.threads,
        max_n: args.max_n,
        max_n_fraction: args.max_n_fraction,
        max_n_count: args.max_n_count,
        low_complexity_threshold: args.low_complexity_threshold,
        entropy_threshold: args.entropy_threshold,
        kmer_ref: args.kmer_ref.clone(),
        redundant_filters: Vec::new(),
        polyx_policy: args.polyx_policy.clone(),
    };
    Ok(FilterBenchmarkSetup { registry, tools, bench_inputs, input_hash, input_stats_r2, options })
}

fn write_filter_benchmark_explain(setup: &FilterBenchmarkSetup) -> Result<()> {
    write_explain_md(
        &setup.bench_inputs.bench_dir,
        STAGE_FILTER_READS.as_str(),
        &setup.tools,
        &[],
        None,
    )?;
    write_explain_plan_json(
        &setup.bench_inputs.bench_dir,
        STAGE_FILTER_READS.as_str(),
        &setup.tools,
        &setup.registry,
        None,
    )
}

fn ensure_filter_benchmark_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(STAGE_FILTER_READS.as_str(), tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_FILTER_READS.as_str(), tools, platform, catalog)
}

fn prepare_filter_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqFilterArgs,
    setup: &FilterBenchmarkSetup,
    jobs: usize,
    tool: &str,
) -> Result<FilterToolPlan> {
    let out_dir = setup.bench_inputs.tools_root.join(tool);
    bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
    let tool_spec = build_tool_execution_spec(
        STAGE_FILTER_READS.as_str(),
        tool,
        &setup.registry,
        catalog,
        platform,
    )?;
    let tool_spec = apply_thread_override(&tool_spec, args.threads);
    let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
    let plan = plan_filter(&tool_spec, &args.r1, args.r2.as_deref(), &out_dir, &setup.options)?;
    let outputs = resolve_filter_outputs(&plan, args.r2.is_some())?;
    let params_hash = stable_params_hash(&plan.params);
    let image_digest = tool_spec
        .image
        .digest
        .as_ref()
        .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
        .clone();
    Ok(FilterToolPlan {
        tool: tool.to_string(),
        tool_spec,
        plan,
        outputs,
        params_hash,
        image_digest,
    })
}

fn resolve_filter_outputs(plan: &StagePlanV1, paired_end: bool) -> Result<FilterPlanOutputs> {
    let reads = plan
        .io
        .outputs
        .first()
        .ok_or_else(|| anyhow!("filter plan missing primary reads output"))?
        .path
        .clone();
    let reads_r2 = plan.io.outputs.get(1).map(|artifact| artifact.path.clone());
    if paired_end && reads_r2.is_none() {
        return Err(anyhow!("filter paired-end plan missing R2 reads output"));
    }
    Ok(FilterPlanOutputs { reads, reads_r2 })
}

fn execute_filter_tool(
    tool_plan: &FilterToolPlan,
    runner: RuntimeKind,
    jobs: usize,
) -> Result<FilterToolExecution> {
    let result = execute_plans_with_jobs(
        vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&tool_plan.plan)],
        runner,
        jobs,
    )?
    .into_iter()
    .next()
    .ok_or_else(|| anyhow!("missing execution result for {}", tool_plan.tool))?;
    Ok(FilterToolExecution { result })
}

fn filter_tool_failure(tool_plan: &FilterToolPlan, exit_code: i32) -> Option<RawFailure> {
    (exit_code != 0).then(|| RawFailure {
        stage: STAGE_FILTER_READS.as_str().to_string(),
        tool: tool_plan.tool.clone(),
        reason: format!("tool {} failed with status {exit_code}", tool_plan.tool),
        category: ErrorCategory::ToolError,
    })
}

fn build_filter_record<S: ::std::hash::BuildHasher>(
    inputs: &FilterRecordInputs<'_, S>,
) -> Result<BenchmarkRecord<FastqFilterMetrics>> {
    let platform = inputs.platform;
    let bench_inputs = inputs.bench_inputs;
    let input_stats_r2 = inputs.input_stats_r2;
    let input_hash = inputs.input_hash;
    let tool_plan = inputs.tool_plan;
    let output_reads = tool_plan.outputs.reads.as_path();
    let execution = inputs.execution;
    let output_stats = observe_filter_outputs(inputs)?;
    let output_stats_r1 = output_stats.r1;
    let output_stats_r2 = output_stats.r2;
    let accounting = filter_read_accounting(
        bench_inputs.input_stats,
        input_stats_r2,
        output_stats_r1,
        output_stats_r2.as_ref(),
    );
    let out_dir = output_reads.parent().ok_or_else(|| anyhow!("filter output has no parent"))?;
    let report_path = out_dir.join("filter_report.json");
    let report = build_filter_report(&FilterReportBuildInputs {
        bench_inputs,
        tool_plan,
        report_path: &report_path,
        accounting: &accounting,
        output_stats_r1: &output_stats_r1,
        execution,
    });
    let metric_set = build_filter_metric_set(&report, &bench_inputs.input_stats, &output_stats_r1)?;
    write_filter_artifacts(out_dir, &report_path, &report, &metric_set)?;

    build_filter_benchmark_record(
        &FilterBenchmarkRecordInputs { platform, bench_inputs, input_hash, tool_plan, execution },
        metric_set,
    )
}

fn build_filter_benchmark_record(
    inputs: &FilterBenchmarkRecordInputs<'_>,
    metric_set: MetricSet<FastqFilterMetrics>,
) -> Result<BenchmarkRecord<FastqFilterMetrics>> {
    let context = build_filter_context(inputs);
    let record = BenchmarkRecord {
        context,
        execution: filter_execution_metrics(inputs.execution),
        metrics: metric_set,
    };
    record.validate()?;
    Ok(record)
}

fn filter_execution_metrics(execution: &FilterToolExecution) -> ExecutionMetrics {
    ExecutionMetrics {
        runtime_s: execution.result.runtime_s,
        memory_mb: execution.result.memory_mb,
        exit_code: execution.result.exit_code,
    }
}

fn build_filter_context(
    inputs: &FilterBenchmarkRecordInputs<'_>,
) -> bijux_dna_analyze::BenchmarkContext {
    build_benchmark_context(
        &inputs.tool_plan.tool,
        inputs.tool_plan.tool_spec.tool_version.clone(),
        inputs.tool_plan.image_digest.clone(),
        inputs.bench_inputs.runner,
        inputs.platform,
        inputs.input_hash.to_string(),
        inputs.tool_plan.plan.params.clone(),
    )
}

fn build_filter_report(inputs: &FilterReportBuildInputs<'_>) -> FilterReadsReportV1 {
    let params = &inputs.tool_plan.plan.params;
    let backend_report = filter_backend_report(params);
    let report_params = FilterReportParams::from_params(params, &inputs.bench_inputs.r1);
    let report_outputs = FilterReportOutputs::from_paths(
        &inputs.tool_plan.outputs.reads,
        inputs.tool_plan.outputs.reads_r2.as_deref(),
        inputs.report_path,
    );
    let report_measurements =
        FilterReportMeasurements::from_accounting(inputs.accounting, backend_report.removal_counts);
    FilterReadsReportV1 {
        schema_version: FILTER_READS_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_FILTER_READS.as_str().to_string(),
        stage_id: STAGE_FILTER_READS.as_str().to_string(),
        tool_id: inputs.tool_plan.tool.clone(),
        paired_mode: report_outputs.paired_mode,
        threads: inputs.tool_plan.tool_spec.resources.threads,
        input_r1: report_params.input_r1,
        input_r2: report_params.input_r2,
        output_r1: report_outputs.output_r1,
        output_r2: report_outputs.output_r2,
        report_json: report_outputs.report_json,
        max_n: report_params.max_n,
        max_n_fraction: report_params.max_n_fraction,
        max_n_count: report_params.max_n_count,
        low_complexity_threshold: report_params.low_complexity_threshold,
        entropy_threshold: report_params.entropy_threshold,
        n_policy: Some("drop".to_string()),
        polyx_policy: report_params.polyx_policy,
        contaminant_db: report_params.contaminant_db,
        reads_in: report_measurements.reads_in,
        reads_out: report_measurements.reads_out,
        reads_dropped: report_measurements.reads_dropped,
        reads_removed_by_n: report_measurements.reads_removed_by_n,
        reads_removed_by_entropy: report_measurements.reads_removed_by_entropy,
        reads_removed_low_complexity: report_measurements.reads_removed_low_complexity,
        reads_removed_by_kmer: report_measurements.reads_removed_by_kmer,
        reads_removed_contaminant_kmer: report_measurements.reads_removed_contaminant_kmer,
        reads_removed_by_length: report_measurements.reads_removed_by_length,
        bases_in: report_measurements.bases_in,
        bases_out: report_measurements.bases_out,
        pairs_in: report_measurements.pairs_in,
        pairs_out: report_measurements.pairs_out,
        mean_q_before: inputs.bench_inputs.input_stats.mean_q,
        mean_q_after: inputs.output_stats_r1.mean_q,
        runtime_s: Some(inputs.execution.result.runtime_s),
        memory_mb: Some(inputs.execution.result.memory_mb),
        exit_code: Some(inputs.execution.result.exit_code),
        raw_backend_report: backend_report.raw_backend_report,
        raw_backend_report_format: backend_report.raw_backend_report_format,
        backend_metrics: backend_report.metrics,
    }
}

fn filter_paired_mode(output_reads_r2: Option<&Path>) -> PairedMode {
    if output_reads_r2.is_some() {
        PairedMode::PairedEnd
    } else {
        PairedMode::SingleEnd
    }
}

fn write_filter_artifacts(
    out_dir: &Path,
    report_path: &Path,
    report: &FilterReadsReportV1,
    metric_set: &MetricSet<FastqFilterMetrics>,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(report_path, report).context("write filter report")?;
    let metrics_json = serde_json::to_value(metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write filter metrics")
}

fn build_filter_metric_set(
    report: &FilterReadsReportV1,
    input_stats_r1: &SeqkitMetrics,
    output_stats_r1: &SeqkitMetrics,
) -> Result<MetricSet<FastqFilterMetrics>> {
    validate_filter_removal_evidence(report)?;
    let metrics = filter_metrics_from_report(report, input_stats_r1, output_stats_r1);
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;
    Ok(metric_set)
}

fn validate_filter_removal_evidence(report: &FilterReadsReportV1) -> Result<()> {
    if let Some(passed_filter_reads) = report
        .backend_metrics
        .as_ref()
        .and_then(|metrics| metrics.get("passed_filter_reads"))
        .and_then(serde_json::Value::as_u64)
    {
        if passed_filter_reads > report.reads_in {
            return Err(anyhow!(
                "filter backend passed_filter_reads {passed_filter_reads} exceeds reads_in {}",
                report.reads_in
            ));
        }
    }
    for (name, value) in [
        ("reads_removed_by_n", report.reads_removed_by_n),
        ("reads_removed_by_entropy", report.reads_removed_by_entropy),
        ("reads_removed_low_complexity", report.reads_removed_low_complexity),
        ("reads_removed_by_kmer", report.reads_removed_by_kmer),
        ("reads_removed_contaminant_kmer", report.reads_removed_contaminant_kmer),
        ("reads_removed_by_length", report.reads_removed_by_length),
    ] {
        if value > report.reads_dropped {
            return Err(anyhow!(
                "filter {name} {value} exceeds reads_dropped {}",
                report.reads_dropped
            ));
        }
    }
    Ok(())
}

fn filter_metrics_from_report(
    report: &FilterReadsReportV1,
    input_stats_r1: &SeqkitMetrics,
    output_stats_r1: &SeqkitMetrics,
) -> FastqFilterMetrics {
    FastqFilterMetrics {
        reads_in: report.reads_in,
        reads_out: report.reads_out,
        reads_dropped: report.reads_dropped,
        reads_removed_by_n: report.reads_removed_by_n,
        reads_removed_by_entropy: report.reads_removed_by_entropy,
        reads_removed_low_complexity: report.reads_removed_low_complexity,
        reads_removed_by_kmer: report.reads_removed_by_kmer,
        reads_removed_contaminant_kmer: report.reads_removed_contaminant_kmer,
        reads_removed_by_length: report.reads_removed_by_length,
        bases_in: report.bases_in,
        bases_out: report.bases_out,
        pairs_in: report.pairs_in,
        pairs_out: report.pairs_out,
        mean_q_before: report.mean_q_before,
        mean_q_after: report.mean_q_after,
        delta_metrics: derive_trim_delta(input_stats_r1, output_stats_r1),
    }
}

fn observe_filter_outputs<S: ::std::hash::BuildHasher>(
    inputs: &FilterRecordInputs<'_, S>,
) -> Result<FilterObservedOutputs> {
    let output_reads = inputs.tool_plan.outputs.reads.as_path();
    let output_stats_r1 = if inputs.execution.result.exit_code == 0 && output_reads.exists() {
        observe_fastq_stats(
            inputs.catalog,
            inputs.platform,
            inputs.bench_inputs.runner,
            output_reads,
        )?
    } else {
        inputs.bench_inputs.input_stats
    };
    let output_stats_r2 =
        if let Some(output_reads_r2) = inputs.tool_plan.outputs.reads_r2.as_deref() {
            if inputs.execution.result.exit_code == 0 && output_reads_r2.exists() {
                Some(observe_fastq_stats(
                    inputs.catalog,
                    inputs.platform,
                    inputs.bench_inputs.runner,
                    output_reads_r2,
                )?)
            } else {
                inputs.input_stats_r2.copied()
            }
        } else {
            None
        };

    Ok(FilterObservedOutputs { r1: output_stats_r1, r2: output_stats_r2 })
}

fn filter_read_accounting(
    input_stats_r1: SeqkitMetrics,
    input_stats_r2: Option<&SeqkitMetrics>,
    output_stats_r1: SeqkitMetrics,
    output_stats_r2: Option<&SeqkitMetrics>,
) -> FilterReadAccounting {
    let reads_in = input_stats_r1.reads + input_stats_r2.map_or(0, |stats| stats.reads);
    let reads_out = output_stats_r1.reads + output_stats_r2.map_or(0, |stats| stats.reads);
    let bases_in = input_stats_r1.bases + input_stats_r2.map_or(0, |stats| stats.bases);
    let bases_out = output_stats_r1.bases + output_stats_r2.map_or(0, |stats| stats.bases);
    let pairs_in = input_stats_r2.map(|stats| input_stats_r1.reads.min(stats.reads));
    let pairs_out = output_stats_r2.map(|stats| output_stats_r1.reads.min(stats.reads));

    FilterReadAccounting {
        reads_in,
        reads_out,
        reads_dropped: reads_in.saturating_sub(reads_out),
        bases_in,
        bases_out,
        pairs_in,
        pairs_out,
    }
}

fn filter_backend_report(params: &serde_json::Value) -> FilterBackendReport {
    let raw_backend_report = params
        .get("raw_backend_report")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    let raw_backend_report_format = params
        .get("raw_backend_report_format")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    let metrics = parse_filter_backend_metrics(
        raw_backend_report.as_deref().map(Path::new),
        raw_backend_report_format.as_deref(),
    );
    let removal_counts =
        derive_filter_removal_counts(metrics.as_ref(), args_kmer_filter_requested(params));

    FilterBackendReport { raw_backend_report, raw_backend_report_format, metrics, removal_counts }
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(clippy::struct_field_names)]
struct FilterRemovalCounts {
    reads_removed_by_n: u64,
    reads_removed_by_entropy: u64,
    reads_removed_low_complexity: u64,
    reads_removed_by_kmer: u64,
    reads_removed_contaminant_kmer: u64,
    reads_removed_by_length: u64,
}

fn args_kmer_filter_requested(params: &serde_json::Value) -> bool {
    params
        .get("kmer_ref")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| !value.is_empty())
}

fn parse_filter_backend_metrics(
    raw_backend_report: Option<&std::path::Path>,
    raw_backend_report_format: Option<&str>,
) -> Option<serde_json::Value> {
    match (raw_backend_report, raw_backend_report_format) {
        (Some(path), Some("fastp_json")) => std::fs::read_to_string(path)
            .ok()
            .and_then(|raw| bijux_dna_domain_fastq::observer::parse_fastp_metrics(&raw).ok())
            .and_then(|metrics| serde_json::to_value(metrics).ok()),
        (Some(path), Some("bbduk_stats")) => std::fs::read_to_string(path)
            .ok()
            .and_then(|raw| bijux_dna_domain_fastq::observer::parse_bbduk_reads_removed(&raw).ok())
            .map(|reads_removed| {
                serde_json::json!({
                    "schema_version": "bijux.bbduk.filter.metrics.v1",
                    "reads_removed": reads_removed
                })
            }),
        _ => None,
    }
}

fn derive_filter_removal_counts(
    backend_metrics: Option<&serde_json::Value>,
    kmer_filter_requested: bool,
) -> FilterRemovalCounts {
    let mut counts = FilterRemovalCounts::default();
    let Some(metrics) = backend_metrics.and_then(serde_json::Value::as_object) else {
        return counts;
    };
    counts.reads_removed_by_n =
        metrics.get("too_many_n_reads").and_then(serde_json::Value::as_u64).unwrap_or(0);
    counts.reads_removed_by_length =
        metrics.get("too_short_reads").and_then(serde_json::Value::as_u64).unwrap_or(0);
    if kmer_filter_requested {
        let removed = metrics.get("reads_removed").and_then(serde_json::Value::as_u64).unwrap_or(0);
        counts.reads_removed_by_kmer = removed;
        counts.reads_removed_contaminant_kmer = removed;
    }
    counts
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::{derive_filter_removal_counts, parse_filter_backend_metrics};

    #[test]
    fn parse_filter_backend_metrics_reads_fastp_json() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("fastp.filter.json");
        bijux_dna_infra::write_bytes(
            &report_path,
            serde_json::json!({
                "filtering_result": {
                    "passed_filter_reads": 95_u64,
                    "low_quality_reads": 4_u64,
                    "too_many_N_reads": 2_u64,
                    "too_short_reads": 3_u64
                }
            })
            .to_string(),
        )
        .expect("write fastp report");

        let parsed =
            parse_filter_backend_metrics(Some(&report_path), Some("fastp_json")).expect("metrics");
        assert_eq!(parsed["passed_filter_reads"], serde_json::json!(95_u64));
        assert_eq!(parsed["too_many_n_reads"], serde_json::json!(2_u64));
        assert_eq!(parsed["too_short_reads"], serde_json::json!(3_u64));
    }

    #[test]
    fn derive_filter_removal_counts_maps_backend_specific_fields() {
        let counts = derive_filter_removal_counts(
            Some(&serde_json::json!({
                "too_many_n_reads": 2_u64,
                "too_short_reads": 3_u64,
                "reads_removed": 11_u64
            })),
            true,
        );
        assert_eq!(counts.reads_removed_by_n, 2);
        assert_eq!(counts.reads_removed_by_length, 3);
        assert_eq!(counts.reads_removed_by_kmer, 11);
        assert_eq!(counts.reads_removed_contaminant_kmer, 11);
    }
}
