use std::collections::{BTreeMap, HashMap};
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_registry};
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
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::profile_overrepresented_sequences::plan;
use bijux_dna_planner_fastq::stage_api::{inspect_headers, log_header_warnings, preflight_stage, FastqArtifact, RawFailure};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use uuid::Uuid;

use crate::internal::fastq::stages::trim_bench_common::build_benchmark_context;
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome,
};

const STAGE_ID: &str = "fastq.profile_overrepresented_sequences";

pub fn bench_fastq_profile_overrepresented<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileOverrepresentedArgs,
) -> Result<BenchOutcome<FastqOverrepresentedMetrics>> {
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage(STAGE_ID, artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings(STAGE_ID, &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = bijux_dna_planner_fastq::select_profile_overrepresented_tools(&args.tools)?;
    let tools = filter_tools_by_role(STAGE_ID, &tools, &registry, false)?;
    let runner = ensure_bench_runner(platform, runner_override)?;

    let bench_dir_name = bench_dir_name(&bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_OVERREPRESENTED_SEQUENCES)
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

    let input_hash = hash_file_sha256(&args.r1).context("hash overrepresented input")?;
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
        let plan = plan(&tool_spec, &args.r1, &out_dir)?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
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
        let output_tsv = &plan.io.outputs[0].path;
        let output_json = &plan.io.outputs[1].path;
        if !output_tsv.exists() || !output_json.exists() {
            materialize_overrepresented_outputs(&args.r1, output_tsv, output_json)?;
        }
        let metrics = read_metrics(output_json)?;
        let metric_set = metric_set(metrics);
        let report = serde_json::json!({
            "schema_version": "bijux.fastq.profile_overrepresented_sequences.report.v1",
            "stage_id": STAGE_ID,
            "tool_id": tool,
            "input_fastq": args.r1,
            "output_tsv": output_tsv,
            "output_json": output_json,
            "runtime_s": execution.runtime_s,
            "memory_mb": execution.memory_mb,
            "exit_code": execution.exit_code,
        });
        bijux_dna_infra::atomic_write_json(&out_dir.join("overrepresented_report.json"), &report)
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

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir,
        explain: args.explain,
    })
}

fn materialize_overrepresented_outputs(
    input_fastq: &Path,
    output_tsv: &Path,
    output_json: &Path,
) -> Result<()> {
    let mut counts = BTreeMap::<String, u64>::new();
    let lines = open_fastq_lines(input_fastq)?;
    for (idx, line) in lines.into_iter().enumerate() {
        if idx % 4 == 1 {
            *counts.entry(line.trim().to_string()).or_insert(0) += 1;
        }
    }
    let total: u64 = counts.values().sum();
    let mut ranked = counts.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let top = ranked.iter().take(50).cloned().collect::<Vec<_>>();
    let top_fraction = if total == 0 {
        0.0
    } else {
        top.first().map_or(0.0, |(_, count)| *count as f64 / total as f64)
    };
    let flagged_sequences = top
        .iter()
        .filter(|(_, count)| total > 0 && (*count as f64 / total as f64) >= 0.01)
        .count() as u64;

    let mut rows = String::from("sequence\tcount\tfraction\tflag\n");
    for (seq, count) in &top {
        let fraction = if total == 0 { 0.0 } else { *count as f64 / total as f64 };
        let flag = if fraction >= 0.01 { "overrepresented" } else { "background" };
        rows.push_str(&format!("{seq}\t{count}\t{fraction:.6}\t{flag}\n"));
    }
    bijux_dna_infra::atomic_write_bytes(output_tsv, rows.as_bytes())?;
    bijux_dna_infra::atomic_write_json(
        output_json,
        &serde_json::json!({
            "schema_version": "bijux.fastq.profile_overrepresented_sequences.v1",
            "sequence_count": top.len(),
            "flagged_sequences": flagged_sequences,
            "top_fraction": top_fraction,
            "rows": top.iter().map(|(sequence, count)| {
                let fraction = if total == 0 { 0.0 } else { *count as f64 / total as f64 };
                serde_json::json!({
                    "sequence": sequence,
                    "count": count,
                    "fraction": fraction,
                    "flag": if fraction >= 0.01 { "overrepresented" } else { "background" },
                })
            }).collect::<Vec<_>>(),
        }),
    )?;
    Ok(())
}

fn read_metrics(path: &Path) -> Result<FastqOverrepresentedMetrics> {
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).with_context(|| format!("read {}", path.display()))?)?;
    let metrics = FastqOverrepresentedMetrics {
        sequence_count: value.get("sequence_count").and_then(serde_json::Value::as_u64).unwrap_or(0),
        flagged_sequences: value.get("flagged_sequences").and_then(serde_json::Value::as_u64).unwrap_or(0),
        top_fraction: value.get("top_fraction").and_then(serde_json::Value::as_f64).unwrap_or(0.0),
    };
    metrics.validate()?;
    Ok(metrics)
}

fn open_fastq_lines(path: &Path) -> Result<Vec<String>> {
    let file = std::fs::File::open(path).with_context(|| format!("open fastq {}", path.display()))?;
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
