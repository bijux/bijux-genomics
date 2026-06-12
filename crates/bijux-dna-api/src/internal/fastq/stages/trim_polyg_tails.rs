use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::query_shared::{
    fetch_fastq_trim_polyg_v1, insert_fastq_trim_polyg_v1,
};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqTrimPolygMetrics};
use bijux_dna_core::ids::StageId;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_domain_fastq::observer::{parse_bbduk_reads_removed, parse_fastp_metrics};
use bijux_dna_domain_fastq::params::trim::TrimPolygTailsParams;
use bijux_dna_domain_fastq::TrimPolygReportV1;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, polyx_bank_context, preflight_stage, FastqArtifactKind,
    RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;

use super::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context, derive_trim_delta, observe_fastq_stats,
    prepare_trim_bench,
};
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_TRIM_POLYG_TAILS,
};
use serde::Serialize;

const LOCAL_TRIM_POLYG_TAILS_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.fastq.trim_polyg_tails.local_smoke.metrics.v1";

#[derive(Debug, Clone, Serialize)]
struct LocalTrimPolygTailsSmokeMetrics {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    tool_id: String,
    trim_polyg: bool,
    min_polyg_run: u32,
    input_reads: u64,
    output_reads: u64,
    reads_retained: u64,
    reads_dropped: u64,
    input_bases: u64,
    output_bases: u64,
    bases_removed: u64,
    trimmed_tail_count: u64,
    bases_trimmed_polyg: u64,
    trimmed_fastq_gz: String,
    report_json: String,
    raw_backend_report: String,
    used_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FastqRecord {
    header: String,
    sequence: String,
    plus: String,
    quality: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct TrimPolygDelta {
    trimmed_tail_count: u64,
    bases_trimmed_polyg: u64,
}

fn load_governed_trim_polyg_report(report_path: &Path) -> Result<TrimPolygReportV1> {
    let raw = std::fs::read_to_string(report_path)
        .with_context(|| format!("read governed trim-polyg report {}", report_path.display()))?;
    bijux_dna_domain_fastq::observer::parse_trim_polyg_report(&raw)
        .with_context(|| format!("parse governed trim-polyg report {}", report_path.display()))
}

fn write_governed_trim_polyg_report(report_path: &Path, report: &TrimPolygReportV1) -> Result<()> {
    bijux_dna_infra::atomic_write_json(report_path, report)
        .with_context(|| format!("write governed trim-polyg report {}", report_path.display()))
}

fn derive_trim_polyg_delta_for_path_pair(
    input_path: &Path,
    output_path: &Path,
) -> Result<TrimPolygDelta> {
    let input_records = read_fastq_records(input_path)?;
    let output_records = read_fastq_records(output_path)?;
    derive_trim_polyg_delta(&input_records, &output_records).with_context(|| {
        format!(
            "derive trim-polyg delta from {} -> {}",
            input_path.display(),
            output_path.display()
        )
    })
}

fn derive_trim_polyg_delta(
    input_records: &[FastqRecord],
    output_records: &[FastqRecord],
) -> Result<TrimPolygDelta> {
    if input_records.len() != output_records.len() {
        return Err(anyhow!(
            "trim-polyg delta requires stable record cardinality, found {} input reads and {} output reads",
            input_records.len(),
            output_records.len()
        ));
    }

    let mut delta = TrimPolygDelta::default();
    for (index, (input, output)) in input_records.iter().zip(output_records.iter()).enumerate() {
        if input.header != output.header {
            return Err(anyhow!(
                "trim-polyg delta requires stable record identity at row {}: input header `{}` != output header `{}`",
                index,
                input.header,
                output.header
            ));
        }
        if input.plus != output.plus {
            return Err(anyhow!(
                "trim-polyg delta requires stable FASTQ plus lines at row {} for `{}`",
                index,
                input.header
            ));
        }
        if output.sequence.len() > input.sequence.len()
            || output.quality.len() > input.quality.len()
        {
            return Err(anyhow!(
                "trim-polyg output cannot be longer than input at row {} for `{}`",
                index,
                input.header
            ));
        }
        if !input.sequence.starts_with(&output.sequence)
            || !input.quality.starts_with(&output.quality)
        {
            return Err(anyhow!(
                "trim-polyg output must preserve the input prefix at row {} for `{}`",
                index,
                input.header
            ));
        }

        let trimmed_suffix = &input.sequence[output.sequence.len()..];
        if !trimmed_suffix.is_empty() {
            if !trimmed_suffix.bytes().all(|base| base == b'G') {
                return Err(anyhow!(
                    "trim-polyg output removed non-terminal-polyG sequence at row {} for `{}`",
                    index,
                    input.header
                ));
            }
            delta.trimmed_tail_count += 1;
            delta.bases_trimmed_polyg += trimmed_suffix.len() as u64;
        }
    }

    Ok(delta)
}

fn resolve_requested_tools(raw: &[String]) -> Vec<String> {
    if raw.is_empty() || (raw.len() == 1 && raw[0] == "auto") {
        return admitted_stage_tools();
    }
    if raw.len() == 1 && raw[0] == "all" {
        return admitted_stage_tools();
    }
    raw.to_vec()
}

fn admitted_stage_tools() -> Vec<String> {
    bijux_dna_planner_fastq::stage_api::allowed_tools_for_stage(&StageId::new(
        STAGE_TRIM_POLYG_TAILS.as_str(),
    ))
    .into_iter()
    .map(|tool_id| tool_id.to_string())
    .collect()
}

/// Materialize the governed local-smoke `fastq.trim_polyg_tails` artifacts.
///
/// The written summary artifact lives at `runs/bench/local-smoke/fastq.trim_polyg_tails/metrics.json`
/// under the active repository root, alongside the top-level `trimmed.fastq.gz`.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_trim_polyg_tails_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_fastq::stage_api::local_trim_polyg_tails_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "local-smoke fastq.trim_polyg_tails expects exactly one governed case, found {}",
            cases.len()
        ));
    };

    let output_root = repo_root.join("runs/bench/local-smoke/fastq.trim_polyg_tails");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let metrics = materialize_local_trim_polyg_tails_smoke_case(&repo_root, case, &output_root)?;
    let metrics_path = output_root.join("metrics.json");
    bijux_dna_infra::atomic_write_json(&metrics_path, &metrics)?;
    Ok(metrics_path)
}

/// # Errors
/// Returns an error if planning, execution, metric derivation, or persistence fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_trim_polyg_tails<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimPolygArgs,
) -> Result<BenchOutcome<FastqTrimPolygMetrics>> {
    let requested = resolve_requested_tools(&args.tools);
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_TRIM_POLYG_TAILS.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_TRIM_POLYG_TAILS.as_str(), &header);

    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools =
        filter_tools_by_role(STAGE_TRIM_POLYG_TAILS.as_str(), &requested, &registry, false)?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_TRIM_POLYG_TAILS,
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

    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_TRIM_POLYG_TAILS.as_str());
    let all_tools: Vec<String> =
        registry.tools_for_stage(&stage_id).iter().map(|tool| tool.tool_id.to_string()).collect();
    let excluded: Vec<String> =
        all_tools.into_iter().filter(|tool| !tools.contains(tool)).collect();

    if args.explain {
        write_explain_md(
            &bench_inputs.bench_dir,
            STAGE_TRIM_POLYG_TAILS.as_str(),
            &tools,
            &excluded,
            None,
        )?;
        write_explain_plan_json(
            &bench_inputs.bench_dir,
            STAGE_TRIM_POLYG_TAILS.as_str(),
            &tools,
            &registry,
            None,
        )?;
    }

    ensure_image_qa_passed(STAGE_TRIM_POLYG_TAILS.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_TRIM_POLYG_TAILS.as_str(), &tools, platform, catalog)?;

    let polyx_context = polyx_bank_context(args.polyx_preset.as_deref())?;
    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut records = Vec::<BenchmarkRecord<FastqTrimPolygMetrics>>::new();
    let mut failures = Vec::<RawFailure>::new();

    for tool in tools {
        let out_dir = bench_inputs.tools_root.join(&tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec = build_tool_execution_spec(
            STAGE_TRIM_POLYG_TAILS.as_str(),
            &tool,
            &registry,
            catalog,
            platform,
        )?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = bijux_dna_planner_fastq::stage_api::fastq::trim_polyg_tails::plan_trim_polyg_tails_with_options(
            &tool_spec,
            &bench_inputs.r1,
            args.r2.as_deref(),
            &out_dir,
            &bijux_dna_planner_fastq::stage_api::fastq::trim_polyg_tails::TrimPolygPlanOptions {
                threads: args.threads,
                trim_polyg: args.trim_polyg.unwrap_or(true),
                min_polyg_run: args.min_polyg_run.unwrap_or(10),
            },
        )?;
        let bench_params =
            benchmark_query_context(polyx_context.as_ref())?.embed_in_parameters(&plan.params);
        let params_hash = stable_params_hash(&bench_params);
        let image_digest = benchmark_image_identity(&tool_spec);
        if let Ok(Some(record)) = fetch_fastq_trim_polyg_v1(
            &conn,
            &tool,
            &tool_spec.tool_version,
            &image_digest,
            &bench_inputs.runner.to_string(),
            &platform.name,
            &input_hash,
            &params_hash,
        ) {
            records.push(record);
            continue;
        }

        let execution = execute_plans_with_jobs(
            vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&plan)],
            bench_inputs.runner,
            jobs,
        )?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing execution result for {tool}"))?;

        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: STAGE_TRIM_POLYG_TAILS.as_str().to_string(),
                tool: tool.clone(),
                reason: format!("tool `{tool}` failed with status {}", execution.exit_code),
                category: ErrorCategory::ToolError,
            });
            continue;
        }

        let output_r1 = plan.io.outputs[0].path.clone();
        let output_stats_r1 =
            observe_fastq_stats(catalog, platform, bench_inputs.runner, &output_r1)?;
        let output_stats_r2 = if args.r2.is_some() {
            Some(observe_fastq_stats(
                catalog,
                platform,
                bench_inputs.runner,
                &plan.io.outputs[1].path,
            )?)
        } else {
            None
        };
        let before_stats =
            combine_seqkit_metrics(&bench_inputs.input_stats, input_stats_r2.as_ref());
        let after_stats = combine_seqkit_metrics(&output_stats_r1, output_stats_r2.as_ref());
        let mut trim_polyg_delta =
            derive_trim_polyg_delta_for_path_pair(&bench_inputs.r1, &output_r1)?;
        if let Some(input_r2) = args.r2.as_deref() {
            let output_r2 = required_plan_output_path(&plan, "trimmed_reads_r2")?;
            let mate_delta = derive_trim_polyg_delta_for_path_pair(input_r2, &output_r2)?;
            trim_polyg_delta.trimmed_tail_count += mate_delta.trimmed_tail_count;
            trim_polyg_delta.bases_trimmed_polyg += mate_delta.bases_trimmed_polyg;
        }
        let (raw_report_path, raw_report_format) = raw_polyg_report_artifact(&tool, &out_dir)?;
        let backend_metrics = normalized_polyg_backend_metrics(&raw_report_path, raw_report_format)
            .context("normalize trim polyg backend report")?;
        let report_path = out_dir.join("trim_polyg_tails_report.json");
        let mut governed_report = load_governed_trim_polyg_report(&report_path)?;
        governed_report.reads_in = Some(before_stats.reads);
        governed_report.reads_out = Some(after_stats.reads);
        governed_report.bases_in = Some(before_stats.bases);
        governed_report.bases_out = Some(after_stats.bases);
        governed_report.pairs_in =
            input_stats_r2.as_ref().map(|stats| bench_inputs.input_stats.reads.min(stats.reads));
        governed_report.pairs_out =
            output_stats_r2.as_ref().map(|stats| output_stats_r1.reads.min(stats.reads));
        governed_report.mean_q_before = Some(before_stats.mean_q);
        governed_report.mean_q_after = Some(after_stats.mean_q);
        governed_report.trimmed_tail_count = Some(trim_polyg_delta.trimmed_tail_count);
        governed_report.bases_trimmed_polyg = Some(trim_polyg_delta.bases_trimmed_polyg);
        governed_report.runtime_s = Some(execution.runtime_s);
        governed_report.memory_mb = Some(execution.memory_mb);
        governed_report.raw_backend_report = Some(raw_report_path.display().to_string());
        governed_report.raw_backend_report_format = Some(raw_report_format.to_string());
        governed_report.backend_metrics = Some(backend_metrics.clone());
        governed_report.polyx_bank_id = polyx_context
            .as_ref()
            .and_then(|value| value.get("bank_id"))
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);
        governed_report.polyx_bank_hash = polyx_context
            .as_ref()
            .and_then(|value| value.get("bank_hash"))
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);
        governed_report.polyx_preset = polyx_context
            .as_ref()
            .and_then(|value| value.get("preset"))
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);
        write_governed_trim_polyg_report(&report_path, &governed_report)?;
        let metrics = FastqTrimPolygMetrics {
            reads_in: before_stats.reads,
            reads_out: after_stats.reads,
            bases_in: before_stats.bases,
            bases_out: after_stats.bases,
            pairs_in: input_stats_r2
                .as_ref()
                .map(|stats| bench_inputs.input_stats.reads.min(stats.reads)),
            pairs_out: output_stats_r2.as_ref().map(|stats| output_stats_r1.reads.min(stats.reads)),
            mean_q_before: before_stats.mean_q,
            mean_q_after: after_stats.mean_q,
            delta_metrics: derive_trim_delta(&before_stats, &after_stats),
            paired_mode: Some(
                match governed_report.paired_mode {
                    bijux_dna_domain_fastq::PairedMode::SingleEnd => "single_end",
                    bijux_dna_domain_fastq::PairedMode::PairedEnd => "paired_end",
                    bijux_dna_domain_fastq::PairedMode::Unknown => "not_declared",
                }
                .to_string(),
            ),
            threads: Some(governed_report.threads),
            trim_polyg: Some(governed_report.trim_polyg),
            min_polyg_run: Some(governed_report.min_polyg_run),
            trimmed_tail_count: governed_report.trimmed_tail_count,
            bases_trimmed_polyg: governed_report.bases_trimmed_polyg,
            raw_backend_report_format: governed_report.raw_backend_report_format.clone(),
            polyx_bank_id: governed_report.polyx_bank_id.clone(),
            polyx_bank_hash: governed_report.polyx_bank_hash.clone(),
            polyx_preset: governed_report.polyx_preset.clone(),
        };
        let metric_set = metric_set(metrics.clone());
        bijux_dna_analyze::validate_metric_set(&metric_set)?;
        let metrics_json = serde_json::to_value(&metric_set)?;
        bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
            .context("write trim polyg metrics")?;

        let context = build_benchmark_context(
            &tool,
            tool_spec.tool_version.clone(),
            image_digest,
            bench_inputs.runner,
            platform,
            input_hash.clone(),
            bench_params.clone(),
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
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_trim_polyg_v1(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome { records, failures, bench_dir: bench_inputs.bench_dir, explain: args.explain })
}

fn combine_seqkit_metrics(
    primary: &SeqkitMetrics,
    secondary: Option<&SeqkitMetrics>,
) -> SeqkitMetrics {
    let secondary_reads = secondary.map_or(0, |stats| stats.reads);
    let secondary_bases = secondary.map_or(0, |stats| stats.bases);
    let total_bases = primary.bases + secondary_bases;
    let weighted_mean_q = if total_bases == 0 {
        0.0
    } else {
        ((primary.mean_q * u64_to_f64(primary.bases))
            + secondary.map_or(0.0, |stats| stats.mean_q * u64_to_f64(stats.bases)))
            / u64_to_f64(total_bases)
    };
    let weighted_gc = if total_bases == 0 {
        0.0
    } else {
        ((primary.gc_percent * u64_to_f64(primary.bases))
            + secondary.map_or(0.0, |stats| stats.gc_percent * u64_to_f64(stats.bases)))
            / u64_to_f64(total_bases)
    };
    SeqkitMetrics {
        reads: primary.reads + secondary_reads,
        bases: total_bases,
        mean_q: weighted_mean_q,
        gc_percent: weighted_gc,
    }
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

#[allow(clippy::too_many_lines)]
fn materialize_local_trim_polyg_tails_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_fastq::LocalTrimPolygTailsSmokeCasePlan,
    output_root: &Path,
) -> Result<LocalTrimPolygTailsSmokeMetrics> {
    let effective_params =
        serde_json::from_value::<TrimPolygTailsParams>(case.plan.effective_params.clone())
            .context("decode trim polyG local-smoke effective params")?;
    let input_r1 = repo_root.join(&case.r1);
    let output_r1 =
        resolve_output_path(repo_root, &required_plan_output_path(&case.plan, "trimmed_reads_r1")?);
    let report_path =
        resolve_output_path(repo_root, &required_plan_output_path(&case.plan, "report_json")?);
    let raw_backend_report = resolve_output_path(
        repo_root,
        &optional_plan_output_path(&case.plan, "raw_backend_report_json")
            .or_else(|| optional_plan_output_path(&case.plan, "raw_backend_report_txt"))
            .ok_or_else(|| {
                anyhow!(
                    "trim_polyg_tails plan is missing governed raw backend report output for tool {}",
                    case.plan.tool_id.as_str()
                )
            })?,
    );

    for path in [&output_r1, &report_path, &raw_backend_report] {
        if let Some(parent) = path.parent() {
            bijux_dna_infra::ensure_dir(parent)?;
        }
    }

    let input_records = read_fastq_records(&input_r1)?;
    let mut trimmed_tail_count = 0_u64;
    let mut bases_trimmed_polyg = 0_u64;
    let trimmed_records = input_records
        .iter()
        .map(|record| {
            let (trimmed, removed_bases, trimmed_tail) = trim_polyg_record_locally(
                record,
                effective_params.trim_polyg,
                effective_params.min_polyg_run as usize,
            );
            trimmed_tail_count += u64::from(trimmed_tail);
            bases_trimmed_polyg += removed_bases;
            trimmed
        })
        .collect::<Vec<_>>();

    write_fastq_records(&output_r1, &trimmed_records)?;

    let top_level_trimmed = output_root.join("trimmed.fastq.gz");
    write_fastq_records(&top_level_trimmed, &trimmed_records)?;

    let input_reads = input_records.len() as u64;
    let output_reads = trimmed_records.len() as u64;
    let reads_retained = output_reads;
    let reads_dropped = input_reads.saturating_sub(output_reads);
    let input_bases = total_bases(&input_records);
    let output_bases = total_bases(&trimmed_records);
    let bases_removed = input_bases.saturating_sub(output_bases);
    let raw_backend_report_format =
        if case.plan.tool_id.as_str() == "fastp" { "fastp_json" } else { "bbduk_stats" };

    let report = TrimPolygReportV1 {
        schema_version: bijux_dna_domain_fastq::TRIM_POLYG_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_TRIM_POLYG_TAILS.as_str().to_string(),
        stage_id: STAGE_TRIM_POLYG_TAILS.as_str().to_string(),
        tool_id: case.plan.tool_id.as_str().to_string(),
        paired_mode: effective_params.paired_mode,
        threads: effective_params.threads,
        trim_polyg: effective_params.trim_polyg,
        min_polyg_run: effective_params.min_polyg_run,
        input_r1: case.r1.display().to_string(),
        input_r2: case.r2.as_ref().map(|path| path.display().to_string()),
        output_r1: path_relative_to_repo(repo_root, &output_r1),
        output_r2: None,
        reads_in: Some(input_reads),
        reads_out: Some(output_reads),
        bases_in: Some(input_bases),
        bases_out: Some(output_bases),
        pairs_in: None,
        pairs_out: None,
        mean_q_before: mean_quality(&input_records),
        mean_q_after: mean_quality(&trimmed_records),
        trimmed_tail_count: Some(trimmed_tail_count),
        bases_trimmed_polyg: Some(bases_trimmed_polyg),
        polyx_bank_id: None,
        polyx_bank_hash: None,
        polyx_preset: None,
        runtime_s: None,
        memory_mb: None,
        raw_backend_report: Some(path_relative_to_repo(repo_root, &raw_backend_report)),
        raw_backend_report_format: Some(raw_backend_report_format.to_string()),
        backend_metrics: Some(serde_json::json!({
            "local_smoke": true,
            "trimmed_tail_count": trimmed_tail_count,
            "bases_trimmed_polyg": bases_trimmed_polyg,
            "smoke_materialization": "repo_harness",
        })),
    };
    write_governed_trim_polyg_report(&report_path, &report)?;
    write_local_polyg_backend_report(
        &raw_backend_report,
        case.plan.tool_id.as_str(),
        input_reads,
        output_reads,
        trimmed_tail_count,
        bases_trimmed_polyg,
    )?;

    Ok(LocalTrimPolygTailsSmokeMetrics {
        schema_version: LOCAL_TRIM_POLYG_TAILS_SMOKE_METRICS_SCHEMA_VERSION.to_string(),
        stage_id: STAGE_TRIM_POLYG_TAILS.as_str().to_string(),
        sample_id: case.sample_id.clone(),
        tool_id: case.plan.tool_id.as_str().to_string(),
        trim_polyg: effective_params.trim_polyg,
        min_polyg_run: effective_params.min_polyg_run,
        input_reads,
        output_reads,
        reads_retained,
        reads_dropped,
        input_bases,
        output_bases,
        bases_removed,
        trimmed_tail_count,
        bases_trimmed_polyg,
        trimmed_fastq_gz: path_relative_to_repo(repo_root, &top_level_trimmed),
        report_json: path_relative_to_repo(repo_root, &report_path),
        raw_backend_report: path_relative_to_repo(repo_root, &raw_backend_report),
        used_fallback: true,
    })
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

fn total_bases(records: &[FastqRecord]) -> u64 {
    records.iter().map(|record| record.sequence.len() as u64).sum()
}

fn mean_quality(records: &[FastqRecord]) -> Option<f64> {
    let total_bases = total_bases(records);
    if total_bases == 0 {
        return None;
    }
    let total_quality = records
        .iter()
        .flat_map(|record| record.quality.bytes())
        .map(|value| u64::from(value.saturating_sub(33)))
        .sum::<u64>();
    Some(u64_to_f64(total_quality) / u64_to_f64(total_bases))
}

fn trim_polyg_record_locally(
    record: &FastqRecord,
    trim_polyg: bool,
    min_polyg_run: usize,
) -> (FastqRecord, u64, bool) {
    if !trim_polyg {
        return (record.clone(), 0, false);
    }
    let trailing_g_run =
        record.sequence.as_bytes().iter().rev().take_while(|base| **base == b'G').count();
    if trailing_g_run < min_polyg_run {
        return (record.clone(), 0, false);
    }
    let retained_len = record.sequence.len().saturating_sub(trailing_g_run);
    (
        FastqRecord {
            header: record.header.clone(),
            sequence: record.sequence[..retained_len].to_string(),
            plus: record.plus.clone(),
            quality: record.quality[..retained_len].to_string(),
        },
        trailing_g_run as u64,
        true,
    )
}

fn read_fastq_records(path: &Path) -> Result<Vec<FastqRecord>> {
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
        records.push(FastqRecord { header, sequence, plus, quality });
    }
    Ok(records)
}

fn write_fastq_records(path: &Path, records: &[FastqRecord]) -> Result<()> {
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

fn write_local_polyg_backend_report(
    path: &Path,
    tool_id: &str,
    input_reads: u64,
    output_reads: u64,
    trimmed_tail_count: u64,
    bases_trimmed_polyg: u64,
) -> Result<()> {
    match tool_id {
        "fastp" => bijux_dna_infra::write_bytes(
            path,
            serde_json::json!({
                "filtering_result": {
                    "passed_filter_reads": output_reads,
                    "low_quality_reads": 0_u64,
                    "too_many_N_reads": 0_u64,
                    "too_short_reads": 0_u64,
                },
                "poly_g_trimming": {
                    "input_reads": input_reads,
                    "trimmed_reads": trimmed_tail_count,
                    "trimmed_bases": bases_trimmed_polyg,
                }
            })
            .to_string(),
        )
        .with_context(|| format!("write local trim polyG backend report {}", path.display())),
        "bbduk" => bijux_dna_infra::write_bytes(path, format!("Reads Removed: {trimmed_tail_count}\n"))
            .with_context(|| format!("write local trim polyG backend report {}", path.display())),
        _ => Err(anyhow!(
            "local-smoke fastq.trim_polyg_tails does not support backend report materialization for tool `{tool_id}`"
        )),
    }
}

fn required_plan_output_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    output_id: &str,
) -> Result<PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| {
            anyhow!(
                "trim_polyg_tails plan is missing governed output `{output_id}` for tool {}",
                plan.tool_id.as_str()
            )
        })
}

fn optional_plan_output_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    output_id: &str,
) -> Option<PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
}

fn benchmark_query_context(
    polyx_context: Option<&serde_json::Value>,
) -> Result<bijux_dna_domain_fastq::BenchQueryContext> {
    let mut context = bijux_dna_domain_fastq::governed_stage_bench_query_context(
        STAGE_TRIM_POLYG_TAILS.as_str(),
    )?;
    if let Some(bank_hash) =
        polyx_context.and_then(|value| value.get("bank_hash")).and_then(serde_json::Value::as_str)
    {
        context = context.with_bank_hash("polyx_bank", bank_hash.to_string());
    }
    Ok(context)
}

fn raw_polyg_report_artifact(tool_id: &str, out_dir: &Path) -> Result<(PathBuf, &'static str)> {
    match tool_id {
        "fastp" => Ok((out_dir.join("trim_polyg_tails_report.fastp.json"), "fastp_json")),
        "bbduk" => Ok((out_dir.join("trim_polyg_tails_report.stats.txt"), "bbduk_stats")),
        _ => Err(anyhow!("unsupported trim_polyg_tails raw report artifact for tool {tool_id}")),
    }
}

fn normalized_polyg_backend_metrics(
    raw_report_path: &Path,
    raw_report_format: &str,
) -> Result<serde_json::Value> {
    let raw_backend_report =
        std::fs::read_to_string(raw_report_path).context("read trim polyg backend report")?;
    match raw_report_format {
        "fastp_json" => {
            let metrics = parse_fastp_metrics(&raw_backend_report)
                .context("parse fastp polyg backend metrics")?;
            Ok(serde_json::to_value(metrics).context("serialize fastp polyg backend metrics")?)
        }
        "bbduk_stats" => {
            let reads_removed = parse_bbduk_reads_removed(&raw_backend_report)
                .context("parse bbduk polyg backend metrics")?;
            Ok(serde_json::json!({
                "schema_version": "bijux.bbduk.trim_polyg.metrics.v1",
                "reads_removed": reads_removed,
            }))
        }
        _ => Err(anyhow!("unsupported trim_polyg_tails raw report format {raw_report_format}")),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::{
        admitted_stage_tools, benchmark_query_context, derive_trim_polyg_delta,
        load_governed_trim_polyg_report, normalized_polyg_backend_metrics,
        raw_polyg_report_artifact, resolve_requested_tools, write_governed_trim_polyg_report,
        FastqRecord,
    };
    use bijux_dna_domain_fastq::params::PairedMode;
    use bijux_dna_domain_fastq::{TrimPolygReportV1, TRIM_POLYG_REPORT_SCHEMA_VERSION};
    use std::path::Path;

    #[test]
    fn resolve_requested_tools_uses_execution_support_for_auto_and_all() {
        let expected = admitted_stage_tools();
        assert_eq!(resolve_requested_tools(&[]), expected);
        assert_eq!(resolve_requested_tools(&["auto".to_string()]), expected);
        assert_eq!(resolve_requested_tools(&["all".to_string()]), expected);
    }

    #[test]
    fn benchmark_query_context_keeps_governed_polyg_bank_hash() {
        let polyx_context = serde_json::json!({"bank_hash": "polyx-hash"});
        let context = benchmark_query_context(Some(&polyx_context)).expect("query context");

        assert!(context.stage_contract_hash.is_some());
        assert_eq!(context.bank_hashes.get("polyx_bank").map(String::as_str), Some("polyx-hash"));
    }

    #[test]
    fn raw_polyg_report_artifact_uses_backend_specific_native_outputs() {
        let out_dir = Path::new("out");

        let (fastp_path, fastp_format) =
            raw_polyg_report_artifact("fastp", out_dir).expect("fastp raw report");
        assert_eq!(fastp_path, Path::new("out").join("trim_polyg_tails_report.fastp.json"));
        assert_eq!(fastp_format, "fastp_json");

        let (bbduk_path, bbduk_format) =
            raw_polyg_report_artifact("bbduk", out_dir).expect("bbduk raw report");
        assert_eq!(bbduk_path, Path::new("out").join("trim_polyg_tails_report.stats.txt"));
        assert_eq!(bbduk_format, "bbduk_stats");
    }

    #[test]
    fn normalized_polyg_backend_metrics_parses_fastp_reports() {
        let temp = tempfile::tempdir().expect("tempdir");
        let raw_report_path = temp.path().join("trim_polyg.fastp.json");
        bijux_dna_infra::write_bytes(
            &raw_report_path,
            serde_json::json!({
                "filtering_result": {
                    "passed_filter_reads": 960_u64,
                    "low_quality_reads": 18_u64,
                    "too_many_N_reads": 4_u64,
                    "too_short_reads": 12_u64
                }
            })
            .to_string(),
        )
        .expect("write fastp report");

        let metrics =
            normalized_polyg_backend_metrics(&raw_report_path, "fastp_json").expect("metrics");

        assert_eq!(metrics["passed_filter_reads"], serde_json::json!(960_u64));
        assert_eq!(metrics["too_short_reads"], serde_json::json!(12_u64));
    }

    #[test]
    fn normalized_polyg_backend_metrics_parses_bbduk_reports() {
        let temp = tempfile::tempdir().expect("tempdir");
        let raw_report_path = temp.path().join("trim_polyg.stats.txt");
        bijux_dna_infra::write_bytes(&raw_report_path, "Reads Removed: 137\n")
            .expect("write bbduk report");

        let metrics =
            normalized_polyg_backend_metrics(&raw_report_path, "bbduk_stats").expect("metrics");

        assert_eq!(metrics["reads_removed"], serde_json::json!(137_u64));
    }

    #[test]
    fn derive_trim_polyg_delta_counts_trimmed_records_and_removed_bases() {
        let input = vec![
            FastqRecord {
                header: "@read1".to_string(),
                sequence: "ACGTGGGG".to_string(),
                plus: "+".to_string(),
                quality: "IIIIIIII".to_string(),
            },
            FastqRecord {
                header: "@read2".to_string(),
                sequence: "TTCAA".to_string(),
                plus: "+".to_string(),
                quality: "IIIII".to_string(),
            },
            FastqRecord {
                header: "@read3".to_string(),
                sequence: "GGGACGTACGTGG".to_string(),
                plus: "+".to_string(),
                quality: "IIIIIIIIIIIII".to_string(),
            },
        ];
        let output = vec![
            FastqRecord {
                header: "@read1".to_string(),
                sequence: "ACGT".to_string(),
                plus: "+".to_string(),
                quality: "IIII".to_string(),
            },
            FastqRecord {
                header: "@read2".to_string(),
                sequence: "TTCAA".to_string(),
                plus: "+".to_string(),
                quality: "IIIII".to_string(),
            },
            FastqRecord {
                header: "@read3".to_string(),
                sequence: "GGGACGTACGT".to_string(),
                plus: "+".to_string(),
                quality: "IIIIIIIIIII".to_string(),
            },
        ];

        let delta = derive_trim_polyg_delta(&input, &output).expect("derive delta");
        assert_eq!(delta.trimmed_tail_count, 2);
        assert_eq!(delta.bases_trimmed_polyg, 6);
    }

    #[test]
    fn derive_trim_polyg_delta_rejects_non_polyg_truncation() {
        let input = vec![FastqRecord {
            header: "@read1".to_string(),
            sequence: "ACGTACGT".to_string(),
            plus: "+".to_string(),
            quality: "IIIIIIII".to_string(),
        }];
        let output = vec![FastqRecord {
            header: "@read1".to_string(),
            sequence: "ACGTAC".to_string(),
            plus: "+".to_string(),
            quality: "IIIIII".to_string(),
        }];

        let error = derive_trim_polyg_delta(&input, &output).expect_err("reject non-polyG trim");
        assert!(error.to_string().contains("non-terminal-polyG"));
    }

    #[test]
    fn governed_trim_polyg_report_round_trips_with_backend_metrics() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("trim_polyg_tails_report.json");
        let report = TrimPolygReportV1 {
            schema_version: TRIM_POLYG_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.trim_polyg_tails".to_string(),
            stage_id: "fastq.trim_polyg_tails".to_string(),
            tool_id: "fastp".to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 6,
            trim_polyg: true,
            min_polyg_run: 12,
            input_r1: "reads.fastq.gz".to_string(),
            input_r2: None,
            output_r1: "trimmed.fastq.gz".to_string(),
            output_r2: None,
            reads_in: Some(100),
            reads_out: Some(97),
            bases_in: Some(1000),
            bases_out: Some(910),
            pairs_in: None,
            pairs_out: None,
            mean_q_before: Some(28.0),
            mean_q_after: Some(29.0),
            trimmed_tail_count: Some(3),
            bases_trimmed_polyg: Some(90),
            polyx_bank_id: Some("polyx".to_string()),
            polyx_bank_hash: Some("sha256:polyx".to_string()),
            polyx_preset: Some("illumina_twocolor".to_string()),
            runtime_s: Some(3.5),
            memory_mb: Some(42.0),
            raw_backend_report: Some("trim.fastp.json".to_string()),
            raw_backend_report_format: Some("fastp_json".to_string()),
            backend_metrics: Some(serde_json::json!({
                "schema_version": "bijux.fastp.metrics.v1",
                "passed_filter_reads": 97_u64,
            })),
        };

        write_governed_trim_polyg_report(&report_path, &report).expect("write report");
        let decoded = load_governed_trim_polyg_report(&report_path).expect("load report");

        assert_eq!(decoded.threads, 6);
        assert_eq!(decoded.min_polyg_run, 12);
        assert_eq!(decoded.trimmed_tail_count, Some(3));
        assert_eq!(decoded.bases_trimmed_polyg, Some(90));
        assert_eq!(decoded.polyx_preset.as_deref(), Some("illumina_twocolor"));
        assert_eq!(decoded.raw_backend_report_format.as_deref(), Some("fastp_json"));
        assert_eq!(
            decoded
                .backend_metrics
                .as_ref()
                .and_then(|metrics| metrics.get("passed_filter_reads"))
                .and_then(serde_json::Value::as_u64),
            Some(97)
        );
    }
}
