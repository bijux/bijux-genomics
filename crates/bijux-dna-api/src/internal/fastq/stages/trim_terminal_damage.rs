use std::collections::HashMap;

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
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_domain_fastq::params::trim::TrimTerminalDamageParams;
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::stage_api::fastq::trim_terminal_damage::plan_trim_terminal_damage;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{filter_tools_by_role, load_workspace_registry};
use crate::internal::fastq::stages::record_identity::stable_params_hash;

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

/// # Errors
/// Returns an error if planning, execution, metric derivation, or persistence fails.
pub fn bench_fastq_trim_terminal_damage<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimTerminalDamageArgs,
) -> Result<BenchOutcome<FastqTrimTerminalDamageMetrics>> {
    let requested = normalize_tools(&args.tools);
    let artifact_kind = if args.r2.is_some() {
        FastqArtifactKind::PairedEnd
    } else {
        FastqArtifactKind::SingleEnd
    };
    preflight_stage(STAGE_TRIM_TERMINAL_DAMAGE.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_TRIM_TERMINAL_DAMAGE.as_str(), &header);

    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
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
    let input_hash = if let Some(r2) = args.r2.as_deref() {
        format!(
            "{}+{}",
            bench_inputs.input_hash,
            bijux_dna_infra::hash_file_sha256(r2)?
        )
    } else {
        bench_inputs.input_hash.clone()
    };
    let input_stats_r2 = if let Some(r2) = args.r2.as_deref() {
        Some(observe_fastq_stats(
            catalog,
            platform,
            bench_inputs.runner,
            r2,
        )?)
    } else {
        None
    };

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

    ensure_image_qa_passed(
        STAGE_TRIM_TERMINAL_DAMAGE.as_str(),
        &tools,
        platform,
        catalog,
    )?;
    ensure_tool_qa_passed(
        STAGE_TRIM_TERMINAL_DAMAGE.as_str(),
        &tools,
        platform,
        catalog,
    )?;

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
            args.r2.as_deref(),
            &out_dir,
            args.damage_mode.as_deref().unwrap_or("ancient"),
            args.trim_5p_bases.unwrap_or(2),
            args.trim_3p_bases.unwrap_or(2),
        )?;
        let bench_params = benchmark_query_context()?.embed_in_parameters(&plan.params);
        let params_hash = stable_params_hash(&bench_params);
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
            &input_hash,
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
        let metrics = FastqTrimTerminalDamageMetrics {
            reads_in: before_stats.reads,
            reads_out: after_stats.reads,
            bases_in: before_stats.bases,
            bases_out: after_stats.bases,
            pairs_in: input_stats_r2
                .as_ref()
                .map(|stats| bench_inputs.input_stats.reads.min(stats.reads)),
            pairs_out: output_stats_r2
                .as_ref()
                .map(|stats| output_stats_r1.reads.min(stats.reads)),
            mean_q_before: before_stats.mean_q,
            mean_q_after: after_stats.mean_q,
            delta_metrics: derive_trim_delta(&before_stats, &after_stats),
        };
        let metric_set = metric_set(metrics.clone());
        bijux_dna_analyze::validate_metric_set(&metric_set)?;
        let effective_params =
            serde_json::from_value::<TrimTerminalDamageParams>(plan.effective_params.clone())
                .context("decode trim terminal damage effective params")?;

        let pre_profile_r1 = terminal_damage_profile(&bench_inputs.r1)?;
        let pre_profile_r2 = if let Some(r2) = args.r2.as_deref() {
            Some(terminal_damage_profile(r2)?)
        } else {
            None
        };
        let post_profile_r1 = terminal_damage_profile(&output_r1)?;
        let post_profile_r2 = if args.r2.is_some() {
            Some(terminal_damage_profile(&plan.io.outputs[1].path)?)
        } else {
            None
        };
        let combined_asymmetry =
            |left: Option<&serde_json::Value>, right: Option<&serde_json::Value>| -> Option<f64> {
                let values = [
                    left.and_then(serde_json::Value::as_f64),
                    right.and_then(serde_json::Value::as_f64),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
                if values.is_empty() {
                    None
                } else {
                    Some(values.iter().sum::<f64>() / values.len() as f64)
                }
            };
        let ct_ga_asymmetry_pre = combined_asymmetry(
            pre_profile_r1.get("ct_ga_asymmetry"),
            pre_profile_r2
                .as_ref()
                .and_then(|profile| profile.get("ct_ga_asymmetry")),
        );
        let ct_ga_asymmetry_post = combined_asymmetry(
            post_profile_r1.get("ct_ga_asymmetry"),
            post_profile_r2
                .as_ref()
                .and_then(|profile| profile.get("ct_ga_asymmetry")),
        );
        let udg_classification = args
            .damage_mode
            .clone()
            .unwrap_or_else(|| infer_udg_classification(&bench_inputs.r1));
        let report = serde_json::json!({
            "schema_version": "bijux.fastq.trim_terminal_damage.report.v1",
            "stage": STAGE_TRIM_TERMINAL_DAMAGE.as_str(),
            "stage_id": STAGE_TRIM_TERMINAL_DAMAGE.as_str(),
            "tool": tool,
            "tool_id": tool,
            "reads_in": metrics.reads_in,
            "reads_out": metrics.reads_out,
            "bases_in": metrics.bases_in,
            "bases_out": metrics.bases_out,
            "mean_q_before": metrics.mean_q_before,
            "mean_q_after": metrics.mean_q_after,
            "udg_classification": udg_classification,
            "ct_ga_asymmetry_pre": ct_ga_asymmetry_pre,
            "ct_ga_asymmetry_post": ct_ga_asymmetry_post,
            "output_r1": output_r1,
            "output_r2": args.r2.as_ref().map(|_| plan.io.outputs[1].path.clone()),
            "execution_policy": effective_params.execution_policy,
            "trim_5p_bases": effective_params.trim_5p_bases,
            "trim_3p_bases": effective_params.trim_3p_bases,
            "requested_trim_5p_bases": effective_params.requested_trim_5p_bases,
            "requested_trim_3p_bases": effective_params.requested_trim_3p_bases,
            "terminal_base_composition_pre_r1": pre_profile_r1.get("terminal_base_composition_5p").cloned().unwrap_or_else(|| serde_json::json!({})),
            "terminal_base_composition_post_r1": post_profile_r1.get("terminal_base_composition_5p").cloned().unwrap_or_else(|| serde_json::json!({})),
            "ct_ga_asymmetry_pre_r1": pre_profile_r1.get("ct_ga_asymmetry").cloned().unwrap_or_else(|| serde_json::json!(0.0)),
            "ct_ga_asymmetry_post_r1": post_profile_r1.get("ct_ga_asymmetry").cloned().unwrap_or_else(|| serde_json::json!(0.0)),
            "terminal_base_composition_pre_r2": pre_profile_r2.as_ref().and_then(|profile| profile.get("terminal_base_composition_5p")).cloned(),
            "terminal_base_composition_post_r2": post_profile_r2.as_ref().and_then(|profile| profile.get("terminal_base_composition_5p")).cloned(),
            "ct_ga_asymmetry_pre_r2": pre_profile_r2.as_ref().and_then(|profile| profile.get("ct_ga_asymmetry")).cloned(),
            "ct_ga_asymmetry_post_r2": post_profile_r2.as_ref().and_then(|profile| profile.get("ct_ga_asymmetry")).cloned(),
            "runtime_s": execution.runtime_s,
            "memory_mb": execution.memory_mb,
        });
        bijux_dna_infra::atomic_write_json(
            &out_dir.join("trim_terminal_damage_report.json"),
            &report,
        )
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

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: bench_inputs.bench_dir,
        explain: args.explain,
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
        ((primary.mean_q * primary.bases as f64)
            + secondary.map_or(0.0, |stats| stats.mean_q * stats.bases as f64))
            / total_bases as f64
    };
    let weighted_gc = if total_bases == 0 {
        0.0
    } else {
        ((primary.gc_percent * primary.bases as f64)
            + secondary.map_or(0.0, |stats| stats.gc_percent * stats.bases as f64))
            / total_bases as f64
    };
    SeqkitMetrics {
        reads: primary.reads + secondary_reads,
        bases: total_bases,
        mean_q: weighted_mean_q,
        gc_percent: weighted_gc,
    }
}

fn benchmark_query_context() -> Result<bijux_dna_domain_fastq::BenchQueryContext> {
    bijux_dna_domain_fastq::governed_stage_bench_query_context(STAGE_TRIM_TERMINAL_DAMAGE.as_str())
}

#[cfg(test)]
mod tests {
    use super::{admitted_stage_tools, normalize_tools};

    #[test]
    fn normalize_tools_uses_execution_support_for_auto_and_all() {
        let expected = admitted_stage_tools();
        assert_eq!(normalize_tools(&[]), expected);
        assert_eq!(normalize_tools(&["auto".to_string()]), expected);
        assert_eq!(normalize_tools(&["all".to_string()]), expected);
    }
}
