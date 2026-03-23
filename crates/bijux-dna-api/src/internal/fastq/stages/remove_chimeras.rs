use std::collections::HashMap;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_workspace_registry};
use anyhow::{anyhow, Result};
use bijux_dna_analyze::load::sqlite::bench::{fetch_fastq_chimeras_v1, insert_fastq_chimeras_v1};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqChimeraMetrics};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::params_hash;
use bijux_dna_domain_fastq::{
    params::edna::ChimeraDetectionEffectiveParams, PairedMode, RemoveChimerasReportV1,
    REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION,
};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::stage_api::{
    bench_dir_name, inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind,
    RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use uuid::Uuid;

use crate::internal::fastq::stages::trim_bench_common::{
    build_benchmark_context, observe_fastq_stats,
};
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};

const STAGE_ID: &str = "fastq.remove_chimeras";

pub fn bench_fastq_remove_chimeras<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqRemoveChimerasArgs,
) -> Result<BenchOutcome<FastqChimeraMetrics>> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = bijux_dna_planner_fastq::select_remove_chimeras_tools(&args.tools)?;
    let tools = filter_tools_by_role(STAGE_ID, &tools, &registry, false)?;
    let runner = ensure_bench_runner(platform, runner_override)?;
    let artifact_kind = if args.r2.is_some() {
        FastqArtifactKind::PairedEnd
    } else {
        FastqArtifactKind::SingleEnd
    };
    preflight_stage(STAGE_ID, artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_ID, &header);
    let input_stats_r1 = observe_fastq_stats(catalog, platform, runner, &args.r1)?;
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
        bench_dir_name(&bijux_dna_domain_fastq::stages::ids::STAGE_REMOVE_CHIMERAS)
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
        let plan = bijux_dna_planner_fastq::tool_adapters::fastq::remove_chimeras::plan(
            &tool_spec,
            &args.r1,
            args.r2.as_deref(),
            &out_dir,
        )?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_chimeras_v1(
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
        let filtered_reads = plan
            .io
            .outputs
            .iter()
            .find(|artifact| artifact.name.as_str() == "chimera_filtered_reads")
            .ok_or_else(|| anyhow!("remove_chimeras plan missing chimera_filtered_reads"))?;
        let metrics_output = plan
            .io
            .outputs
            .iter()
            .find(|artifact| artifact.name.as_str() == "chimera_metrics_json")
            .ok_or_else(|| anyhow!("remove_chimeras plan missing chimera_metrics_json"))?;
        let report_output = plan
            .io
            .outputs
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .ok_or_else(|| anyhow!("remove_chimeras plan missing report_json"))?;
        let chimeras_fasta = plan
            .io
            .outputs
            .iter()
            .find(|artifact| artifact.name.as_str() == "chimeras_fasta")
            .map(|artifact| artifact.path.clone());
        let uchime_report_tsv = plan
            .io
            .outputs
            .iter()
            .find(|artifact| artifact.name.as_str() == "uchime_report_tsv")
            .map(|artifact| artifact.path.clone());
        let used_fallback = !filtered_reads.path.exists();
        if used_fallback {
            std::fs::copy(&args.r1, &filtered_reads.path)?;
        }
        let output_stats_r1 = observe_fastq_stats(catalog, platform, runner, &filtered_reads.path)?;
        let reads_in =
            input_stats_r1.reads + input_stats_r2.as_ref().map_or(0, |stats| stats.reads);
        let reads_out = output_stats_r1.reads;
        let chimeras_removed = reads_in.saturating_sub(reads_out);
        let chimera_fraction = if reads_in == 0 {
            0.0
        } else {
            chimeras_removed as f64 / reads_in as f64
        };
        let effective_params: ChimeraDetectionEffectiveParams =
            serde_json::from_value(plan.effective_params.clone())
                .map_err(|error| anyhow!("parse remove_chimeras effective params: {error}"))?;
        let report = build_remove_chimeras_report(
            tool,
            &effective_params,
            &args.r1,
            &filtered_reads.path,
            &metrics_output.path,
            chimeras_fasta.as_deref(),
            uchime_report_tsv.as_deref(),
            reads_in,
            reads_out,
            chimeras_removed,
            chimera_fraction,
            used_fallback,
            execution.runtime_s,
            execution.memory_mb,
            execution.exit_code,
        );
        bijux_dna_infra::atomic_write_json(&report_output.path, &report)?;
        bijux_dna_infra::atomic_write_json(
            &metrics_output.path,
            &compatibility_metrics_from_report(&report),
        )?;
        let metrics = FastqChimeraMetrics {
            reads_in,
            reads_out,
            chimeras_removed,
            chimera_fraction,
        };
        let metric_set = metric_set(metrics);
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
        insert_fastq_chimeras_v1(&conn, &record)?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir,
        explain: args.explain,
    })
}

fn parse_uchime_summary(path: Option<&std::path::Path>) -> Option<serde_json::Value> {
    let path = path?;
    let raw = std::fs::read_to_string(path).ok()?;
    let parsed_records = raw.lines().filter(|line| !line.trim().is_empty()).count() as u64;
    let flagged_records = raw
        .lines()
        .filter(|line| line.split('\t').next_back().is_some_and(|flag| flag == "Y"))
        .count() as u64;
    Some(serde_json::json!({
        "parsed_records": parsed_records,
        "flagged_records": flagged_records,
    }))
}

fn build_remove_chimeras_report(
    tool_id: &str,
    effective_params: &ChimeraDetectionEffectiveParams,
    input_reads: &std::path::Path,
    output_reads: &std::path::Path,
    chimera_metrics_json: &std::path::Path,
    chimeras_fasta: Option<&std::path::Path>,
    uchime_report_tsv: Option<&std::path::Path>,
    reads_in: u64,
    reads_out: u64,
    chimeras_removed: u64,
    chimera_fraction: f64,
    used_fallback: bool,
    runtime_s: f64,
    memory_mb: f64,
    exit_code: i32,
) -> RemoveChimerasReportV1 {
    RemoveChimerasReportV1 {
        schema_version: REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_ID.to_string(),
        stage_id: STAGE_ID.to_string(),
        tool_id: tool_id.to_string(),
        paired_mode: PairedMode::SingleEnd,
        threads: effective_params.threads,
        method: effective_params.method.clone(),
        detection_scope: effective_params.detection_scope.clone(),
        chimera_removed_definition: effective_params.chimera_removed_definition.clone(),
        input_reads: input_reads.display().to_string(),
        output_reads: output_reads.display().to_string(),
        chimera_metrics_json: chimera_metrics_json.display().to_string(),
        chimeras_fasta: chimeras_fasta.map(|path| path.display().to_string()),
        uchime_report_tsv: uchime_report_tsv.map(|path| path.display().to_string()),
        reads_in: Some(reads_in),
        reads_out: Some(reads_out),
        chimeras_removed: Some(chimeras_removed),
        chimera_fraction: Some(chimera_fraction),
        used_fallback,
        raw_backend_report: uchime_report_tsv.map(|path| path.display().to_string()),
        raw_backend_report_format: uchime_report_tsv
            .map(|_| effective_params.raw_backend_report_format.clone()),
        runtime_s: Some(runtime_s),
        memory_mb: Some(memory_mb),
        exit_code: Some(exit_code),
        backend_metrics: parse_uchime_summary(uchime_report_tsv),
    }
}

fn compatibility_metrics_from_report(report: &RemoveChimerasReportV1) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "bijux.fastq.remove_chimeras.v2",
        "chimera_fraction": report.chimera_fraction.unwrap_or(0.0),
        "chimeras_removed": report.chimeras_removed.unwrap_or(0),
        "non_chimera_reads": report.reads_out.unwrap_or(0),
        "tool": report.tool_id,
        "used_fallback": report.used_fallback,
    })
}
