use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_registry};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_read_lengths_v1, insert_fastq_read_lengths_v1,
};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkRecord, FastqReadLengthMetrics, StageMetricSchema,
};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::params_hash;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::RawFailure;
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use uuid::Uuid;

use crate::internal::fastq::stages::trim_bench_common::build_benchmark_context;
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};

const STAGE_ID: &str = "fastq.profile_read_lengths";

pub fn bench_fastq_profile_read_lengths<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqProfileReadLengthsArgs,
) -> Result<BenchOutcome<FastqReadLengthMetrics>> {
    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = bijux_dna_planner_fastq::select_profile_read_lengths_tools(&args.tools)?;
    let tools = filter_tools_by_role(STAGE_ID, &tools, &registry, false)?;
    let runner = ensure_bench_runner(platform, runner_override)?;
    let input_hash = hash_file_sha256(&args.r1).context("hash read-length input")?;

    let bench_dir_name =
        bench_dir_name(&bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_READ_LENGTHS)
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
        let plan = bijux_dna_planner_fastq::tool_adapters::fastq::profile_read_lengths::plan(
            &tool_spec, &args.r1, &out_dir,
        )?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_read_lengths_v1(
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

        let lengths = read_fastq_lengths(&args.r1)?;
        if !plan.io.outputs[0].path.exists() || !plan.io.outputs[1].path.exists() {
            write_length_outputs(&plan.io.outputs[0].path, &plan.io.outputs[1].path, &lengths)?;
        }
        let metrics = metrics_from_lengths(&lengths)?;
        let metric_set = metric_set(metrics);
        let report = serde_json::json!({
            "schema_version": "bijux.fastq.profile_read_lengths.report.v1",
            "stage_id": STAGE_ID,
            "tool_id": tool,
            "input_fastq": args.r1,
            "length_distribution_tsv": plan.io.outputs[0].path,
            "length_distribution_json": plan.io.outputs[1].path,
            "runtime_s": execution.runtime_s,
            "memory_mb": execution.memory_mb,
            "exit_code": execution.exit_code,
        });
        bijux_dna_infra::atomic_write_json(&out_dir.join("profile_read_lengths_report.json"), &report)?;
        bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &serde_json::to_value(&metric_set)?)?;
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
        append_jsonl(&bench_path, &record)?;
        insert_fastq_read_lengths_v1(&conn, &record)?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir,
        explain: args.explain,
    })
}

fn read_fastq_lengths(path: &Path) -> Result<Vec<usize>> {
    let raw = if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
    {
        let output = bijux_dna_runner::command_runner::run_command(
            "gzip",
            &["-cd".to_string(), path.to_string_lossy().into_owned()],
        )
        .with_context(|| format!("gzip -cd {}", path.display()))?;
        if output.exit_code != 0 {
            return Err(anyhow!("failed to decompress {}", path.display()));
        }
        output.stdout
    } else {
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?
    };

    let mut lengths = Vec::new();
    let mut lines = raw.lines();
    loop {
        let header = lines.next();
        let seq = lines.next();
        let plus = lines.next();
        let qual = lines.next();
        let (Some(header), Some(seq), Some(plus), Some(_qual)) = (header, seq, plus, qual) else {
            break;
        };
        if !header.starts_with('@') || !plus.starts_with('+') {
            return Err(anyhow!("invalid FASTQ framing in {}", path.display()));
        }
        lengths.push(seq.len());
    }
    if lengths.is_empty() {
        return Err(anyhow!("no reads detected in {}", path.display()));
    }
    Ok(lengths)
}

fn write_length_outputs(tsv: &Path, json: &Path, lengths: &[usize]) -> Result<()> {
    let mut hist = BTreeMap::<usize, u64>::new();
    for &len in lengths {
        *hist.entry(len).or_insert(0) += 1;
    }
    let mut tsv_body = String::from("sample_id\tread_length\tcount\n");
    for (len, count) in &hist {
        tsv_body.push_str(&format!("sample\t{len}\t{count}\n"));
    }
    bijux_dna_infra::atomic_write_bytes(tsv, tsv_body.as_bytes())?;
    let json_body = serde_json::json!({
        "schema_version": "bijux.fastq.profile_read_lengths.v1",
        "histogram": hist.iter().map(|(len, count)| serde_json::json!({"read_length": len, "count": count})).collect::<Vec<_>>(),
    });
    bijux_dna_infra::atomic_write_json(json, &json_body)?;
    Ok(())
}

fn metrics_from_lengths(lengths: &[usize]) -> Result<FastqReadLengthMetrics> {
    let read_count = lengths.len() as u64;
    let total: usize = lengths.iter().sum();
    let max_read_length = lengths.iter().copied().max().unwrap_or(0) as u64;
    let distinct_lengths = lengths.iter().copied().collect::<BTreeSet<_>>().len() as u64;
    let metrics = FastqReadLengthMetrics {
        read_count,
        mean_read_length: total as f64 / read_count as f64,
        max_read_length,
        distinct_lengths,
    };
    metrics.validate()?;
    Ok(metrics)
}
