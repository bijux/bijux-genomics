use std::collections::HashMap;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_workspace_registry};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_cluster_otus_v1, insert_fastq_cluster_otus_v1,
};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqClusterOtusMetrics};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_core::prelude::params_hash;
use bijux_dna_domain_fastq::params::edna::OtuClusteringEffectiveParams;
use bijux_dna_domain_fastq::{ClusterOtusReportV1, CLUSTER_OTUS_REPORT_SCHEMA_VERSION};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::stage_api::{
    bench_dir_name, inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind,
    RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use uuid::Uuid;

use crate::internal::fastq::stages::preprocess::materialize_amplicon_stage_outputs_for_bench;
use crate::internal::fastq::stages::trim_bench_common::build_benchmark_context;
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};

const STAGE_ID: &str = "fastq.cluster_otus";

#[derive(Debug, Clone, Copy)]
pub(crate) struct ClusterOtusTableMetrics {
    pub otu_count: u64,
    pub sample_count: u64,
}

pub(crate) fn cluster_otus_options_from_args(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqClusterOtusArgs,
) -> bijux_dna_planner_fastq::ClusterOtusStageParams {
    let mut options = bijux_dna_planner_fastq::ClusterOtusStageParams::default();
    if let Some(otu_identity) = args.otu_identity {
        options.otu_identity = otu_identity;
    }
    options.threads = args.threads;
    options
}

pub(crate) fn read_cluster_otus_table_metrics(
    path: &std::path::Path,
) -> Result<ClusterOtusTableMetrics> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut samples = std::collections::BTreeSet::new();
    let mut otus = std::collections::BTreeSet::new();
    for line in raw.lines().skip(1) {
        let mut parts = line.split('\t');
        let Some(sample_id) = parts.next().map(str::trim) else {
            continue;
        };
        let Some(otu_id) = parts.next().map(str::trim) else {
            continue;
        };
        if !sample_id.is_empty() {
            samples.insert(sample_id.to_string());
        }
        if !otu_id.is_empty() {
            otus.insert(otu_id.to_string());
        }
    }
    Ok(ClusterOtusTableMetrics {
        otu_count: otus.len() as u64,
        sample_count: samples.len() as u64,
    })
}

pub(crate) fn count_cluster_otus_representatives(path: &std::path::Path) -> Result<u64> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(raw
        .lines()
        .filter(|line| line.trim_start().starts_with('>'))
        .count() as u64)
}

pub(crate) fn canonical_cluster_otus_report(
    tool_id: &str,
    input_reads: &std::path::Path,
    otu_table: &std::path::Path,
    otu_representatives: &std::path::Path,
    taxonomy_ready_fasta: &std::path::Path,
    taxonomy_ready_fastq: &std::path::Path,
    report_json: &std::path::Path,
    effective_params: &OtuClusteringEffectiveParams,
    table_metrics: ClusterOtusTableMetrics,
    representative_sequence_count: u64,
    runtime_s: Option<f64>,
    memory_mb: Option<f64>,
    exit_code: Option<i32>,
    used_fallback: bool,
    raw_backend_report: Option<&std::path::Path>,
    backend_metrics: Option<serde_json::Value>,
) -> ClusterOtusReportV1 {
    ClusterOtusReportV1 {
        schema_version: CLUSTER_OTUS_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_ID.to_string(),
        stage_id: STAGE_ID.to_string(),
        tool_id: tool_id.to_string(),
        otu_identity: effective_params.identity_threshold,
        threads: effective_params.threads,
        input_reads: input_reads.display().to_string(),
        otu_table: otu_table.display().to_string(),
        otu_representatives: otu_representatives.display().to_string(),
        taxonomy_ready_fasta: taxonomy_ready_fasta.display().to_string(),
        taxonomy_ready_fastq: taxonomy_ready_fastq.display().to_string(),
        report_json: report_json.display().to_string(),
        otu_count: table_metrics.otu_count,
        sample_count: table_metrics.sample_count,
        representative_sequence_count,
        output_table_kind: effective_params.output_table_kind.clone(),
        used_fallback,
        runtime_s,
        memory_mb,
        exit_code,
        raw_backend_report: raw_backend_report.map(|path| path.display().to_string()),
        raw_backend_report_format: effective_params.raw_backend_report_format.clone(),
        backend_metrics,
    }
}

pub fn bench_fastq_cluster_otus<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqClusterOtusArgs,
) -> Result<BenchOutcome<FastqClusterOtusMetrics>> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = bijux_dna_planner_fastq::select_cluster_otus_tools(&args.tools)?;
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
    let input_hash = if let Some(r2) = args.r2.as_deref() {
        format!(
            "{}+{}",
            hash_file_sha256(&args.r1).context("hash cluster otus input r1")?,
            hash_file_sha256(r2).context("hash cluster otus input r2")?
        )
    } else {
        hash_file_sha256(&args.r1).context("hash cluster otus input")?
    };
    let bench_dir_name = bench_dir_name(&bijux_dna_domain_fastq::stages::ids::STAGE_CLUSTER_OTUS)
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
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = bijux_dna_planner_fastq::tool_adapters::fastq::cluster_otus::plan_with_options(
            &tool_spec,
            &args.r1,
            args.r2.as_deref(),
            &out_dir,
            &cluster_otus_options_from_args(args),
        )?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_cluster_otus_v1(
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
        let otu_table = output_path(&plan, "otu_table")?;
        let otu_representatives = output_path(&plan, "otu_representatives")?;
        let taxonomy_ready_fasta = output_path(&plan, "taxonomy_ready_fasta")?;
        let taxonomy_ready_fastq = output_path(&plan, "taxonomy_ready_fastq")?;
        let report_json = output_path(&plan, "report_json")?;
        let table_metrics = read_cluster_otus_table_metrics(&otu_table)?;
        let representative_count = count_cluster_otus_representatives(&otu_representatives)?;
        let metrics = FastqClusterOtusMetrics {
            otu_count: payload
                .get("otu_count")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(table_metrics.otu_count),
            representative_count,
        };
        let metric_set = metric_set(metrics);
        let effective_params: OtuClusteringEffectiveParams =
            serde_json::from_value(plan.effective_params.clone())
                .context("parse cluster_otus effective params")?;
        let raw_backend_report = out_dir.join("otu_clusters.uc");
        let report = canonical_cluster_otus_report(
            tool,
            &args.r1,
            &otu_table,
            &otu_representatives,
            &taxonomy_ready_fasta,
            &taxonomy_ready_fastq,
            &report_json,
            &effective_params,
            table_metrics,
            representative_count,
            Some(execution.runtime_s),
            Some(execution.memory_mb),
            Some(execution.exit_code),
            payload
                .get("used_fallback")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            raw_backend_report
                .exists()
                .then_some(raw_backend_report.as_path()),
            Some(serde_json::json!({
                "tool_payload": payload,
            })),
        );
        bijux_dna_infra::atomic_write_json(&report_json, &report)?;
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
        insert_fastq_cluster_otus_v1(&conn, &record)?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir,
        explain: args.explain,
    })
}

fn output_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    output_id: &str,
) -> Result<std::path::PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| anyhow!("cluster_otus plan missing {output_id} output"))
}
