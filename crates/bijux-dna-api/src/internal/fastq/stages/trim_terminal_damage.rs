use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::query_shared::{
    fetch_fastq_trim_terminal_damage_v1, insert_fastq_trim_terminal_damage_v1,
};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkRecord, FastqTrimTerminalDamageMetrics,
};
use bijux_dna_core::ids::StageId;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_domain_fastq::observer::parse_terminal_damage_report;
use bijux_dna_domain_fastq::params::trim::{
    parse_terminal_damage_execution_policy, terminal_damage_execution_policy_label,
    TrimTerminalDamageParams,
};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::stage_api::fastq::trim_terminal_damage::{
    plan_trim_terminal_damage_with_options, TrimTerminalDamagePlanOptions,
};
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use super::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context, derive_trim_delta, observe_fastq_stats,
    prepare_trim_bench, require_existing_benchmark_output,
};
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_TRIM_TERMINAL_DAMAGE,
};
use bijux_dna_stage_contract::StagePlanV1;
use serde::Serialize;

const LOCAL_TRIM_TERMINAL_DAMAGE_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.fastq.trim_terminal_damage.local_smoke.metrics.v1";

#[derive(Debug, Clone, Serialize)]
struct LocalTrimTerminalDamageSmokeMetrics {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    tool_id: String,
    damage_mode: String,
    execution_policy: String,
    input_reads: u64,
    output_reads: u64,
    input_bases: u64,
    output_bases: u64,
    trim_5p_bases: u32,
    trim_3p_bases: u32,
    bases_removed: u64,
    trimmed_fastq_gz: String,
    report_json: String,
    used_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalFastqRecord {
    header: String,
    sequence: String,
    plus: String,
    quality: String,
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
        STAGE_TRIM_TERMINAL_DAMAGE.as_str(),
    ))
    .into_iter()
    .map(|tool_id| tool_id.to_string())
    .collect()
}

/// Materialize the governed local-smoke `fastq.trim_terminal_damage` artifacts.
///
/// The written summary artifact lives at `target/local-smoke/fastq.trim_terminal_damage/metrics.json`
/// under the active repository root, alongside the top-level `trimmed.fastq.gz`.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_trim_terminal_damage_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases =
        bijux_dna_planner_fastq::stage_api::local_trim_terminal_damage_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "local-smoke fastq.trim_terminal_damage expects exactly one governed case, found {}",
            cases.len()
        ));
    };

    let output_root = repo_root.join("target/local-smoke/fastq.trim_terminal_damage");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let metrics =
        materialize_local_trim_terminal_damage_smoke_case(&repo_root, case, &output_root)?;
    let metrics_path = output_root.join("metrics.json");
    bijux_dna_infra::atomic_write_json(&metrics_path, &metrics)?;
    Ok(metrics_path)
}

/// # Errors
/// Returns an error if planning, execution, metric derivation, or persistence fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_trim_terminal_damage<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimTerminalDamageArgs,
) -> Result<BenchOutcome<FastqTrimTerminalDamageMetrics>> {
    let requested = resolve_requested_tools(&args.tools);
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_TRIM_TERMINAL_DAMAGE.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_TRIM_TERMINAL_DAMAGE.as_str(), &header);

    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools =
        filter_tools_by_role(STAGE_TRIM_TERMINAL_DAMAGE.as_str(), &requested, &registry, false)?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_TRIM_TERMINAL_DAMAGE,
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

    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_TRIM_TERMINAL_DAMAGE.as_str());
    let all_tools: Vec<String> =
        registry.tools_for_stage(&stage_id).iter().map(|tool| tool.tool_id.to_string()).collect();
    let excluded: Vec<String> =
        all_tools.into_iter().filter(|tool| !tools.contains(tool)).collect();

    if args.explain {
        write_explain_md(
            &bench_inputs.bench_dir,
            STAGE_TRIM_TERMINAL_DAMAGE.as_str(),
            &tools,
            &excluded,
            None,
        )?;
        write_explain_plan_json(
            &bench_inputs.bench_dir,
            STAGE_TRIM_TERMINAL_DAMAGE.as_str(),
            &tools,
            &registry,
            None,
        )?;
    }

    ensure_image_qa_passed(STAGE_TRIM_TERMINAL_DAMAGE.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_TRIM_TERMINAL_DAMAGE.as_str(), &tools, platform, catalog)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut records = Vec::<BenchmarkRecord<FastqTrimTerminalDamageMetrics>>::new();
    let mut failures = Vec::<RawFailure>::new();

    for tool in tools {
        let out_dir = bench_inputs.tools_root.join(&tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec = build_tool_execution_spec(
            STAGE_TRIM_TERMINAL_DAMAGE.as_str(),
            &tool,
            &registry,
            catalog,
            platform,
        )?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let damage_mode = args
            .damage_mode
            .as_deref()
            .unwrap_or("ancient")
            .parse()
            .map_err(|err: String| anyhow!(err))
            .with_context(|| {
                format!(
                    "parse fastq.trim_terminal_damage damage_mode `{}`",
                    args.damage_mode.as_deref().unwrap_or("ancient")
                )
            })?;
        let execution_policy = parse_requested_execution_policy(args.execution_policy.as_deref())?;
        let plan = plan_trim_terminal_damage_with_options(
            &tool_spec,
            &bench_inputs.r1,
            args.r2.as_deref(),
            &out_dir,
            &TrimTerminalDamagePlanOptions {
                threads: args.threads,
                damage_mode,
                execution_policy,
                trim_5p_bases: args.trim_5p_bases.unwrap_or(2),
                trim_3p_bases: args.trim_3p_bases.unwrap_or(2),
            },
        )?;
        let bench_params = benchmark_query_context()?.embed_in_parameters(&plan.params);
        let params_hash = stable_params_hash(&bench_params);
        let image_digest = benchmark_image_identity(&tool_spec);
        if let Ok(Some(record)) = fetch_fastq_trim_terminal_damage_v1(
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
                stage: STAGE_TRIM_TERMINAL_DAMAGE.as_str().to_string(),
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
        let governed_report = read_governed_terminal_damage_report(&plan)?;
        let metrics = FastqTrimTerminalDamageMetrics {
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
            trim_5p_bases: Some(governed_report.trim_5p_bases),
            trim_3p_bases: Some(governed_report.trim_3p_bases),
            damage_mode: Some(governed_report.damage_mode.as_str().to_string()),
            execution_policy: Some(
                terminal_damage_execution_policy_label(Some(governed_report.execution_policy))
                    .to_string(),
            ),
            requested_trim_5p_bases: governed_report.requested_trim_5p_bases,
            requested_trim_3p_bases: governed_report.requested_trim_3p_bases,
            reads_retained: Some(after_stats.reads),
            bases_removed: Some(before_stats.bases.saturating_sub(after_stats.bases)),
            udg_classification: Some(governed_report.udg_classification.clone()),
            ct_ga_asymmetry_pre: governed_report.ct_ga_asymmetry_pre,
            ct_ga_asymmetry_post: governed_report.ct_ga_asymmetry_post,
            delta_metrics: derive_trim_delta(&before_stats, &after_stats),
        };
        let metric_set = metric_set(metrics.clone());
        bijux_dna_analyze::validate_metric_set(&metric_set)?;
        let effective_params =
            serde_json::from_value::<TrimTerminalDamageParams>(plan.effective_params.clone())
                .context("decode trim terminal damage effective params")?;

        if governed_report.tool_id != tool {
            return Err(anyhow!(
                "terminal damage report drift: expected tool `{tool}`, found `{}`",
                governed_report.tool_id
            ));
        }
        if governed_report.execution_policy != effective_params.execution_policy {
            return Err(anyhow!(
                "terminal damage report drift: expected execution_policy `{:?}`, found `{:?}`",
                effective_params.execution_policy,
                governed_report.execution_policy
            ));
        }
        let metrics_json = serde_json::to_value(&metric_set)?;
        let metrics_path = out_dir.join("metrics.json");
        bijux_dna_infra::atomic_write_json(&metrics_path, &metrics_json)
            .context("write trim terminal damage metrics")?;
        let report_path = required_plan_output_path(&plan, "report_json")?;
        prune_trim_terminal_damage_payload(
            &out_dir,
            &report_path,
            &metrics_path,
            &governed_report,
        )?;

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
        insert_fastq_trim_terminal_damage_v1(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome { records, failures, bench_dir: bench_inputs.bench_dir, explain: args.explain })
}

fn materialize_local_trim_terminal_damage_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_fastq::LocalTrimTerminalDamageSmokeCasePlan,
    output_root: &Path,
) -> Result<LocalTrimTerminalDamageSmokeMetrics> {
    let effective_params =
        serde_json::from_value::<TrimTerminalDamageParams>(case.plan.effective_params.clone())
            .context("decode trim terminal damage local-smoke effective params")?;
    let input_r1 = repo_root.join(&case.r1);
    let output_r1 = resolve_output_path(repo_root, &case.plan.io.outputs[0].path);
    if let Some(parent) = output_r1.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }

    let input_records = read_local_fastq_records(&input_r1)?;
    let trimmed_records = input_records
        .iter()
        .map(|record| {
            trim_local_fastq_record(
                record,
                effective_params.trim_5p_bases as usize,
                effective_params.trim_3p_bases as usize,
            )
        })
        .collect::<Vec<_>>();

    write_local_fastq_records(&output_r1, &trimmed_records)?;

    let top_level_trimmed = output_root.join("trimmed.fastq.gz");
    write_local_fastq_records(&top_level_trimmed, &trimmed_records)?;

    let input_reads = input_records.len() as u64;
    let output_reads = trimmed_records.len() as u64;
    let input_bases = total_bases(&input_records);
    let output_bases = total_bases(&trimmed_records);
    let report_path =
        resolve_output_path(repo_root, &required_plan_output_path(&case.plan, "report_json")?);
    let report = bijux_dna_domain_fastq::TerminalDamageReportV1 {
        schema_version: bijux_dna_domain_fastq::TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_TRIM_TERMINAL_DAMAGE.as_str().to_string(),
        stage_id: STAGE_TRIM_TERMINAL_DAMAGE.as_str().to_string(),
        tool_id: case.plan.tool_id.as_str().to_string(),
        paired_mode: effective_params.paired_mode,
        threads: effective_params.threads,
        damage_mode: effective_params.damage_mode,
        execution_policy: effective_params.execution_policy,
        trim_5p_bases: effective_params.trim_5p_bases,
        trim_3p_bases: effective_params.trim_3p_bases,
        requested_trim_5p_bases: effective_params
            .requested_trim_5p_bases
            .or(Some(effective_params.trim_5p_bases)),
        requested_trim_3p_bases: effective_params
            .requested_trim_3p_bases
            .or(Some(effective_params.trim_3p_bases)),
        udg_classification: match effective_params.damage_mode {
            bijux_dna_domain_fastq::params::DamageMode::Ancient => "non_udg".to_string(),
            bijux_dna_domain_fastq::params::DamageMode::UdgTrimmed => "udg_trimmed".to_string(),
        },
        input_r1: case.r1.display().to_string(),
        input_r2: case.r2.as_ref().map(|path| path.display().to_string()),
        output_r1: path_relative_to_repo(repo_root, &output_r1),
        output_r2: None,
        reads_in: Some(input_reads),
        reads_out: Some(output_reads),
        bases_in: Some(input_bases),
        bases_out: Some(output_bases),
        mean_q_before: mean_quality(&input_records),
        mean_q_after: mean_quality(&trimmed_records),
        ct_ga_asymmetry_pre: None,
        ct_ga_asymmetry_post: None,
        ct_ga_asymmetry_pre_r1: None,
        ct_ga_asymmetry_post_r1: None,
        ct_ga_asymmetry_pre_r2: None,
        ct_ga_asymmetry_post_r2: None,
        terminal_base_composition_pre_r1: None,
        terminal_base_composition_post_r1: None,
        terminal_base_composition_pre_r2: None,
        terminal_base_composition_post_r2: None,
        raw_backend_report: None,
        raw_backend_report_format: None,
        runtime_s: None,
        memory_mb: None,
        used_fallback: true,
        backend_metrics: Some(serde_json::json!({
            "local_smoke": true,
            "bases_removed": input_bases.saturating_sub(output_bases),
            "smoke_materialization": "repo_harness",
        })),
    };
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(LocalTrimTerminalDamageSmokeMetrics {
        schema_version: LOCAL_TRIM_TERMINAL_DAMAGE_SMOKE_METRICS_SCHEMA_VERSION.to_string(),
        stage_id: STAGE_TRIM_TERMINAL_DAMAGE.as_str().to_string(),
        sample_id: case.sample_id.clone(),
        tool_id: case.plan.tool_id.as_str().to_string(),
        damage_mode: effective_params.damage_mode.as_str().to_string(),
        execution_policy: terminal_damage_execution_policy_label(Some(
            effective_params.execution_policy,
        ))
        .to_string(),
        input_reads,
        output_reads,
        input_bases,
        output_bases,
        trim_5p_bases: effective_params.trim_5p_bases,
        trim_3p_bases: effective_params.trim_3p_bases,
        bases_removed: input_bases.saturating_sub(output_bases),
        trimmed_fastq_gz: path_relative_to_repo(repo_root, &top_level_trimmed),
        report_json: path_relative_to_repo(repo_root, &report_path),
        used_fallback: true,
    })
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

fn prune_trim_terminal_damage_payload(
    out_dir: &Path,
    report_path: &Path,
    metrics_path: &Path,
    report: &bijux_dna_domain_fastq::TerminalDamageReportV1,
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
        for entry in fs::read_dir(&dir)
            .with_context(|| format!("read terminal damage tool dir {}", dir.display()))?
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
                .with_context(|| format!("prune terminal damage payload {}", path.display()))?;
        }
    }

    Ok(())
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
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

fn total_bases(records: &[LocalFastqRecord]) -> u64 {
    records.iter().map(|record| record.sequence.len() as u64).sum()
}

fn mean_quality(records: &[LocalFastqRecord]) -> Option<f64> {
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

fn trim_local_fastq_record(
    record: &LocalFastqRecord,
    trim_5p_bases: usize,
    trim_3p_bases: usize,
) -> LocalFastqRecord {
    let len = record.sequence.len();
    let start = trim_5p_bases.min(len);
    let retained = len.saturating_sub(start);
    let end = len.saturating_sub(trim_3p_bases.min(retained));
    LocalFastqRecord {
        header: record.header.clone(),
        sequence: record.sequence[start..end].to_string(),
        plus: record.plus.clone(),
        quality: record.quality[start..end].to_string(),
    }
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

fn benchmark_query_context() -> Result<bijux_dna_domain_fastq::BenchQueryContext> {
    bijux_dna_domain_fastq::governed_stage_bench_query_context(STAGE_TRIM_TERMINAL_DAMAGE.as_str())
}

fn parse_requested_execution_policy(
    value: Option<&str>,
) -> Result<Option<bijux_dna_domain_fastq::params::trim::TerminalDamageExecutionPolicy>> {
    let Some(value) = value else {
        return Ok(None);
    };
    parse_terminal_damage_execution_policy(value).ok_or_else(|| {
        anyhow!(
            "invalid fastq.trim_terminal_damage execution_policy `{value}`: expected policy_derived, explicit_terminal_trim, or preserve_udg_trimmed_ends"
        )
    })
}

fn required_plan_output_path(plan: &StagePlanV1, output_id: &str) -> Result<std::path::PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| {
            anyhow!(
                "trim_terminal_damage plan is missing governed output `{output_id}` for tool {}",
                plan.tool_id.as_str()
            )
        })
}

fn read_governed_terminal_damage_report(
    plan: &StagePlanV1,
) -> Result<bijux_dna_domain_fastq::TerminalDamageReportV1> {
    let report_path = required_plan_output_path(plan, "report_json")?;
    let report_path = require_existing_benchmark_output(&report_path, "report_json")?;
    let report = std::fs::read_to_string(report_path)
        .with_context(|| format!("read terminal damage report {}", report_path.display()))?;
    parse_terminal_damage_report(&report).context("parse governed terminal damage report")
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        admitted_stage_tools, parse_requested_execution_policy, prune_trim_terminal_damage_payload,
        read_governed_terminal_damage_report, required_plan_output_path, resolve_requested_tools,
    };
    use bijux_dna_core::contract::{ArtifactRole, StageIO, ToolConstraints};
    use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, ToolId};
    use bijux_dna_core::prelude::{ArtifactRef, CommandSpecV1, ContainerImageRefV1};
    use bijux_dna_domain_fastq::params::{
        trim::parse_terminal_damage_execution_policy, DamageMode,
    };
    use bijux_dna_stage_contract::{PlanDecisionReason, StagePlanV1};
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn resolve_requested_tools_uses_execution_support_for_auto_and_all() {
        let expected = admitted_stage_tools();
        assert_eq!(resolve_requested_tools(&[]), expected);
        assert_eq!(resolve_requested_tools(&["auto".to_string()]), expected);
        assert_eq!(resolve_requested_tools(&["all".to_string()]), expected);
    }

    #[test]
    fn terminal_damage_execution_policy_parser_accepts_policy_derived() {
        assert!(parse_requested_execution_policy(None)
            .unwrap_or_else(|err| panic!("default policy: {err}"))
            .is_none());
        assert!(parse_requested_execution_policy(Some("policy_derived"))
            .unwrap_or_else(|err| panic!("policy_derived: {err}"))
            .is_none());
    }

    #[test]
    fn terminal_damage_execution_policy_parser_rejects_unknown_policy() {
        let error = match parse_requested_execution_policy(Some("trim_whatever")) {
            Ok(value) => panic!("unknown policy must fail: {value:?}"),
            Err(err) => err,
        };
        assert!(error.to_string().contains("invalid fastq.trim_terminal_damage execution_policy"));
    }

    #[test]
    fn trim_terminal_damage_report_path_follows_governed_outputs() {
        let plan = StagePlanV1 {
            stage_id: StageId::from_static("fastq.trim_terminal_damage"),
            stage_instance_id: None,
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("cutadapt"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
            command: CommandSpecV1 { template: vec!["cutadapt".to_string()] },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: Vec::new(),
                outputs: vec![ArtifactRef::required(
                    ArtifactId::from_static("report_json"),
                    PathBuf::from("custom/trim_terminal_damage_report.json"),
                    ArtifactRole::ReportJson,
                )],
            },
            out_dir: PathBuf::from("custom"),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            aux_images: BTreeMap::new(),
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
            reason: PlanDecisionReason::default(),
        };

        assert_eq!(
            required_plan_output_path(&plan, "report_json")
                .unwrap_or_else(|err| panic!("report path: {err}")),
            PathBuf::from("custom/trim_terminal_damage_report.json")
        );
    }

    #[test]
    fn read_governed_terminal_damage_report_uses_governed_contract() {
        let temp = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let report_path = temp.path().join("trim_terminal_damage_report.json");
        bijux_dna_infra::write_bytes(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.trim_terminal_damage.report.v2",
                "stage": "fastq.trim_terminal_damage",
                "stage_id": "fastq.trim_terminal_damage",
                "tool_id": "cutadapt",
                "paired_mode": "single_end",
                "threads": 1,
                "damage_mode": "ancient",
                "execution_policy": "explicit_terminal_trim",
                "trim_5p_bases": 2,
                "trim_3p_bases": 1,
                "requested_trim_5p_bases": 2,
                "requested_trim_3p_bases": 1,
                "udg_classification": "non_udg",
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "trimmed.fastq.gz",
                "output_r2": null,
                "reads_in": null,
                "reads_out": null,
                "bases_in": null,
                "bases_out": null,
                "mean_q_before": null,
                "mean_q_after": null,
                "ct_ga_asymmetry_pre": null,
                "ct_ga_asymmetry_post": null,
                "ct_ga_asymmetry_pre_r1": null,
                "ct_ga_asymmetry_post_r1": null,
                "ct_ga_asymmetry_pre_r2": null,
                "ct_ga_asymmetry_post_r2": null,
                "terminal_base_composition_pre_r1": null,
                "terminal_base_composition_post_r1": null,
                "terminal_base_composition_pre_r2": null,
                "terminal_base_composition_post_r2": null,
                "raw_backend_report": "cutadapt.damage.json",
                "raw_backend_report_format": "cutadapt_json",
                "runtime_s": null,
                "memory_mb": null,
                "used_fallback": false,
                "backend_metrics": null
            })
            .to_string(),
        )
        .unwrap_or_else(|err| panic!("write report: {err}"));

        let plan = StagePlanV1 {
            stage_id: StageId::from_static("fastq.trim_terminal_damage"),
            stage_instance_id: None,
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("cutadapt"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
            command: CommandSpecV1 { template: vec!["cutadapt".to_string()] },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: Vec::new(),
                outputs: vec![ArtifactRef::required(
                    ArtifactId::from_static("report_json"),
                    report_path,
                    ArtifactRole::ReportJson,
                )],
            },
            out_dir: temp.path().to_path_buf(),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            aux_images: BTreeMap::new(),
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
            reason: PlanDecisionReason::default(),
        };

        let report = read_governed_terminal_damage_report(&plan)
            .unwrap_or_else(|err| panic!("governed report: {err}"));
        assert_eq!(report.tool_id, "cutadapt");
        assert_eq!(report.raw_backend_report_format.as_deref(), Some("cutadapt_json"));
    }

    #[test]
    fn prune_trim_terminal_damage_payload_keeps_reports_and_run_artifacts() {
        let temp = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let out_dir = temp.path().join("cutadapt");
        let run_artifacts = out_dir.join("run_artifacts");
        fs::create_dir_all(&run_artifacts).expect("mkdir");

        let report_path = out_dir.join("trim_terminal_damage_report.json");
        let metrics_path = out_dir.join("metrics.json");
        let raw_backend_report = out_dir.join("trim_terminal_damage.cutadapt.raw.json");
        let trimmed_r1 = out_dir.join("R1.trimmed.fastq.gz");
        let trimmed_r2 = out_dir.join("R2.trimmed.fastq.gz");
        let stage_report = run_artifacts.join("stage_report.json");

        fs::write(&report_path, "{}").expect("write report");
        fs::write(&metrics_path, "{}").expect("write metrics");
        fs::write(&raw_backend_report, "{}").expect("write backend report");
        fs::write(&trimmed_r1, "trimmed").expect("write r1");
        fs::write(&trimmed_r2, "trimmed").expect("write r2");
        fs::write(&stage_report, "{}").expect("write run artifact");

        let report = bijux_dna_domain_fastq::TerminalDamageReportV1 {
            schema_version: "bijux.fastq.trim_terminal_damage.report.v2".to_string(),
            stage: "fastq.trim_terminal_damage".to_string(),
            stage_id: "fastq.trim_terminal_damage".to_string(),
            tool_id: "cutadapt".to_string(),
            paired_mode: bijux_dna_domain_fastq::PairedMode::PairedEnd,
            threads: 2,
            damage_mode: DamageMode::Ancient,
            execution_policy: parse_terminal_damage_execution_policy("explicit_terminal_trim")
                .unwrap_or_else(|| panic!("execution policy must parse"))
                .unwrap_or_else(|| panic!("execution policy must be explicit")),
            trim_5p_bases: 2,
            trim_3p_bases: 2,
            requested_trim_5p_bases: Some(2),
            requested_trim_3p_bases: Some(2),
            udg_classification: "non_udg".to_string(),
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            output_r1: trimmed_r1.display().to_string(),
            output_r2: Some(trimmed_r2.display().to_string()),
            reads_in: Some(100),
            reads_out: Some(90),
            bases_in: Some(1000),
            bases_out: Some(900),
            mean_q_before: Some(28.0),
            mean_q_after: Some(30.0),
            ct_ga_asymmetry_pre: Some(0.4),
            ct_ga_asymmetry_post: Some(0.1),
            ct_ga_asymmetry_pre_r1: None,
            ct_ga_asymmetry_post_r1: None,
            ct_ga_asymmetry_pre_r2: None,
            ct_ga_asymmetry_post_r2: None,
            terminal_base_composition_pre_r1: None,
            terminal_base_composition_post_r1: None,
            terminal_base_composition_pre_r2: None,
            terminal_base_composition_post_r2: None,
            raw_backend_report: Some(raw_backend_report.display().to_string()),
            raw_backend_report_format: Some("cutadapt_json".to_string()),
            runtime_s: Some(1.0),
            memory_mb: Some(64.0),
            used_fallback: false,
            backend_metrics: None,
        };

        prune_trim_terminal_damage_payload(&out_dir, &report_path, &metrics_path, &report)
            .unwrap_or_else(|err| panic!("prune payload: {err}"));

        assert!(report_path.is_file());
        assert!(metrics_path.is_file());
        assert!(raw_backend_report.is_file());
        assert!(stage_report.is_file());
        assert!(!trimmed_r1.exists());
        assert!(!trimmed_r2.exists());
    }
}
