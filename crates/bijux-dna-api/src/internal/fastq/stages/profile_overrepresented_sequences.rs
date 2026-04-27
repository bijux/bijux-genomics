use std::collections::{BTreeMap, HashMap};
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_overrepresented_v1, insert_fastq_overrepresented_v1,
};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkRecord, FastqOverrepresentedMetrics, StageMetricSchema,
};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::params_hash;
use bijux_dna_domain_fastq::{
    FastqOverrepresentedProfileParams, OverrepresentedSequenceRowV1, PairedMode,
    ProfileOverrepresentedReportV1, PROFILE_OVERREPRESENTED_REPORT_SCHEMA_VERSION,
};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::profile_overrepresented_sequences::plan_with_options;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use uuid::Uuid;

use crate::internal::fastq::stages::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context,
};
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};

const STAGE_ID: &str = "fastq.profile_overrepresented_sequences";

/// Benchmark FASTQ overrepresented-sequence profiling tools.
///
/// # Errors
/// Returns an error if planning, execution, report parsing, or persistence fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_profile_overrepresented<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileOverrepresentedArgs,
) -> Result<BenchOutcome<FastqOverrepresentedMetrics>> {
    preflight_overrepresented_inputs(args)?;

    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = bijux_dna_planner_fastq::select_profile_overrepresented_tools(&args.tools)?;
    let tools = filter_tools_by_role(STAGE_ID, &tools, &registry, false)?;
    let runner = ensure_bench_runner(platform, runner_override)?;

    let bench_dir_name = bench_dir_name(
        &bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_OVERREPRESENTED_SEQUENCES,
    )
    .ok_or_else(|| anyhow!("bench dir missing for {STAGE_ID}"))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_dna_infra::ensure_dir(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, STAGE_ID, &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, STAGE_ID, &tools, &registry, None)?;
    }

    ensure_image_qa_passed(STAGE_ID, &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_ID, &tools, platform, catalog)?;

    let input_hash = overrepresented_input_hash(args)?;
    let sqlite_path = bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::<BenchmarkRecord<FastqOverrepresentedMetrics>>::new();

    for tool in &tools {
        let out_dir = tools_root.join(tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec = build_tool_execution_spec(STAGE_ID, tool, &registry, catalog, platform)?;
        let plan = plan_with_options(
            &tool_spec,
            &args.r1,
            args.r2.as_deref(),
            &out_dir,
            args.threads,
            args.top_k,
        )?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = benchmark_image_identity(&tool_spec);
        if let Ok(Some(record)) = fetch_fastq_overrepresented_v1(
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
        let execution = execute_plans_with_jobs(
            vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&plan)],
            runner,
            jobs,
        )?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing execution result for {tool}"))?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: STAGE_ID.to_string(),
                tool: tool.clone(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
                category: ErrorCategory::ToolError,
            });
            continue;
        }
        let output_tsv = required_output_path(&plan, "overrepresented_sequences_tsv")?;
        let output_json = required_output_path(&plan, "overrepresented_sequences_json")?;
        let report_json = required_output_path(&plan, "report_json")?;
        if !output_tsv.exists() || !output_json.exists() {
            materialize_overrepresented_outputs(
                &args.r1,
                args.r2.as_deref(),
                output_tsv,
                output_json,
                args.top_k.unwrap_or(50).max(1),
            )?;
        }
        let effective_params: FastqOverrepresentedProfileParams =
            serde_json::from_value(plan.effective_params.clone())
                .context("parse overrepresented effective params")?;
        let payload = read_overrepresented_payload(output_json)?;
        let metrics = payload.metrics.clone();
        let metric_set = metric_set(metrics);
        let report = ProfileOverrepresentedReportV1 {
            schema_version: PROFILE_OVERREPRESENTED_REPORT_SCHEMA_VERSION.to_string(),
            stage: STAGE_ID.to_string(),
            stage_id: STAGE_ID.to_string(),
            tool_id: tool.clone(),
            paired_mode: if args.r2.is_some() {
                PairedMode::PairedEnd
            } else {
                PairedMode::SingleEnd
            },
            threads: effective_params.threads,
            top_k: effective_params.top_k,
            input_r1: args.r1.display().to_string(),
            input_r2: args.r2.as_ref().map(|path| path.display().to_string()),
            overrepresented_sequences_tsv: output_tsv.display().to_string(),
            overrepresented_sequences_json: output_json.display().to_string(),
            report_json: report_json.display().to_string(),
            sequence_count: payload.metrics.sequence_count,
            flagged_sequences: payload.metrics.flagged_sequences,
            top_fraction: payload.metrics.top_fraction,
            rows: payload.rows,
            runtime_s: Some(execution.runtime_s),
            memory_mb: Some(execution.memory_mb),
            exit_code: Some(execution.exit_code),
            raw_backend_report: None,
            raw_backend_report_format: None,
        };
        bijux_dna_infra::atomic_write_json(report_json, &report)
            .context("write overrepresented report")?;
        let metrics_json = serde_json::to_value(&metric_set)?;
        bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
            .context("write overrepresented metrics")?;
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
        insert_fastq_overrepresented_v1(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome { records, failures, bench_dir, explain: args.explain })
}

fn preflight_overrepresented_inputs(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileOverrepresentedArgs,
) -> Result<()> {
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_ID, artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_ID, &header);
    Ok(())
}

fn overrepresented_input_hash(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileOverrepresentedArgs,
) -> Result<String> {
    if let Some(r2) = args.r2.as_deref() {
        return Ok(format!(
            "{}+{}",
            hash_file_sha256(&args.r1).context("hash overrepresented input r1")?,
            hash_file_sha256(r2).context("hash overrepresented input r2")?
        ));
    }
    hash_file_sha256(&args.r1).context("hash overrepresented input")
}

fn materialize_overrepresented_outputs(
    input_fastq: &Path,
    input_fastq_r2: Option<&Path>,
    output_tsv: &Path,
    output_json: &Path,
    top_k: u32,
) -> Result<()> {
    let mut counts = BTreeMap::<String, u64>::new();
    for path in std::iter::once(input_fastq).chain(input_fastq_r2.into_iter()) {
        let lines = open_fastq_lines(path)?;
        for (idx, line) in lines.into_iter().enumerate() {
            if idx % 4 == 1 {
                *counts.entry(line.trim().to_string()).or_insert(0) += 1;
            }
        }
    }
    let total: u64 = counts.values().sum();
    let mut ranked = counts.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let top = ranked
        .iter()
        .take(usize::try_from(top_k).unwrap_or(usize::MAX))
        .cloned()
        .collect::<Vec<_>>();
    let top_fraction = if total == 0 {
        0.0
    } else {
        top.first().map_or(0.0, |(_, count)| u64_to_f64(*count) / u64_to_f64(total))
    };
    let flagged_sequences = top
        .iter()
        .filter(|(_, count)| total > 0 && (u64_to_f64(*count) / u64_to_f64(total)) >= 0.01)
        .count();

    let rows = top
        .iter()
        .map(|(sequence, count)| {
            let fraction = if total == 0 { 0.0 } else { u64_to_f64(*count) / u64_to_f64(total) };
            OverrepresentedSequenceRowV1 {
                sequence: sequence.clone(),
                count: *count,
                fraction,
                flag: if fraction >= 0.01 {
                    "overrepresented".to_string()
                } else {
                    "background".to_string()
                },
            }
        })
        .collect::<Vec<_>>();

    let mut tsv = String::from("sequence\tcount\tfraction\tflag\n");
    for row in &rows {
        tsv.push_str(&row.sequence);
        tsv.push('\t');
        tsv.push_str(&row.count.to_string());
        tsv.push('\t');
        let fraction_text = format!("{:.6}", row.fraction);
        tsv.push_str(&fraction_text);
        tsv.push('\t');
        tsv.push_str(&row.flag);
        tsv.push('\n');
    }
    bijux_dna_infra::atomic_write_bytes(output_tsv, tsv.as_bytes())?;
    bijux_dna_infra::atomic_write_json(
        output_json,
        &serde_json::json!({
            "schema_version": "bijux.fastq.profile_overrepresented_sequences.v1",
            "top_k": top_k,
            "sequence_count": usize_to_u64(rows.len()),
            "flagged_sequences": usize_to_u64(flagged_sequences),
            "top_fraction": top_fraction,
            "rows": rows,
        }),
    )?;
    Ok(())
}

#[derive(Debug, Clone)]
struct OverrepresentedPayload {
    metrics: FastqOverrepresentedMetrics,
    rows: Vec<OverrepresentedSequenceRowV1>,
}

fn read_overrepresented_payload(path: &Path) -> Result<OverrepresentedPayload> {
    let value: serde_json::Value = serde_json::from_slice(
        &std::fs::read(path).with_context(|| format!("read {}", path.display()))?,
    )?;
    let rows = value
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            Some(OverrepresentedSequenceRowV1 {
                sequence: entry.get("sequence").and_then(serde_json::Value::as_str)?.to_string(),
                count: entry.get("count").and_then(serde_json::Value::as_u64)?,
                fraction: entry.get("fraction").and_then(serde_json::Value::as_f64).unwrap_or(0.0),
                flag: entry
                    .get("flag")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("background")
                    .to_string(),
            })
        })
        .collect::<Vec<_>>();
    let metrics = FastqOverrepresentedMetrics {
        sequence_count: value
            .get("sequence_count")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_else(|| usize_to_u64(rows.len())),
        flagged_sequences: value
            .get("flagged_sequences")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_else(|| {
                rows.iter()
                    .filter(|row| row.flag == "overrepresented")
                    .count()
                    .try_into()
                    .unwrap_or(u64::MAX)
            }),
        top_fraction: value
            .get("top_fraction")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or_else(|| rows.first().map_or(0.0, |row| row.fraction)),
    };
    metrics.validate()?;
    Ok(OverrepresentedPayload { metrics, rows })
}

fn required_output_path<'a>(
    plan: &'a bijux_dna_stage_contract::StagePlanV1,
    artifact_id: &str,
) -> Result<&'a Path> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_id)
        .map(|artifact| artifact.path.as_path())
        .ok_or_else(|| anyhow!("missing required output artifact `{artifact_id}`"))
}

fn open_fastq_lines(path: &Path) -> Result<Vec<String>> {
    let file =
        std::fs::File::open(path).with_context(|| format!("open fastq {}", path.display()))?;
    if path.extension().and_then(|ext| ext.to_str()) == Some("gz") {
        let decoder = flate2::read::MultiGzDecoder::new(file);
        let reader = BufReader::new(decoder);
        return reader
            .lines()
            .collect::<std::result::Result<Vec<_>, _>>()
            .with_context(|| format!("read gz fastq {}", path.display()));
    }
    let reader = BufReader::new(file);
    reader
        .lines()
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("read fastq {}", path.display()))
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

fn usize_to_u64(value: usize) -> u64 {
    value.try_into().unwrap_or(u64::MAX)
}
