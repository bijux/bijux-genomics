use std::collections::HashMap;
use std::path::Path;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_registry};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{fetch_fastq_screen_v1, insert_fastq_screen_v1};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqScreenMetrics};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::params_hash;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::select_screen_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::screen_taxonomy::plan_screen;
use bijux_dna_planner_fastq::stage_api::observer::{input_fastq_stats, parse_seqkit_stats};
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::backend::docker::executor::resolve_image_for_run;
use bijux_dna_runner::step_runner::{execute_observer_command, StageResultV1};
use uuid::Uuid;

use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_SCREEN_TAXONOMY,
};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;

use super::trim_bench_common::build_benchmark_context;

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_screen<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqScreenArgs,
) -> Result<BenchOutcome<FastqScreenMetrics>> {
    let tools = select_screen_tools(&args.tools)?;
    let artifact_kind = if args.r2.is_some() {
        FastqArtifactKind::PairedEnd
    } else {
        FastqArtifactKind::SingleEnd
    };
    preflight_stage(STAGE_SCREEN_TAXONOMY.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_SCREEN_TAXONOMY.as_str(), &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_SCREEN_TAXONOMY.as_str(), &tools, &registry, false)?;
    let bench_inputs = prepare_screen_bench(catalog, platform, runner_override, args)?;

    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_SCREEN_TAXONOMY.as_str());
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
            STAGE_SCREEN_TAXONOMY.as_str(),
            &tools,
            &excluded,
            None,
        )?;
        write_explain_plan_json(
            &bench_inputs.bench_dir,
            STAGE_SCREEN_TAXONOMY.as_str(),
            &tools,
            &registry,
            None,
        )?;
    }

    ensure_image_qa_passed(STAGE_SCREEN_TAXONOMY.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_SCREEN_TAXONOMY.as_str(), &tools, platform, catalog)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::<RawFailure>::new();
    let mut records = Vec::<BenchmarkRecord<FastqScreenMetrics>>::new();
    for tool in tools {
        let out_dir = bench_inputs.tools_root.join(&tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec = build_tool_execution_spec(
            STAGE_SCREEN_TAXONOMY.as_str(),
            &tool,
            &registry,
            catalog,
            platform,
        )?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = plan_screen(&tool_spec, &bench_inputs.r1, bench_inputs.r2.as_deref(), &out_dir)?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_screen_v1(
            &conn,
            &tool,
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
            vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&plan)],
            bench_inputs.runner,
            jobs,
        )?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing execution result for {tool}"))?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: STAGE_SCREEN_TAXONOMY.as_str().to_string(),
                tool: tool.clone(),
                reason: format!("tool `{tool}` failed with status {}", execution.exit_code),
                category: ErrorCategory::ToolError,
            });
            continue;
        }
        let record = build_screen_record(
            platform,
            &bench_inputs,
            &tool,
            &tool_spec,
            &plan.params,
            &out_dir,
            &execution,
        )?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_screen_v1(&conn, &record).context("insert bench sqlite")?;
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
struct ScreenBenchInputs {
    runner: RuntimeKind,
    r1: std::path::PathBuf,
    r2: Option<std::path::PathBuf>,
    input_hash: String,
    input_stats: SeqkitMetrics,
    input_stats_r2: Option<SeqkitMetrics>,
    bench_dir: std::path::PathBuf,
    tools_root: std::path::PathBuf,
}

fn prepare_screen_bench<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqScreenArgs,
) -> Result<ScreenBenchInputs> {
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir_name = bench_dir_name(&STAGE_SCREEN_TAXONOMY)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_SCREEN_TAXONOMY.as_str()))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_dna_infra::ensure_dir(&tools_root).context("create tools output dir")?;

    let r1 = args.r1.canonicalize().context("resolve r1 path")?;
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
        return Err(anyhow!("seqkit screen observer failed: {}", stats_output.stderr));
    }

    let (r2, input_stats_r2) = if let Some(r2) = args.r2.as_deref() {
        let r2 = r2.canonicalize().context("resolve r2 path")?;
        let r2_dir = r2
            .parent()
            .ok_or_else(|| anyhow!("r2 has no parent"))?
            .to_path_buf();
        let stats_spec = input_fastq_stats(&r2_dir, &r2)?;
        let stats_output = execute_observer_command(
            &seqkit_image.full_name,
            stats_spec.mount_dir.as_path(),
            &stats_spec.args,
            runner,
        )?;
        if stats_output.exit_code != 0 {
            return Err(anyhow!("seqkit screen observer failed for r2: {}", stats_output.stderr));
        }
        (Some(r2), Some(parse_seqkit_stats(&stats_output.stdout)?))
    } else {
        (None, None)
    };

    let input_hash = if let Some(r2) = r2.as_ref() {
        format!(
            "{}+{}",
            hash_file_sha256(&args.r1).context("hash screen input r1")?,
            hash_file_sha256(r2).context("hash screen input r2")?
        )
    } else {
        hash_file_sha256(&args.r1).context("hash screen input")?
    };

    Ok(ScreenBenchInputs {
        runner,
        r1,
        r2,
        input_hash,
        input_stats: parse_seqkit_stats(&stats_output.stdout)?,
        input_stats_r2,
        bench_dir,
        tools_root,
    })
}

fn build_screen_record(
    platform: &PlatformSpec,
    bench_inputs: &ScreenBenchInputs,
    tool: &str,
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    params: &serde_json::Value,
    out_dir: &Path,
    execution: &StageResultV1,
) -> Result<BenchmarkRecord<FastqScreenMetrics>> {
    let report_path = params
        .get("report")
        .and_then(serde_json::Value::as_str)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| out_dir.join("screen_report.tsv"));
    let (contamination_rate, contamination_summary) = parse_screen_report(&report_path)?;
    let reads_in = bench_inputs.input_stats.reads
        + bench_inputs
            .input_stats_r2
            .as_ref()
            .map_or(0, |stats| stats.reads);
    let bases_in = bench_inputs.input_stats.bases
        + bench_inputs
            .input_stats_r2
            .as_ref()
            .map_or(0, |stats| stats.bases);
    let pairs = bench_inputs
        .input_stats_r2
        .as_ref()
        .map_or(0, |stats| bench_inputs.input_stats.reads.min(stats.reads));
    let metrics = FastqScreenMetrics {
        reads_in,
        reads_out: reads_in,
        bases_in,
        bases_out: bases_in,
        pairs_in: pairs,
        pairs_out: pairs,
        contamination_rate,
        contamination_summary: contamination_summary.into(),
    };
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    let report = serde_json::json!({
        "schema_version": "bijux.fastq.screen_taxonomy.report.v1",
        "stage_id": STAGE_SCREEN_TAXONOMY.as_str(),
        "tool_id": tool,
        "input_fastq_r1": bench_inputs.r1,
        "input_fastq_r2": bench_inputs.r2,
        "reads_in": metrics.reads_in,
        "reads_out": metrics.reads_out,
        "bases_in": metrics.bases_in,
        "bases_out": metrics.bases_out,
        "pairs_in": metrics.pairs_in,
        "pairs_out": metrics.pairs_out,
        "contamination_rate": metrics.contamination_rate,
        "contamination_summary": metrics.contamination_summary,
        "runtime_s": execution.runtime_s,
        "memory_mb": execution.memory_mb,
    });
    bijux_dna_infra::atomic_write_json(&out_dir.join("screen_taxonomy_report.json"), &report)
        .context("write screen taxonomy report")?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write screen taxonomy metrics")?;

    let context = build_benchmark_context(
        tool,
        tool_spec.tool_version.clone(),
        tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone(),
        bench_inputs.runner,
        platform,
        bench_inputs.input_hash.clone(),
        params.clone(),
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
    Ok(record)
}

fn parse_screen_report(path: &Path) -> Result<(f64, serde_json::Value)> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("screen report missing: {}", path.display()))?;
    let mut entries = Vec::new();
    let mut unmapped_percent = None;
    for (idx, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            return Err(anyhow!(
                "screen report line {} has {} columns",
                idx + 1,
                parts.len()
            ));
        }
        let label = parts[0].trim().to_string();
        let percent = parts
            .last()
            .ok_or_else(|| anyhow!("screen report line {} missing percent", idx + 1))?
            .trim()
            .trim_end_matches('%')
            .parse::<f64>()
            .with_context(|| format!("screen report line {} percent parse", idx + 1))?;
        let label_lower = label.to_lowercase();
        if label_lower.contains("unmapped")
            || (label_lower.contains("no hit") && unmapped_percent.is_none())
        {
            unmapped_percent = Some(percent);
        }
        entries.push(serde_json::json!({
            "reference": label,
            "percent": percent,
        }));
    }
    let contamination_rate = unmapped_percent.map_or(0.0, |value| (100.0 - value).max(0.0) / 100.0);
    Ok((
        contamination_rate,
        serde_json::json!({
            "schema_version": "bijux.screen_summary.v1",
            "entries": entries,
        }),
    ))
}
