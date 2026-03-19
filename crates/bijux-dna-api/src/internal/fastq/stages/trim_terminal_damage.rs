use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::query_shared::{
    fetch_fastq_trim_terminal_damage_v1, insert_fastq_trim_terminal_damage_v1,
};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqTrimTerminalDamageMetrics};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::params_hash;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::stage_api::fastq::trim_terminal_damage::plan_trim_terminal_damage;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifact, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use uuid::Uuid;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{filter_tools_by_role, load_registry};

use super::trim_bench_common::{
    build_benchmark_context, derive_trim_delta, infer_udg_classification, observe_fastq_stats,
    prepare_trim_bench, terminal_damage_profile,
};
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_TRIM_TERMINAL_DAMAGE,
};

fn normalize_tools(raw: &[String]) -> Vec<String> {
    if raw.is_empty() || (raw.len() == 1 && raw[0] == "auto") {
        return vec!["cutadapt".to_string(), "seqkit".to_string()];
    }
    if raw.len() == 1 && raw[0] == "all" {
        return vec!["cutadapt".to_string(), "seqkit".to_string()];
    }
    raw.to_vec()
}

/// # Errors
/// Returns an error if planning, execution, metric derivation, or persistence fails.
pub fn bench_fastq_trim_terminal_damage<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimTerminalDamageArgs,
) -> Result<BenchOutcome<FastqTrimTerminalDamageMetrics>> {
    let requested = normalize_tools(&args.tools);
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage(STAGE_TRIM_TERMINAL_DAMAGE.as_str(), artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings(STAGE_TRIM_TERMINAL_DAMAGE.as_str(), &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(
        STAGE_TRIM_TERMINAL_DAMAGE.as_str(),
        &requested,
        &registry,
        false,
    )?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_TRIM_TERMINAL_DAMAGE,
    )?;

    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_TRIM_TERMINAL_DAMAGE.as_str());
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
        let plan = plan_trim_terminal_damage(
            &tool_spec,
            &bench_inputs.r1,
            &out_dir,
            args.damage_mode.as_deref().unwrap_or("ancient"),
            args.trim_5p_bases.unwrap_or(2),
            args.trim_3p_bases.unwrap_or(2),
        )?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_trim_terminal_damage_v1(
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
                stage: STAGE_TRIM_TERMINAL_DAMAGE.as_str().to_string(),
                tool: tool.clone(),
                reason: format!("tool `{tool}` failed with status {}", execution.exit_code),
                category: ErrorCategory::ToolError,
            });
            continue;
        }

        let output_fastq = plan.io.outputs[0].path.clone();
        let output_stats = observe_fastq_stats(catalog, platform, bench_inputs.runner, &output_fastq)?;
        let metrics = FastqTrimTerminalDamageMetrics {
            reads_in: bench_inputs.input_stats.reads,
            reads_out: output_stats.reads,
            bases_in: bench_inputs.input_stats.bases,
            bases_out: output_stats.bases,
            pairs_in: None,
            pairs_out: None,
            mean_q_before: bench_inputs.input_stats.mean_q,
            mean_q_after: output_stats.mean_q,
            delta_metrics: derive_trim_delta(&bench_inputs.input_stats, &output_stats),
        };
        let metric_set = metric_set(metrics.clone());
        bijux_dna_analyze::validate_metric_set(&metric_set)?;

        let pre_profile = terminal_damage_profile(&bench_inputs.r1)?;
        let post_profile = terminal_damage_profile(&output_fastq)?;
        let udg_classification = args
            .damage_mode
            .clone()
            .unwrap_or_else(|| infer_udg_classification(&bench_inputs.r1));
        let report = serde_json::json!({
            "schema_version": "bijux.fastq.trim_terminal_damage.report.v1",
            "stage_id": STAGE_TRIM_TERMINAL_DAMAGE.as_str(),
            "tool_id": tool,
            "reads_in": metrics.reads_in,
            "reads_out": metrics.reads_out,
            "bases_in": metrics.bases_in,
            "bases_out": metrics.bases_out,
            "mean_q_before": metrics.mean_q_before,
            "mean_q_after": metrics.mean_q_after,
            "udg_classification": udg_classification,
            "trim_5p_bases": args.trim_5p_bases.unwrap_or(2),
            "trim_3p_bases": args.trim_3p_bases.unwrap_or(2),
            "terminal_base_composition_pre": pre_profile.get("terminal_base_composition_5p").cloned().unwrap_or_else(|| serde_json::json!({})),
            "terminal_base_composition_post": post_profile.get("terminal_base_composition_5p").cloned().unwrap_or_else(|| serde_json::json!({})),
            "ct_ga_asymmetry_pre": pre_profile.get("ct_ga_asymmetry").cloned().unwrap_or_else(|| serde_json::json!(0.0)),
            "ct_ga_asymmetry_post": post_profile.get("ct_ga_asymmetry").cloned().unwrap_or_else(|| serde_json::json!(0.0)),
            "runtime_s": execution.runtime_s,
            "memory_mb": execution.memory_mb,
        });
        bijux_dna_infra::atomic_write_json(&out_dir.join("trim_terminal_damage_report.json"), &report)
            .context("write trim terminal damage report")?;
        let metrics_json = serde_json::to_value(&metric_set)?;
        bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
            .context("write trim terminal damage metrics")?;

        let context = build_benchmark_context(
            &tool,
            tool_spec.tool_version.clone(),
            image_digest,
            bench_inputs.runner,
            platform,
            bench_inputs.input_hash.clone(),
            plan.params.clone(),
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
        insert_fastq_trim_terminal_damage_v1(&conn, &record)
            .context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: bench_inputs.bench_dir,
        explain: args.explain,
    })
}
