use std::collections::HashMap;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_workspace_registry};
use anyhow::{anyhow, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_duplicates_v1, insert_fastq_duplicates_v1,
};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqDuplicateMetrics};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use crate::internal::fastq::stages::trim_bench_common::{
    build_benchmark_context, observe_fastq_stats, require_existing_benchmark_output,
};
use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};

const STAGE_ID: &str = "fastq.remove_duplicates";

#[derive(Debug, Clone, PartialEq)]
struct DuplicateReportCounts {
    reads_in: u64,
    reads_out: u64,
    duplicate_reads: u64,
    dedup_rate: f64,
}

fn ensure_remove_duplicates_tools_support_input_mode(
    tools: &[String],
    paired_mode: bool,
) -> Result<()> {
    let incompatible = tools
        .iter()
        .filter(|tool_id| {
            !bijux_dna_planner_fastq::tool_adapters::fastq::remove_duplicates::deduplicate_tool_supports_paired_mode(
                tool_id,
                paired_mode,
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    if incompatible.is_empty() {
        return Ok(());
    }
    Err(anyhow!(
        "fastq.remove_duplicates does not support {} inputs for tool(s): {}",
        if paired_mode {
            "paired-end"
        } else {
            "single-end"
        },
        incompatible.join(", "),
    ))
}

pub fn bench_fastq_remove_duplicates<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqRemoveDuplicatesArgs,
) -> Result<BenchOutcome<FastqDuplicateMetrics>> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = bijux_dna_planner_fastq::select_remove_duplicates_tools(&args.tools)?;
    let tools = filter_tools_by_role(STAGE_ID, &tools, &registry, false)?;
    ensure_remove_duplicates_tools_support_input_mode(&tools, args.r2.is_some())?;
    let artifact_kind = if args.r2.is_some() {
        FastqArtifactKind::PairedEnd
    } else {
        FastqArtifactKind::SingleEnd
    };
    preflight_stage(STAGE_ID, artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_ID, &header);
    let runner = ensure_bench_runner(platform, runner_override)?;
    let input_stats = observe_fastq_stats(catalog, platform, runner, &args.r1)?;
    let input_stats_r2 = if let Some(r2) = args.r2.as_deref() {
        Some(observe_fastq_stats(catalog, platform, runner, r2)?)
    } else {
        None
    };
    let input_hash = if let Some(r2) = args.r2.as_deref() {
        format!("{}+{}", hash_file_sha256(&args.r1)?, hash_file_sha256(r2)?)
    } else {
        hash_file_sha256(&args.r1)?
    };
    let bench_dir_name =
        bench_dir_name(&bijux_dna_domain_fastq::stages::ids::STAGE_REMOVE_DUPLICATES)
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
        let plan = bijux_dna_planner_fastq::tool_adapters::fastq::remove_duplicates::plan_deduplicate(
            &tool_spec,
            &args.r1,
            args.r2.as_deref(),
            &out_dir,
        )?;
        let bench_params = benchmark_query_context()?.embed_in_parameters(&plan.params);
        let params_hash = stable_params_hash(&bench_params);
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_duplicates_v1(
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
            vec![bijux_dna_stage_contract::execution_step_from_stage_plan(
                &plan,
            )],
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
        let output_r1 = require_existing_benchmark_output(&plan.io.outputs[0].path, "dedup_reads_r1")?;
        let output_stats = observe_fastq_stats(catalog, platform, runner, output_r1)?;
        let output_stats_r2 = if args.r2.is_some() {
            let output_r2 = &plan.io.outputs[1].path;
            let output_r2 = require_existing_benchmark_output(output_r2, "dedup_reads_r2")?;
            Some(observe_fastq_stats(catalog, platform, runner, output_r2)?)
        } else {
            None
        };
        let counts = deduplicate_report_counts(
            input_stats.reads,
            input_stats_r2.as_ref().map(|stats| stats.reads),
            output_stats.reads,
            output_stats_r2.as_ref().map(|stats| stats.reads),
        );
        let metrics = FastqDuplicateMetrics {
            reads_in: counts.reads_in,
            reads_out: counts.reads_out,
            duplicate_reads: counts.duplicate_reads,
            dedup_rate: counts.dedup_rate,
        };
        let metric_set = metric_set(metrics);
        let report = build_deduplicate_report_json(
            tool,
            &args.r1,
            args.r2.as_deref(),
            &plan,
            &counts,
            &execution,
        );
        bijux_dna_infra::atomic_write_json(&out_dir.join("deduplicate_report.json"), &report)?;
        bijux_dna_infra::atomic_write_json(
            &out_dir.join("metrics.json"),
            &serde_json::to_value(&metric_set)?,
        )?;
        let record = BenchmarkRecord {
            context: build_benchmark_context(
                tool,
                tool_spec.tool_version.clone(),
                image_digest,
                runner,
                platform,
                input_hash.clone(),
                bench_params.clone(),
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
        insert_fastq_duplicates_v1(&conn, &record)?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir,
        explain: args.explain,
    })
}

fn benchmark_query_context() -> Result<bijux_dna_domain_fastq::BenchQueryContext> {
    bijux_dna_domain_fastq::governed_stage_bench_query_context(STAGE_ID)
}

fn deduplicate_report_counts(
    input_reads_r1: u64,
    input_reads_r2: Option<u64>,
    output_reads_r1: u64,
    output_reads_r2: Option<u64>,
) -> DuplicateReportCounts {
    let reads_in = input_reads_r1 + input_reads_r2.unwrap_or(0);
    let reads_out = output_reads_r1 + output_reads_r2.unwrap_or(0);
    let duplicate_reads = reads_in.saturating_sub(reads_out);
    let dedup_rate = if reads_in == 0 {
        0.0
    } else {
        duplicate_reads as f64 / reads_in as f64
    };
    DuplicateReportCounts {
        reads_in,
        reads_out,
        duplicate_reads,
        dedup_rate,
    }
}

fn build_deduplicate_report_json(
    tool: &str,
    input_r1: &std::path::Path,
    input_r2: Option<&std::path::Path>,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    counts: &DuplicateReportCounts,
    execution: &bijux_dna_runner::step_runner::StageResultV1,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "bijux.fastq.remove_duplicates.report.v1",
        "stage": STAGE_ID,
        "stage_id": STAGE_ID,
        "tool": tool,
        "tool_id": tool,
        "input_fastq_r1": input_r1,
        "input_fastq_r2": input_r2,
        "dedup_reads_r1": plan.io.outputs[0].path,
        "dedup_reads_r2": if input_r2.is_some() {
            Some(plan.io.outputs[1].path.clone())
        } else {
            None::<std::path::PathBuf>
        },
        "report_json": if input_r2.is_some() {
            plan.io.outputs[2].path.clone()
        } else {
            plan.io.outputs[1].path.clone()
        },
        "reads_in": counts.reads_in,
        "reads_out": counts.reads_out,
        "duplicates_removed": counts.duplicate_reads,
        "duplicate_reads": counts.duplicate_reads,
        "dedup_rate": counts.dedup_rate,
        "runtime_s": execution.runtime_s,
        "memory_mb": execution.memory_mb,
        "exit_code": execution.exit_code,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{build_deduplicate_report_json, deduplicate_report_counts, DuplicateReportCounts};
    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolConstraints, ToolId,
    };
    use bijux_dna_stage_contract::{ArtifactRef, PlanDecisionReason, StageIO, StagePlanV1};
    use bijux_dna_runner::step_runner::StageResultV1;

    fn plan_with_outputs(paired: bool) -> StagePlanV1 {
        let mut outputs = vec![ArtifactRef::required(
            ArtifactId::from_static("dedup_reads_r1"),
            PathBuf::from("out/dedup_r1.fastq.gz"),
            ArtifactRole::Reads,
        )];
        if paired {
            outputs.push(ArtifactRef::required(
                ArtifactId::from_static("dedup_reads_r2"),
                PathBuf::from("out/dedup_r2.fastq.gz"),
                ArtifactRole::Reads,
            ));
        }
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("report_json"),
            PathBuf::from("out/deduplicate_report.json"),
            ArtifactRole::ReportJson,
        ));
        StagePlanV1 {
            stage_id: StageId::from_static("fastq.remove_duplicates"),
            stage_instance_id: None,
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("clumpify"),
            tool_version: "test".to_string(),
            image: serde_json::from_value(serde_json::json!({
                "image": "bijuxdna/clumpify",
                "digest": null,
            }))
            .expect("image"),
            command: CommandSpecV1 {
                template: vec!["echo".to_string(), "ok".to_string()],
            },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: vec![],
                outputs,
            },
            out_dir: PathBuf::from("out"),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            aux_images: std::collections::BTreeMap::new(),
            reason: PlanDecisionReason::default(),
        }
    }

    fn execution() -> StageResultV1 {
        StageResultV1 {
            run_id: "run".to_string(),
            exit_code: 0,
            runtime_s: 1.5,
            memory_mb: 128.0,
            outputs: vec![],
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: "dedup".to_string(),
        }
    }

    #[test]
    fn deduplicate_counts_cover_paired_inputs() {
        let counts = deduplicate_report_counts(100, Some(100), 70, Some(70));
        assert_eq!(
            counts,
            DuplicateReportCounts {
                reads_in: 200,
                reads_out: 140,
                duplicate_reads: 60,
                dedup_rate: 0.3,
            }
        );
    }

    #[test]
    fn deduplicate_report_includes_parser_backed_counts() {
        let plan = plan_with_outputs(true);
        let counts = deduplicate_report_counts(100, Some(100), 80, Some(80));
        let report = build_deduplicate_report_json(
            "clumpify",
            std::path::Path::new("reads_r1.fastq.gz"),
            Some(std::path::Path::new("reads_r2.fastq.gz")),
            &plan,
            &counts,
            &execution(),
        );
        assert_eq!(report["reads_in"], serde_json::json!(200));
        assert_eq!(report["reads_out"], serde_json::json!(160));
        assert_eq!(report["duplicate_reads"], serde_json::json!(40));
        assert_eq!(report["dedup_rate"], serde_json::json!(0.2));
        assert_eq!(
            report["report_json"],
            serde_json::json!("out/deduplicate_report.json")
        );
    }
}
