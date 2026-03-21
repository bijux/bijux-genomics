use std::collections::HashMap;
use std::path::PathBuf;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_workspace_registry};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::core_other::{fetch_fastq_validate_v1, insert_fastq_validate_v1};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkContext, BenchmarkRecord, FastqValidateMetrics,
};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::measure::SeqkitMetrics;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::select_validate_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::validate_reads::{
    plan_with_options as plan_validate_reads, ValidateReadsPlanOptions,
};
use bijux_dna_planner_fastq::stage_api::observer::{input_fastq_stats, parse_seqkit_stats};
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::backend::docker::executor::resolve_image_for_run;
use bijux_dna_runner::step_runner::{execute_observer_command, StageResultV1};

use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_VALIDATE_READS,
};
use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::internal::fastq::stages::trim_bench_common::require_existing_benchmark_output;
use bijux_dna_stage_contract::StagePlanV1;

/// # Errors
/// Returns an error if planning, execution, metric derivation, or persistence fails.
pub fn bench_fastq_validate_reads<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqValidateArgs,
) -> Result<BenchOutcome<bijux_dna_analyze::FastqValidateMetrics>> {
    let tools = select_validate_tools(&args.tools)?;
    let artifact_kind = if args.r2.is_some() {
        FastqArtifactKind::PairedEnd
    } else {
        FastqArtifactKind::SingleEnd
    };
    preflight_stage(STAGE_VALIDATE_READS.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_VALIDATE_READS.as_str(), &header);

    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_VALIDATE_READS.as_str(), &tools, &registry, false)?;
    let bench_inputs = prepare_validate_bench(catalog, platform, runner_override, args)?;

    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_VALIDATE_READS.as_str());
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
            STAGE_VALIDATE_READS.as_str(),
            &tools,
            &excluded,
            None,
        )?;
        write_explain_plan_json(
            &bench_inputs.bench_dir,
            STAGE_VALIDATE_READS.as_str(),
            &tools,
            &registry,
            None,
        )?;
    }

    ensure_image_qa_passed(STAGE_VALIDATE_READS.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_VALIDATE_READS.as_str(), &tools, platform, catalog)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut records = Vec::<BenchmarkRecord<FastqValidateMetrics>>::new();
    let mut failures = Vec::<RawFailure>::new();

    for tool in tools {
        let out_dir = bench_inputs.tools_root.join(&tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec = build_tool_execution_spec(
            STAGE_VALIDATE_READS.as_str(),
            &tool,
            &registry,
            catalog,
            platform,
        )?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = plan_validate_reads(
            &tool_spec,
            &bench_inputs.r1,
            args.r2.as_deref(),
            &out_dir,
            &ValidateReadsPlanOptions {
                q_cutoff: args.q_cutoff,
            },
        )?;
        let bench_params = benchmark_query_context()?.embed_in_parameters(&plan.params);
        let params_hash = stable_params_hash(&bench_params);
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_validate_v1(
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
            vec![bijux_dna_stage_contract::execution_step_from_stage_plan(
                &plan,
            )],
            bench_inputs.runner,
            jobs,
        )?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing execution result for {tool}"))?;

        let record = build_validate_record(
            platform,
            &bench_inputs,
            &tool,
            &tool_spec,
            &bench_params,
            &plan,
            &execution,
        )?;

        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_validate_v1(&conn, &record).context("insert bench sqlite")?;
        if execution.exit_code != 0 && args.strict {
            failures.push(RawFailure {
                stage: STAGE_VALIDATE_READS.as_str().to_string(),
                tool: tool.clone(),
                reason: format!(
                    "validator `{tool}` failed strict validation with status {}",
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
struct ValidateBenchInputs {
    runner: RuntimeKind,
    r1: PathBuf,
    input_hash: String,
    input_stats: SeqkitMetrics,
    input_stats_r2: Option<SeqkitMetrics>,
    bench_dir: PathBuf,
    tools_root: PathBuf,
}

fn prepare_validate_bench<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqValidateArgs,
) -> Result<ValidateBenchInputs> {
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir_name = bench_dir_name(&STAGE_VALIDATE_READS)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_VALIDATE_READS.as_str()))?;
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
        return Err(anyhow!(
            "seqkit validation observer failed: {}",
            stats_output.stderr
        ));
    }

    let input_hash = if let Some(r2) = args.r2.as_deref() {
        format!(
            "{}+{}",
            hash_file_sha256(&r1).context("hash validation input r1")?,
            hash_file_sha256(r2).context("hash validation input r2")?
        )
    } else {
        hash_file_sha256(&r1).context("hash validation input")?
    };
    let input_stats_r2 = if let Some(r2) = args.r2.as_deref() {
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
            return Err(anyhow!(
                "seqkit validation observer failed for r2: {}",
                stats_output.stderr
            ));
        }
        Some(parse_seqkit_stats(&stats_output.stdout)?)
    } else {
        None
    };

    Ok(ValidateBenchInputs {
        runner,
        r1,
        input_hash,
        input_stats: parse_seqkit_stats(&stats_output.stdout)?,
        input_stats_r2,
        bench_dir,
        tools_root,
    })
}

fn build_validate_record(
    platform: &PlatformSpec,
    bench_inputs: &ValidateBenchInputs,
    tool: &str,
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    params: &serde_json::Value,
    plan: &StagePlanV1,
    execution: &StageResultV1,
) -> Result<BenchmarkRecord<FastqValidateMetrics>> {
    let out_dir = &plan.out_dir;
    let metrics = derive_validate_metrics(
        &bench_inputs.input_stats,
        bench_inputs.input_stats_r2.as_ref(),
        execution,
    );
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    let report_path = required_plan_output_path(plan, "validation_report")?;
    let _report_path = require_existing_benchmark_output(&report_path, "validation_report")?;
    let manifest_path = required_plan_output_path(plan, "validated_reads_manifest")?;
    let _manifest_path =
        require_existing_benchmark_output(&manifest_path, "validated_reads_manifest")?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write validation metrics")?;

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

fn required_plan_output_path(plan: &StagePlanV1, output_id: &str) -> Result<PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| {
            anyhow!(
                "validate_reads plan is missing governed output `{output_id}` for tool {}",
                plan.tool_id.as_str()
            )
        })
}

fn derive_validate_metrics(
    input_stats: &SeqkitMetrics,
    input_stats_r2: Option<&SeqkitMetrics>,
    execution: &StageResultV1,
) -> FastqValidateMetrics {
    let merged = format!("{}\n{}", execution.stdout, execution.stderr);
    let reads_total = parse_first_u64_after_key(&merged, "read")
        .or_else(|| parse_first_u64_after_key(&merged, "sequences"))
        .unwrap_or(input_stats.reads);
    let error_count = parse_first_u64_after_key(&merged, "error").unwrap_or(0);
    let reads_invalid = error_count.min(reads_total);
    let reads_valid = if execution.exit_code == 0 {
        reads_total
    } else {
        reads_total.saturating_sub(reads_invalid)
    };
    let reads_in = input_stats.reads + input_stats_r2.map_or(0, |stats| stats.reads);
    let bases_in = input_stats.bases + input_stats_r2.map_or(0, |stats| stats.bases);
    FastqValidateMetrics {
        reads_in,
        reads_out: reads_in,
        bases_in,
        bases_out: bases_in,
        pairs_in: input_stats_r2.map(|stats| input_stats.reads.min(stats.reads)),
        pairs_out: input_stats_r2.map(|stats| input_stats.reads.min(stats.reads)),
        reads_total,
        reads_valid,
        reads_invalid,
        mean_q: input_stats.mean_q,
    }
}

fn parse_first_u64_after_key(text: &str, key: &str) -> Option<u64> {
    for line in text.lines() {
        if !line
            .to_ascii_lowercase()
            .contains(&key.to_ascii_lowercase())
        {
            continue;
        }
        let digits: String = line.chars().filter(char::is_ascii_digit).collect();
        if let Ok(parsed) = digits.parse::<u64>() {
            return Some(parsed);
        }
    }
    None
}

fn benchmark_query_context() -> Result<bijux_dna_domain_fastq::BenchQueryContext> {
    bijux_dna_domain_fastq::governed_stage_bench_query_context(STAGE_VALIDATE_READS.as_str())
}

#[cfg(test)]
mod tests {
    use super::required_plan_output_path;
    use bijux_dna_core::contract::{ArtifactRole, StageIO};
    use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, ToolId};
    use bijux_dna_core::prelude::{ArtifactRef, CommandSpecV1, ContainerImageRefV1};
    use bijux_dna_stage_contract::{PlanDecisionReason, StagePlanV1};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    #[test]
    fn validate_record_paths_follow_plan_outputs() {
        let plan = StagePlanV1 {
            stage_id: StageId::from_static("fastq.validate_reads"),
            stage_instance_id: None,
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("fastqvalidator"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: vec!["fastqvalidator".to_string()],
            },
            resources: Default::default(),
            io: StageIO {
                inputs: Vec::new(),
                outputs: vec![
                    ArtifactRef::required(
                        ArtifactId::from_static("validation_report"),
                        PathBuf::from("custom/validation.json"),
                        ArtifactRole::ReportJson,
                    ),
                    ArtifactRef::required(
                        ArtifactId::from_static("validated_reads_manifest"),
                        PathBuf::from("custom/validated_reads_manifest.json"),
                        ArtifactRole::StageReport,
                    ),
                ],
            },
            out_dir: PathBuf::from("custom"),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            aux_images: BTreeMap::new(),
            reason: PlanDecisionReason::default(),
        };

        assert_eq!(
            required_plan_output_path(&plan, "validation_report").expect("report path"),
            PathBuf::from("custom/validation.json")
        );
        assert_eq!(
            required_plan_output_path(&plan, "validated_reads_manifest").expect("manifest path"),
            PathBuf::from("custom/validated_reads_manifest.json")
        );
    }

    #[test]
    fn missing_validation_manifest_is_rejected_before_metrics() {
        let plan = StagePlanV1 {
            stage_id: StageId::from_static("fastq.validate_reads"),
            stage_instance_id: None,
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("fastqvalidator"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: vec!["fastqvalidator".to_string()],
            },
            resources: Default::default(),
            io: StageIO {
                inputs: Vec::new(),
                outputs: vec![ArtifactRef::required(
                    ArtifactId::from_static("validation_report"),
                    PathBuf::from("custom/validation.json"),
                    ArtifactRole::ReportJson,
                )],
            },
            out_dir: PathBuf::from("custom"),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            aux_images: BTreeMap::new(),
            reason: PlanDecisionReason::default(),
        };

        let error = required_plan_output_path(&plan, "validated_reads_manifest")
            .expect_err("missing manifest must be rejected");
        assert!(error
            .to_string()
            .contains("missing governed output `validated_reads_manifest`"));
    }
}
