use std::collections::{BTreeSet, HashMap};

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_registry};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{fetch_fastq_infer_asvs_v1, insert_fastq_infer_asvs_v1};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqInferAsvsMetrics};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::params_hash;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::RawFailure;
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use uuid::Uuid;

use crate::internal::fastq::stages::preprocess::{
    enforce_amplicon_qc_thresholds_for_bench, materialize_amplicon_stage_outputs_for_bench,
};
use crate::internal::fastq::stages::trim_bench_common::build_benchmark_context;
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};

const STAGE_ID: &str = "fastq.infer_asvs";

pub fn bench_fastq_infer_asvs<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqInferAsvsArgs,
) -> Result<BenchOutcome<FastqInferAsvsMetrics>> {
    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = bijux_dna_planner_fastq::select_infer_asvs_tools(&args.tools)?;
    let tools = filter_tools_by_role(STAGE_ID, &tools, &registry, false)?;
    let runner = ensure_bench_runner(platform, runner_override)?;
    let input_hash = hash_file_sha256(&args.r1).context("hash infer asvs input")?;
    let bench_dir_name = bench_dir_name(&bijux_dna_domain_fastq::stages::ids::STAGE_INFER_ASVS)
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
        let plan = bijux_dna_planner_fastq::tool_adapters::fastq::infer_asvs::plan(
            &tool_spec,
            &args.r1,
            &out_dir,
        )?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_infer_asvs_v1(
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
            failures.push(RawFailure {
                stage: STAGE_ID.to_string(),
                tool: tool.clone(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
                category: ErrorCategory::ToolError,
            });
            continue;
        }
        let payload = materialize_amplicon_stage_outputs_for_bench(&out_dir, &step)?;
        enforce_amplicon_qc_thresholds_for_bench(&out_dir, STAGE_ID, &payload)?;
        let sample_count = infer_sample_count(&plan.io.outputs[0].path)?;
        let metrics = FastqInferAsvsMetrics {
            asv_count: payload.get("asv_count").and_then(serde_json::Value::as_u64).unwrap_or(0),
            sample_count,
        };
        let metric_set = metric_set(metrics);
        let report = serde_json::json!({
            "schema_version": "bijux.fastq.infer_asvs.report.v1",
            "stage_id": STAGE_ID,
            "tool_id": tool,
            "input_fastq": args.r1,
            "asv_table_tsv": plan.io.outputs[0].path,
            "asv_sequences_fasta": plan.io.outputs[1].path,
            "taxonomy_ready_fasta": plan.io.outputs[2].path,
            "taxonomy_ready_fastq": plan.io.outputs[3].path,
            "runtime_s": execution.runtime_s,
            "memory_mb": execution.memory_mb,
            "exit_code": execution.exit_code,
        });
        bijux_dna_infra::atomic_write_json(&out_dir.join("infer_asvs_report.json"), &report)?;
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
        insert_fastq_infer_asvs_v1(&conn, &record)?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir,
        explain: args.explain,
    })
}

fn infer_sample_count(path: &std::path::Path) -> Result<u64> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut samples = BTreeSet::new();
    for line in raw.lines().skip(1) {
        if let Some((sample_id, _rest)) = line.split_once('\t') {
            if !sample_id.trim().is_empty() {
                samples.insert(sample_id.trim().to_string());
            }
        }
    }
    Ok(samples.len() as u64)
}
