use std::collections::HashMap;
use std::path::PathBuf;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_workspace_registry};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::quality::{fetch_fastq_correct_v1, insert_fastq_correct_v1};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkContext, BenchmarkRecord, FastqCorrectMetrics,
};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::params_hash;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::select_correct_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::correct_errors::{
    plan_correct_with_options, parse_quality_encoding, CorrectPlanOptions,
};
use bijux_dna_planner_fastq::stage_api::observer::{input_fastq_stats, parse_seqkit_stats};
use bijux_dna_planner_fastq::stage_api::FastqArtifactKind;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::backend::docker::executor::resolve_image_for_run;
use bijux_dna_runner::step_runner::{execute_observer_command, StageResultV1};
use uuid::Uuid;

use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;

use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_CORRECT_ERRORS,
};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_correct<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqCorrectArgs,
) -> Result<BenchOutcome<bijux_dna_analyze::FastqCorrectMetrics>> {
    let allow_experimental = std::env::var("BIJUX_EXPERIMENTAL_TOOLS").is_ok();
    let tools = select_correct_tools(&args.tools, allow_experimental)?;
    let artifact = FastqArtifactKind::PairedEnd;
    preflight_stage(STAGE_CORRECT_ERRORS.as_str(), artifact)?;
    let r2 = args.r2.as_path();
    let header = inspect_headers(&args.r1, Some(r2), false)?;
    log_header_warnings(STAGE_CORRECT_ERRORS.as_str(), &header);

    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_CORRECT_ERRORS.as_str(), &tools, &registry, false)?;
    let bench_inputs = prepare_correct_bench(catalog, platform, runner_override, args)?;
    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_CORRECT_ERRORS.as_str());
    let all_tools: Vec<String> = registry
        .tools_for_stage(&stage_id)
        .iter()
        .map(|tool| tool.tool_id.to_string())
        .collect();
    let excluded: Vec<String> = all_tools
        .into_iter()
        .filter(|tool| !tools.contains(tool))
        .collect();

    if args.explain {
        write_explain_md(
            &bench_inputs.bench_dir,
            STAGE_CORRECT_ERRORS.as_str(),
            &tools,
            &excluded,
            None,
        )?;
        write_explain_plan_json(
            &bench_inputs.bench_dir,
            STAGE_CORRECT_ERRORS.as_str(),
            &tools,
            &registry,
            None,
        )?;
    }

    ensure_image_qa_passed(STAGE_CORRECT_ERRORS.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_CORRECT_ERRORS.as_str(), &tools, platform, catalog)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::new();
    let mut records = Vec::<BenchmarkRecord<FastqCorrectMetrics>>::new();
    for tool in &tools {
        let out_dir = bench_inputs.tools_root.join(tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec = build_tool_execution_spec(
            STAGE_CORRECT_ERRORS.as_str(),
            tool,
            &registry,
            catalog,
            platform,
        )?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let quality_encoding = args
            .quality_encoding
            .as_deref()
            .map(parse_quality_encoding)
            .transpose()?
            .unwrap_or(
                bijux_dna_domain_fastq::params::correct::QualityEncoding::Phred33,
            );
        let plan = plan_correct_with_options(
            &tool_spec,
            &bench_inputs.r1,
            Some(&bench_inputs.r2),
            &out_dir,
            &CorrectPlanOptions {
                quality_encoding,
                kmer_size: args.kmer_size,
                max_memory_gb: args.max_memory_gb,
                trusted_kmer_artifact: args
                    .trusted_kmer_artifact
                    .as_ref()
                    .map(|path| path.display().to_string()),
                conservative_mode: args.conservative_mode,
            },
        )?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_correct_v1(
            &conn,
            tool,
            &tool_spec.tool_version,
            &image_digest,
            &bench_inputs.runner.to_string(),
            &platform.name,
            &bench_inputs.input_hash,
            &params_hash,
        ) {
            records.push(record);
            continue;
        }
        let execution = execute_plans_with_jobs(
            vec![bijux_dna_stage_contract::execution_step_from_stage_plan(
                &plan,
            )],
            bench_inputs.runner,
            jobs,
        )?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing execution result for {tool}"))?;
        let record = build_correct_record(
            platform,
            &bench_inputs,
            tool,
            &tool_spec,
            &plan.params,
            &out_dir,
            &execution,
        )?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_correct_v1(&conn, &record).context("insert bench sqlite")?;
        if execution.exit_code != 0 {
            let tool_name = tool.clone();
            failures.push(RawFailure {
                stage: STAGE_CORRECT_ERRORS.as_str().to_string(),
                tool: tool.clone(),
                reason: format!(
                    "tool {tool_name} failed with status {}",
                    execution.exit_code
                ),
                category: ErrorCategory::ToolError,
            });
        }
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: bench_inputs.bench_dir,
        explain: args.explain,
    })
}

#[derive(Debug, Clone)]
struct CorrectBenchInputs {
    runner: RuntimeKind,
    r1: PathBuf,
    r2: PathBuf,
    input_hash: String,
    input_stats_r1: SeqkitMetrics,
    bench_dir: PathBuf,
    tools_root: PathBuf,
    seqkit_image: String,
}

fn prepare_correct_bench<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqCorrectArgs,
) -> Result<CorrectBenchInputs> {
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir_name = bench_dir_name(&STAGE_CORRECT_ERRORS)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_CORRECT_ERRORS.as_str()))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_dna_infra::ensure_dir(&tools_root).context("create tools output dir")?;

    let r1 = args.r1.canonicalize().context("resolve r1 path")?;
    let r2 = args.r2.canonicalize().context("resolve r2 path")?;
    let r1_dir = r1
        .parent()
        .ok_or_else(|| anyhow!("r1 has no parent"))?
        .to_path_buf();

    let seqkit_tool = catalog
        .get(bijux_dna_planner_fastq::stage_api::TOOL_SEQKIT)
        .ok_or_else(|| anyhow!("seqkit missing from images catalog"))?;
    let seqkit_image = resolve_image_for_run(seqkit_tool, platform)?;
    let stats_spec = input_fastq_stats(&r1_dir, &r1)?;
    let stats_output = execute_observer_command(
        &seqkit_image.full_name,
        stats_spec.mount_dir.as_path(),
        &stats_spec.args,
        runner,
    )?;
    if stats_output.exit_code != 0 {
        return Err(anyhow!(
            "seqkit correction observer failed: {}",
            stats_output.stderr
        ));
    }

    Ok(CorrectBenchInputs {
        runner,
        r1,
        r2,
        input_hash: hash_file_sha256(&args.r1).context("hash correction input")?,
        input_stats_r1: parse_seqkit_stats(&stats_output.stdout)?,
        bench_dir,
        tools_root,
        seqkit_image: seqkit_image.full_name,
    })
}

fn build_correct_record(
    platform: &PlatformSpec,
    bench_inputs: &CorrectBenchInputs,
    tool: &str,
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    params: &serde_json::Value,
    out_dir: &std::path::Path,
    execution: &StageResultV1,
) -> Result<BenchmarkRecord<FastqCorrectMetrics>> {
    let output_r1 = out_dir.join("reads_r1.fastq.gz");
    let output_stats =
        observe_fastq_stats(&bench_inputs.seqkit_image, bench_inputs.runner, &output_r1)?
            .unwrap_or_else(|| bench_inputs.input_stats_r1.clone());
    let metrics = FastqCorrectMetrics {
        reads_in: bench_inputs.input_stats_r1.reads,
        reads_out: output_stats.reads,
        bases_in: bench_inputs.input_stats_r1.bases,
        bases_out: output_stats.bases,
        pairs_in: Some(bench_inputs.input_stats_r1.reads),
        pairs_out: Some(output_stats.reads),
        mean_q_before: bench_inputs.input_stats_r1.mean_q,
        mean_q_after: output_stats.mean_q,
        kmer_fix_rate: if execution.exit_code == 0 { 1.0 } else { 0.0 },
    };
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    let report = serde_json::json!({
        "schema_version": "bijux.fastq.correct_errors.report.v1",
        "stage": STAGE_CORRECT_ERRORS.as_str(),
        "stage_id": STAGE_CORRECT_ERRORS.as_str(),
        "tool": tool,
        "tool_id": tool,
        "input_r1": bench_inputs.r1,
        "input_r2": bench_inputs.r2,
        "output_r1": output_r1,
        "output_r2": out_dir.join("reads_r2.fastq.gz"),
        "corrected_reads": metrics.reads_out,
        "reads_in": metrics.reads_in,
        "reads_out": metrics.reads_out,
        "bases_in": metrics.bases_in,
        "bases_out": metrics.bases_out,
        "mean_q_before": metrics.mean_q_before,
        "mean_q_after": metrics.mean_q_after,
        "kmer_fix_rate": metrics.kmer_fix_rate,
        "runtime_s": execution.runtime_s,
        "memory_mb": execution.memory_mb,
        "exit_code": execution.exit_code,
    });
    bijux_dna_infra::atomic_write_json(&out_dir.join("correct_report.json"), &report)
        .context("write correction report")?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write correction metrics")?;

    let context = BenchmarkContext {
        tool: tool.to_string(),
        tool_version: tool_spec.tool_version.clone(),
        image_digest: tool_spec
            .image
            .digest
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        runner: bench_inputs.runner.to_string(),
        platform: platform.name.clone(),
        input_hash: bench_inputs.input_hash.clone(),
        parameters: params.clone().into(),
    };
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

fn observe_fastq_stats(
    seqkit_image: &str,
    runner: RuntimeKind,
    reads: &std::path::Path,
) -> Result<Option<SeqkitMetrics>> {
    if !reads.exists() {
        return Ok(None);
    }
    let reads_dir = reads
        .parent()
        .ok_or_else(|| anyhow!("reads path has no parent"))?;
    let stats_spec = input_fastq_stats(reads_dir, reads)?;
    let stats_output = execute_observer_command(
        seqkit_image,
        stats_spec.mount_dir.as_path(),
        &stats_spec.args,
        runner,
    )?;
    if stats_output.exit_code != 0 {
        return Ok(None);
    }
    Ok(Some(parse_seqkit_stats(&stats_output.stdout)?))
}
