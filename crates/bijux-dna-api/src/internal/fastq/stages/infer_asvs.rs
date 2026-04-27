use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_infer_asvs_v1, insert_fastq_infer_asvs_v1,
};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqInferAsvsMetrics};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_domain_fastq::params::edna::AsvInferenceEffectiveParams;
use bijux_dna_domain_fastq::{
    execution_support_for_stage, ExecutionStatus, InferAsvsReportV1,
    INFER_ASVS_REPORT_SCHEMA_VERSION,
};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::stage_api::{
    bench_dir_name, inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind,
    RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;

use crate::internal::fastq::stages::preprocess::{
    enforce_amplicon_qc_thresholds_for_bench, materialize_amplicon_stage_outputs_for_bench,
};
use crate::internal::fastq::stages::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context,
};
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};

const STAGE_ID: &str = "fastq.infer_asvs";

#[derive(Debug, Clone, Copy)]
pub(crate) struct InferAsvsTableMetrics {
    pub asv_count: u64,
    pub sample_count: u64,
}

pub(crate) fn infer_asvs_options_from_args(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqInferAsvsArgs,
) -> bijux_dna_planner_fastq::InferAsvsStageParams {
    let mut options = bijux_dna_planner_fastq::InferAsvsStageParams::baseline();
    if let Some(denoising_method) = args.denoising_method.as_ref() {
        options.denoising_method.clone_from(denoising_method);
    }
    if let Some(pooling_mode) = args.pooling_mode.as_ref() {
        options.pooling_mode.clone_from(pooling_mode);
    }
    if let Some(chimera_policy) = args.chimera_policy.as_ref() {
        options.chimera_policy.clone_from(chimera_policy);
    }
    options.threads = args.threads;
    options
}

pub(crate) fn read_infer_asvs_table_metrics(path: &Path) -> Result<InferAsvsTableMetrics> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut samples = BTreeSet::new();
    let mut features = BTreeSet::new();
    for line in raw.lines().skip(1) {
        let mut parts = line.split('\t');
        let Some(sample_id) = parts.next().map(str::trim) else {
            continue;
        };
        let Some(feature_id) = parts.next().map(str::trim) else {
            continue;
        };
        if !sample_id.is_empty() {
            samples.insert(sample_id.to_string());
        }
        if !feature_id.is_empty() {
            features.insert(feature_id.to_string());
        }
    }
    Ok(InferAsvsTableMetrics {
        asv_count: features.len() as u64,
        sample_count: samples.len() as u64,
    })
}

pub(crate) fn count_fasta_records(path: &Path) -> Result<u64> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(raw.lines().filter(|line| line.trim_start().starts_with('>')).count() as u64)
}

pub(crate) struct InferAsvsReportInputs<'a> {
    pub tool_id: &'a str,
    pub input_r1: &'a Path,
    pub input_r2: Option<&'a Path>,
    pub asv_table_tsv: &'a Path,
    pub asv_sequences_fasta: &'a Path,
    pub taxonomy_reference_fasta: &'a Path,
    pub taxonomy_reads_fastq: &'a Path,
    pub report_json: &'a Path,
    pub effective_params: &'a AsvInferenceEffectiveParams,
    pub table_metrics: InferAsvsTableMetrics,
    pub representative_sequence_count: u64,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i32>,
    pub used_fallback: bool,
    pub backend_metrics: Option<serde_json::Value>,
}

struct InferAsvsOutputs {
    asv_table_tsv: PathBuf,
    asv_sequences_fasta: PathBuf,
    taxonomy_reference_fasta: PathBuf,
    taxonomy_reads_fastq: PathBuf,
    report_json: PathBuf,
}

pub(crate) fn canonical_infer_asvs_report(inputs: InferAsvsReportInputs<'_>) -> InferAsvsReportV1 {
    InferAsvsReportV1 {
        schema_version: INFER_ASVS_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_ID.to_string(),
        stage_id: STAGE_ID.to_string(),
        tool_id: inputs.tool_id.to_string(),
        paired_mode: inputs.effective_params.paired_mode,
        denoising_method: inputs.effective_params.denoising_method.clone(),
        pooling_mode: inputs.effective_params.pooling_mode.clone(),
        chimera_policy: inputs.effective_params.chimera_policy.clone(),
        requires_r_runtime: inputs.effective_params.requires_r_runtime,
        output_table_kind: inputs.effective_params.output_table_kind.clone(),
        input_reads_r1: inputs.input_r1.display().to_string(),
        input_reads_r2: inputs.input_r2.map(|path| path.display().to_string()),
        asv_table_tsv: inputs.asv_table_tsv.display().to_string(),
        asv_sequences_fasta: inputs.asv_sequences_fasta.display().to_string(),
        taxonomy_ready_fasta: inputs.taxonomy_reference_fasta.display().to_string(),
        taxonomy_ready_fastq: inputs.taxonomy_reads_fastq.display().to_string(),
        report_json: inputs.report_json.display().to_string(),
        asv_count: inputs.table_metrics.asv_count,
        sample_count: inputs.table_metrics.sample_count,
        representative_sequence_count: inputs.representative_sequence_count,
        used_fallback: inputs.used_fallback,
        raw_backend_report: Some(inputs.report_json.display().to_string()),
        raw_backend_report_format: Some("infer_asvs_governed_report_json".to_string()),
        runtime_s: inputs.runtime_s,
        memory_mb: inputs.memory_mb,
        exit_code: inputs.exit_code,
        backend_metrics: inputs.backend_metrics,
    }
}

/// Benchmark FASTQ ASV inference tools under governed execution contracts.
///
/// # Errors
/// Returns an error if planning, execution, artifact parsing, or persistence fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_infer_asvs<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqInferAsvsArgs,
) -> Result<BenchOutcome<FastqInferAsvsMetrics>> {
    match execution_support_for_stage(&bijux_dna_domain_fastq::stages::ids::STAGE_INFER_ASVS) {
        Some(support) if support.execution_status == ExecutionStatus::Closed => {}
        _ => {
            return Err(anyhow!("{STAGE_ID} has no admitted governed runtime backend"));
        }
    }
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = bijux_dna_planner_fastq::select_infer_asvs_tools(&args.tools)?;
    let tools = filter_tools_by_role(STAGE_ID, &tools, &registry, false)?;
    let runner = ensure_bench_runner(platform, runner_override)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_ID, artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_ID, &header);
    let input_hash = if let Some(r2) = args.r2.as_deref() {
        format!(
            "{}+{}",
            hash_file_sha256(&args.r1).context("hash infer asvs input r1")?,
            hash_file_sha256(r2).context("hash infer asvs input r2")?
        )
    } else {
        hash_file_sha256(&args.r1).context("hash infer asvs input")?
    };
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
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = bijux_dna_planner_fastq::tool_adapters::fastq::infer_asvs::plan_with_options(
            &tool_spec,
            &args.r1,
            args.r2.as_deref(),
            &out_dir,
            &infer_asvs_options_from_args(args),
        )?;
        let params_hash = stable_params_hash(&plan.params);
        let image_digest = benchmark_image_identity(&tool_spec);
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
            let stderr = execution.stderr.trim();
            let reason = if stderr.is_empty() {
                format!("tool {tool} failed with status {}", execution.exit_code)
            } else {
                format!("tool {tool} failed with status {}: {stderr}", execution.exit_code)
            };
            failures.push(RawFailure {
                stage: STAGE_ID.to_string(),
                tool: tool.clone(),
                reason,
                category: ErrorCategory::ToolError,
            });
            continue;
        }
        let payload = materialize_amplicon_stage_outputs_for_bench(&out_dir, &step)?;
        enforce_amplicon_qc_thresholds_for_bench(&out_dir, STAGE_ID, &payload)?;
        let outputs = resolve_infer_asvs_outputs(&plan)?;
        let table_metrics = read_infer_asvs_table_metrics(&outputs.asv_table_tsv)?;
        let representative_sequence_count = count_fasta_records(&outputs.asv_sequences_fasta)?;
        let metrics = FastqInferAsvsMetrics {
            asv_count: table_metrics.asv_count,
            sample_count: table_metrics.sample_count,
        };
        let metric_set = metric_set(metrics);
        let effective_params: AsvInferenceEffectiveParams =
            serde_json::from_value(plan.effective_params.clone())
                .context("parse infer_asvs effective params")?;
        let used_fallback = infer_asvs_used_fallback(&payload)?;
        let report = canonical_infer_asvs_report(InferAsvsReportInputs {
            tool_id: tool,
            input_r1: &args.r1,
            input_r2: args.r2.as_deref(),
            asv_table_tsv: &outputs.asv_table_tsv,
            asv_sequences_fasta: &outputs.asv_sequences_fasta,
            taxonomy_reference_fasta: &outputs.taxonomy_reference_fasta,
            taxonomy_reads_fastq: &outputs.taxonomy_reads_fastq,
            report_json: &outputs.report_json,
            effective_params: &effective_params,
            table_metrics,
            representative_sequence_count,
            runtime_s: Some(execution.runtime_s),
            memory_mb: Some(execution.memory_mb),
            exit_code: Some(execution.exit_code),
            used_fallback,
            backend_metrics: Some(payload),
        });
        validate_infer_asvs_report_identity(tool, &report)?;
        validate_infer_asvs_report_metrics(&report, &metric_set.metrics)?;
        validate_infer_asvs_report_execution(
            &report,
            execution.runtime_s,
            execution.memory_mb,
            execution.exit_code,
        )?;
        bijux_dna_infra::atomic_write_json(&outputs.report_json, &report)?;
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
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_infer_asvs_v1(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome { records, failures, bench_dir, explain: args.explain })
}

fn output_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    artifact_name: &str,
) -> Result<PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_name)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| anyhow!("infer_asvs plan missing output artifact `{artifact_name}`"))
}

fn resolve_infer_asvs_outputs(
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<InferAsvsOutputs> {
    let outputs = InferAsvsOutputs {
        asv_table_tsv: output_path(plan, "asv_table_tsv")?,
        asv_sequences_fasta: output_path(plan, "asv_sequences_fasta")?,
        taxonomy_reference_fasta: output_path(plan, "taxonomy_ready_fasta")?,
        taxonomy_reads_fastq: output_path(plan, "taxonomy_ready_fastq")?,
        report_json: output_path(plan, "report_json")?,
    };
    validate_infer_asvs_output_paths(&outputs)?;
    Ok(outputs)
}

fn validate_infer_asvs_output_paths(outputs: &InferAsvsOutputs) -> Result<()> {
    let mut paths = BTreeSet::new();
    for path in [
        outputs.asv_table_tsv.as_path(),
        outputs.asv_sequences_fasta.as_path(),
        outputs.taxonomy_reference_fasta.as_path(),
        outputs.taxonomy_reads_fastq.as_path(),
        outputs.report_json.as_path(),
    ] {
        if !paths.insert(path) {
            return Err(anyhow!("infer_asvs output path reused: {}", path.display()));
        }
    }
    Ok(())
}

fn infer_asvs_used_fallback(payload: &serde_json::Value) -> Result<bool> {
    payload
        .get("used_fallback")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("infer_asvs payload missing boolean used_fallback"))
}

fn validate_infer_asvs_report_identity(tool: &str, report: &InferAsvsReportV1) -> Result<()> {
    if report.schema_version != INFER_ASVS_REPORT_SCHEMA_VERSION {
        return Err(anyhow!(
            "infer_asvs report schema mismatch: expected {}, observed {}",
            INFER_ASVS_REPORT_SCHEMA_VERSION,
            report.schema_version
        ));
    }
    if report.stage != STAGE_ID || report.stage_id != STAGE_ID {
        return Err(anyhow!(
            "infer_asvs report stage mismatch: observed stage={} stage_id={}",
            report.stage,
            report.stage_id
        ));
    }
    if report.tool_id != tool {
        return Err(anyhow!(
            "infer_asvs report tool mismatch: expected {}, observed {}",
            tool,
            report.tool_id
        ));
    }
    Ok(())
}

fn validate_infer_asvs_report_metrics(
    report: &InferAsvsReportV1,
    metrics: &FastqInferAsvsMetrics,
) -> Result<()> {
    if report.asv_count != metrics.asv_count {
        return Err(anyhow!(
            "infer_asvs report asv_count mismatch: expected {}, observed {}",
            metrics.asv_count,
            report.asv_count
        ));
    }
    if report.sample_count != metrics.sample_count {
        return Err(anyhow!(
            "infer_asvs report sample_count mismatch: expected {}, observed {}",
            metrics.sample_count,
            report.sample_count
        ));
    }
    Ok(())
}

fn validate_infer_asvs_report_execution(
    report: &InferAsvsReportV1,
    runtime_s: f64,
    memory_mb: f64,
    exit_code: i32,
) -> Result<()> {
    if report.runtime_s.is_none_or(|observed| (observed - runtime_s).abs() > f64::EPSILON) {
        return Err(anyhow!(
            "infer_asvs report runtime mismatch: expected {}, observed {:?}",
            runtime_s,
            report.runtime_s
        ));
    }
    if report.memory_mb.is_none_or(|observed| (observed - memory_mb).abs() > f64::EPSILON) {
        return Err(anyhow!(
            "infer_asvs report memory mismatch: expected {}, observed {:?}",
            memory_mb,
            report.memory_mb
        ));
    }
    if report.exit_code != Some(exit_code) {
        return Err(anyhow!(
            "infer_asvs report exit code mismatch: expected {}, observed {:?}",
            exit_code,
            report.exit_code
        ));
    }
    Ok(())
}
