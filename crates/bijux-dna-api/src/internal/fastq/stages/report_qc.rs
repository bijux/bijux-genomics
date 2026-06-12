use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::internal::fastq::stages::trim_bench_common::benchmark_image_identity;
use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_REPORT_QC,
};
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::fetch_fastq_qc_post_v1;
use bijux_dna_analyze::quality::insert_fastq_qc_post_v1;
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkContext, BenchmarkRecord, FastqQcPostMetrics,
};
use bijux_dna_core::contract::ArtifactRole;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::{ArtifactRef, ContainerImageRefV1};
use bijux_dna_domain_fastq::params::{
    qc_post::{QcAggregationEngine, QcAggregationScope},
    PairedMode,
};
use bijux_dna_domain_fastq::{
    observer::{parse_detect_adapters_report, parse_screen_taxonomy_report},
    GovernedQcContributorV1, GovernedQcInputsManifestV1, GovernedQcManifestContributorV1,
    ReportQcReportV1, GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION, REPORT_QC_REPORT_SCHEMA_VERSION,
};
use bijux_dna_environment::api::{resolve_image, PlatformSpec, RuntimeKind, ToolImageCatalog};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::select_qc_post_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::report_qc::plan_qc_post_with_qc_inputs;
use bijux_dna_planner_fastq::stage_api::observer::{
    input_fastq_stats, parse_multiqc_general_stats_metrics, parse_seqkit_stats,
};
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::backend::docker::executor::resolve_image_for_run;
use bijux_dna_runner::step_runner::{execute_observer_command, StageResultV1};

mod governed_inputs;

#[cfg(test)]
use self::governed_inputs::{
    derived_governed_qc_lineage_hash, governed_qc_contributors, load_governed_qc_inputs_manifest,
    validate_governed_qc_contributors,
};
use self::governed_inputs::{
    governed_qc_contributor_stage_ids, governed_qc_contributor_tool_ids,
    governed_qc_inputs_manifest_path, load_required_qc_inputs_manifest,
    resolve_qc_contributor_aux_images,
};

fn parse_qc_aggregation_scope(value: Option<&str>) -> Result<QcAggregationScope> {
    match value.unwrap_or("governed_qc_artifacts") {
        "governed_qc_artifacts" => Ok(QcAggregationScope::GovernedQcArtifacts),
        "fastq_qc_inputs" => Err(anyhow!(
            "unsupported fastq.report_qc aggregation_scope `fastq_qc_inputs`; native upstream-stage aggregation is not implemented yet, use governed_qc_artifacts"
        )),
        other => Err(anyhow!(
            "unsupported fastq.report_qc aggregation_scope `{other}`; expected: governed_qc_artifacts"
        )),
    }
}

fn parse_qc_aggregation_engine(value: Option<&str>) -> Result<QcAggregationEngine> {
    match value.unwrap_or("auto") {
        "auto" | "multiqc" => Ok(QcAggregationEngine::Multiqc),
        other => Err(anyhow!(
            "unsupported fastq.report_qc aggregation_engine `{other}`; expected one of: auto, multiqc"
        )),
    }
}

/// # Errors
/// Returns an error if planning or execution fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_qc_post(
    catalog: &impl ToolImageCatalog,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqQcPostArgs,
) -> Result<BenchOutcome<bijux_dna_analyze::FastqQcPostMetrics>> {
    let aggregation_engine = parse_qc_aggregation_engine(args.aggregation_engine.as_deref())?;
    let aggregation_scope = parse_qc_aggregation_scope(args.aggregation_scope.as_deref())?;
    let tools = select_qc_post_tools(&args.tools)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_REPORT_QC.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_REPORT_QC.as_str(), &header);

    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_REPORT_QC.as_str(), &tools, &registry, false)?;
    let bench_inputs = prepare_qc_post_bench(catalog, platform, runner_override, args)?;
    let governed_qc = load_required_qc_inputs_manifest(
        &aggregation_scope,
        args.governed_qc_manifest.as_deref(),
        &bench_inputs.bench_dir,
        &bench_inputs.tools_root,
        &tools,
    )?;
    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_REPORT_QC.as_str());
    let all_tools: Vec<String> =
        registry.tools_for_stage(&stage_id).iter().map(|tool| tool.tool_id.to_string()).collect();
    let excluded: Vec<String> =
        all_tools.into_iter().filter(|tool| !tools.contains(tool)).collect();

    if args.explain {
        write_explain_md(
            &bench_inputs.bench_dir,
            STAGE_REPORT_QC.as_str(),
            &tools,
            &excluded,
            None,
        )?;
        write_explain_plan_json(
            &bench_inputs.bench_dir,
            STAGE_REPORT_QC.as_str(),
            &tools,
            &registry,
            None,
        )?;
    }

    ensure_image_qa_passed(STAGE_REPORT_QC.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_REPORT_QC.as_str(), &tools, platform, catalog)?;
    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");

    let jobs = bench_jobs(args.jobs);

    let mut failures = Vec::<RawFailure>::new();
    let mut records = Vec::<BenchmarkRecord<FastqQcPostMetrics>>::new();
    for tool in &tools {
        let out_dir = bench_inputs.tools_root.join(tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec = build_tool_execution_spec(
            STAGE_REPORT_QC.as_str(),
            tool,
            &registry,
            catalog,
            platform,
        )?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let aux_images = resolve_qc_contributor_aux_images(catalog, platform, &governed_qc)?;
        let plan = plan_qc_post_with_qc_inputs(
            &tool_spec,
            &governed_qc.qc_inputs,
            &out_dir,
            aux_images,
            paired_mode_for_bench_inputs(&bench_inputs),
            aggregation_engine.clone(),
            aggregation_scope.clone(),
            Some(&bench_inputs.r1),
            bench_inputs.r2.as_deref(),
        )?;
        let bench_params = benchmark_query_context(governed_qc.lineage_hash.as_deref())?
            .embed_in_parameters(&plan.params);
        let params_hash = stable_params_hash(&bench_params);
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_qc_post_v1(
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
            vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&plan)],
            bench_inputs.runner,
            jobs,
        )?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing execution result for {tool}"))?;
        let record = build_qc_post_record(
            platform,
            &bench_inputs,
            tool,
            &tool_spec,
            &bench_params,
            &governed_qc,
            &out_dir,
            &execution,
        )?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_qc_post_v1(&conn, &record).context("insert bench sqlite")?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: STAGE_REPORT_QC.as_str().to_string(),
                tool: tool.clone(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
                category: ErrorCategory::ToolError,
            });
        }
        records.push(record);
    }

    Ok(BenchOutcome { records, failures, bench_dir: bench_inputs.bench_dir, explain: args.explain })
}

#[derive(Debug, Clone)]
struct QcPostBenchInputs {
    runner: RuntimeKind,
    r1: PathBuf,
    r2: Option<PathBuf>,
    input_hash: String,
    input_stats: SeqkitMetrics,
    input_stats_r2: Option<SeqkitMetrics>,
    bench_dir: PathBuf,
    tools_root: PathBuf,
}

#[derive(Debug, Clone)]
struct GovernedQcInputs {
    qc_inputs: Vec<ArtifactRef>,
    contributors: Vec<GovernedQcContributor>,
    raw_fastqc_dir: Option<PathBuf>,
    lineage_hash: Option<String>,
}

type GovernedQcContributor = GovernedQcManifestContributorV1;
type GovernedQcInputsManifest = GovernedQcInputsManifestV1;

fn prepare_qc_post_bench(
    catalog: &impl ToolImageCatalog,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqQcPostArgs,
) -> Result<QcPostBenchInputs> {
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir_name = bench_dir_name(&STAGE_REPORT_QC)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_REPORT_QC.as_str()))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_dna_infra::ensure_dir(&tools_root).context("create tools output dir")?;

    let r1 = args.r1.canonicalize().context("resolve r1 path")?;
    let r1_dir = r1.parent().ok_or_else(|| anyhow!("r1 has no parent"))?.to_path_buf();

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
        return Err(anyhow!("seqkit qc observer failed: {}", stats_output.stderr));
    }

    let (r2, input_stats_r2) = if let Some(r2) = args.r2.as_deref() {
        let r2 = r2.canonicalize().context("resolve r2 path")?;
        let r2_dir = r2.parent().ok_or_else(|| anyhow!("r2 has no parent"))?.to_path_buf();
        let stats_spec = input_fastq_stats(&r2_dir, &r2)?;
        let stats_output = execute_observer_command(
            &seqkit_image.full_name,
            stats_spec.mount_dir.as_path(),
            &stats_spec.args,
            runner,
        )?;
        if stats_output.exit_code != 0 {
            return Err(anyhow!("seqkit qc observer failed for r2: {}", stats_output.stderr));
        }
        (Some(r2), Some(parse_seqkit_stats(&stats_output.stdout)?))
    } else {
        (None, None)
    };

    Ok(QcPostBenchInputs {
        runner,
        input_hash: if let Some(r2) = r2.as_ref() {
            format!(
                "{}+{}",
                hash_file_sha256(&r1).context("hash qc input r1")?,
                hash_file_sha256(r2).context("hash qc input r2")?
            )
        } else {
            hash_file_sha256(&r1).context("hash qc input")?
        },
        input_stats: parse_seqkit_stats(&stats_output.stdout)?,
        input_stats_r2,
        r1,
        r2,
        bench_dir,
        tools_root,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_qc_post_record(
    platform: &PlatformSpec,
    bench_inputs: &QcPostBenchInputs,
    tool: &str,
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    params: &serde_json::Value,
    governed_qc: &GovernedQcInputs,
    out_dir: &Path,
    execution: &StageResultV1,
) -> Result<BenchmarkRecord<FastqQcPostMetrics>> {
    let metrics = derive_qc_post_metrics(
        &bench_inputs.input_stats,
        bench_inputs.input_stats_r2.as_ref(),
        out_dir,
        governed_qc.raw_fastqc_dir.as_deref(),
        params,
        governed_qc,
    );
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;
    let governed_qc_manifest = governed_qc_inputs_manifest_path(out_dir);
    let report_path = out_dir.join("report_qc_report.json");
    let report = build_governed_qc_post_report(
        tool,
        params,
        &metrics,
        governed_qc,
        &governed_qc_manifest,
        &report_path,
        execution,
    )?;
    bijux_dna_infra::atomic_write_json(&report_path, &report).context("write qc report")?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write qc metrics")?;

    let context = BenchmarkContext {
        tool: tool.to_string(),
        tool_version: tool_spec.tool_version.clone(),
        image_digest: benchmark_image_identity(tool_spec),
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

fn build_governed_qc_post_report(
    tool: &str,
    params: &serde_json::Value,
    metrics: &FastqQcPostMetrics,
    governed_qc: &GovernedQcInputs,
    governed_qc_manifest: &Path,
    report_path: &Path,
    execution: &StageResultV1,
) -> Result<ReportQcReportV1> {
    let paired_mode =
        if params.get("raw_r2").is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd };
    let aggregation_engine = parse_qc_aggregation_engine(
        params.get("aggregation_engine").and_then(serde_json::Value::as_str),
    )?;
    let aggregation_scope = parse_qc_aggregation_scope(
        params.get("aggregation_scope").and_then(serde_json::Value::as_str),
    )?;
    let governed_summary = load_governed_qc_summary(governed_qc);
    let contributors = governed_qc
        .contributors
        .iter()
        .map(|contributor| GovernedQcContributorV1 {
            contributor_id: contributor.contributor_id.clone(),
            stage_id: contributor.stage_id.clone(),
            tool_id: contributor.tool_id.clone(),
            artifact_id: contributor.artifact_id.clone(),
            artifact_role: contributor.artifact_role.as_str().to_string(),
            path: contributor.path.display().to_string(),
        })
        .collect::<Vec<_>>();
    let multiqc_metrics =
        load_multiqc_general_stats(report_path.parent().unwrap_or_else(|| Path::new(".")));
    Ok(ReportQcReportV1 {
        schema_version: REPORT_QC_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_REPORT_QC.as_str().to_string(),
        stage_id: STAGE_REPORT_QC.as_str().to_string(),
        tool_id: tool.to_string(),
        paired_mode,
        aggregation_engine,
        aggregation_scope,
        reads_in: metrics.reads_in,
        reads_out: metrics.reads_out,
        bases_in: metrics.bases_in,
        bases_out: metrics.bases_out,
        pairs_in: metrics.pairs_in,
        pairs_out: metrics.pairs_out,
        mean_q: metrics.mean_q,
        contamination_rate: governed_summary
            .contamination_rate
            .unwrap_or(metrics.contamination_rate),
        adapter_content_max: governed_summary.adapter_content_max,
        adapter_content_mean: governed_summary.adapter_content_mean,
        duplication_rate: governed_summary.duplication_rate,
        n_rate: governed_summary.n_rate,
        kmer_warning_count: governed_summary.kmer_warning_count,
        overrepresented_sequence_count: governed_summary.overrepresented_sequence_count,
        multiqc_sample_count: multiqc_metrics.as_ref().map(|metrics| metrics.sample_count),
        multiqc_module_count: multiqc_metrics.as_ref().map(|metrics| metrics.module_count),
        raw_fastqc_dir: metrics.raw_fastqc_dir.clone(),
        trimmed_fastqc_dir: metrics.trimmed_fastqc_dir.clone(),
        multiqc_report: metrics.multiqc_report.clone(),
        multiqc_data: metrics.multiqc_data.clone(),
        governed_qc_input_count: governed_qc.qc_inputs.len() as u64,
        governed_qc_contributor_stage_ids: governed_qc_contributor_stage_ids(
            &governed_qc.contributors,
        ),
        governed_qc_contributor_tool_ids: governed_qc_contributor_tool_ids(
            &governed_qc.contributors,
        ),
        governed_qc_contributors: contributors,
        governed_qc_lineage_hash: governed_qc.lineage_hash.clone(),
        governed_qc_inputs_manifest: path_if_exists(governed_qc_manifest),
        runtime_s: Some(execution.runtime_s),
        memory_mb: Some(execution.memory_mb),
        exit_code: Some(execution.exit_code),
    })
}

fn load_multiqc_general_stats(
    out_dir: &Path,
) -> Option<bijux_dna_domain_fastq::metrics::MultiqcToolMetricsV1> {
    let path = out_dir.join("multiqc_data").join("multiqc_general_stats.json");
    let raw = std::fs::read_to_string(path).ok()?;
    parse_multiqc_general_stats_metrics(&raw).ok()
}

fn derive_qc_post_metrics(
    input_stats: &SeqkitMetrics,
    input_stats_r2: Option<&SeqkitMetrics>,
    out_dir: &Path,
    raw_fastqc_dir: Option<&Path>,
    params: &serde_json::Value,
    governed_qc: &GovernedQcInputs,
) -> FastqQcPostMetrics {
    let governed_summary = load_governed_qc_summary(governed_qc);
    let multiqc_report = out_dir.join("multiqc_report.html");
    let multiqc_data = out_dir.join("multiqc_data");
    let trimmed_fastqc_dir = out_dir.join("fastqc_trimmed");
    let reads_in = input_stats.reads + input_stats_r2.map_or(0, |stats| stats.reads);
    let bases_in = input_stats.bases + input_stats_r2.map_or(0, |stats| stats.bases);
    let weighted_q_sum = input_stats.mean_q * u64_to_f64(input_stats.bases)
        + input_stats_r2.map_or(0.0, |stats| stats.mean_q * u64_to_f64(stats.bases));
    let mean_q = if bases_in == 0 { 0.0 } else { weighted_q_sum / u64_to_f64(bases_in) };
    FastqQcPostMetrics {
        reads_in,
        reads_out: reads_in,
        bases_in,
        bases_out: bases_in,
        pairs_in: input_stats_r2.map(|stats| input_stats.reads.min(stats.reads)),
        pairs_out: input_stats_r2.map(|stats| input_stats.reads.min(stats.reads)),
        mean_q,
        contamination_rate: governed_summary.contamination_rate.unwrap_or(0.0),
        aggregation_engine: params
            .get("aggregation_engine")
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string),
        aggregation_scope: params
            .get("aggregation_scope")
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string),
        governed_qc_input_count: Some(governed_qc.qc_inputs.len() as u64),
        governed_qc_contributor_stage_ids: serde_json::json!(governed_qc_contributor_stage_ids(
            &governed_qc.contributors
        ))
        .into(),
        governed_qc_contributor_tool_ids: serde_json::json!(governed_qc_contributor_tool_ids(
            &governed_qc.contributors
        ))
        .into(),
        governed_qc_lineage_hash: governed_qc.lineage_hash.clone(),
        multiqc_sample_count: load_multiqc_general_stats(out_dir)
            .map(|metrics| metrics.sample_count),
        multiqc_module_count: load_multiqc_general_stats(out_dir)
            .map(|metrics| metrics.module_count),
        raw_fastqc_dir: raw_fastqc_dir.and_then(path_if_exists),
        trimmed_fastqc_dir: path_if_exists(&trimmed_fastqc_dir),
        multiqc_report: path_if_exists(&multiqc_report),
        multiqc_data: path_if_exists(&multiqc_data),
    }
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

#[derive(Debug, Default, Clone, Copy)]
struct GovernedQcSummary {
    contamination_rate: Option<f64>,
    adapter_content_max: Option<f64>,
    adapter_content_mean: Option<f64>,
    duplication_rate: Option<f64>,
    n_rate: Option<f64>,
    kmer_warning_count: Option<u64>,
    overrepresented_sequence_count: Option<u64>,
}

fn load_governed_qc_summary(governed_qc: &GovernedQcInputs) -> GovernedQcSummary {
    let mut summary = GovernedQcSummary::default();
    if let Some(path) = governed_qc_report_path(
        governed_qc,
        "fastq.screen_taxonomy",
        "report_json",
        ArtifactRole::ReportJson,
    ) {
        if let Ok(raw) = std::fs::read_to_string(path) {
            if let Ok(report) = parse_screen_taxonomy_report(&raw) {
                summary.contamination_rate = report.contamination_rate;
            }
        }
    }
    if let Some(path) = governed_qc_report_path(
        governed_qc,
        "fastq.detect_adapters",
        "report_json",
        ArtifactRole::ReportJson,
    ) {
        if let Ok(raw) = std::fs::read_to_string(path) {
            if let Ok(report) = parse_detect_adapters_report(&raw) {
                summary.adapter_content_max = report.adapter_content_max;
                summary.adapter_content_mean = report.adapter_content_mean;
                summary.duplication_rate = report.duplication_rate;
                summary.n_rate = report.n_rate;
                summary.kmer_warning_count = report.kmer_warning_count;
                summary.overrepresented_sequence_count = report.overrepresented_sequence_count;
            }
        }
    }
    summary
}

fn governed_qc_report_path<'a>(
    governed_qc: &'a GovernedQcInputs,
    stage_id: &str,
    artifact_id: &str,
    artifact_role: ArtifactRole,
) -> Option<&'a Path> {
    governed_qc
        .contributors
        .iter()
        .find(|contributor| {
            contributor.stage_id == stage_id
                && contributor.artifact_id == artifact_id
                && contributor.artifact_role == artifact_role
        })
        .map(|contributor| contributor.path.as_path())
}

fn path_if_exists(path: &Path) -> Option<String> {
    path.exists().then(|| path.display().to_string())
}

fn paired_mode_for_bench_inputs(bench_inputs: &QcPostBenchInputs) -> PairedMode {
    if bench_inputs.r2.is_some() {
        PairedMode::PairedEnd
    } else {
        PairedMode::SingleEnd
    }
}

fn benchmark_query_context(
    lineage_hash: Option<&str>,
) -> Result<bijux_dna_domain_fastq::BenchQueryContext> {
    let mut context =
        bijux_dna_domain_fastq::governed_stage_bench_query_context(STAGE_REPORT_QC.as_str())?;
    if let Some(lineage_hash) = lineage_hash {
        context = context.with_lineage_hash(lineage_hash.to_string());
    }
    Ok(context)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::{
        build_qc_post_record, derive_qc_post_metrics, derived_governed_qc_lineage_hash,
        governed_qc_contributor_stage_ids, governed_qc_contributor_tool_ids,
        governed_qc_contributors, governed_qc_inputs_manifest_path,
        load_governed_qc_inputs_manifest, load_required_qc_inputs_manifest,
        parse_qc_aggregation_engine, parse_qc_aggregation_scope, resolve_qc_contributor_aux_images,
        validate_governed_qc_contributors, GovernedQcContributor, GovernedQcInputs,
        GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION,
    };
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use bijux_dna_core::contract::{ArtifactRole, ToolConstraints};
    use bijux_dna_core::ids::{ArtifactId, ToolId};
    use bijux_dna_core::prelude::measure::SeqkitMetrics;
    use bijux_dna_core::prelude::{
        ArtifactRef, CommandSpecV1, ContainerImageRefV1, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_fastq::params::qc_post::{QcAggregationEngine, QcAggregationScope};
    use bijux_dna_domain_fastq::ReportQcReportV1;
    use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
    use bijux_dna_runner::step_runner::StageResultV1;

    #[test]
    fn required_governed_qc_manifest_is_enforced() {
        let temp = tempfile::tempdir().expect("tempdir");
        let error = load_required_qc_inputs_manifest(
            &QcAggregationScope::GovernedQcArtifacts,
            None,
            temp.path(),
            temp.path(),
            &["multiqc".to_string()],
        )
        .expect_err("manifest requirement must be enforced");
        assert!(error.to_string().contains("requires --governed-qc-manifest"));
    }

    #[test]
    fn fastq_qc_input_scope_is_rejected_until_native_aggregation_exists() {
        let error = parse_qc_aggregation_scope(Some("fastq_qc_inputs"))
            .expect_err("unsupported scope must fail fast");
        assert!(error.to_string().contains("not implemented yet"));
    }

    #[test]
    fn report_qc_manifest_loader_discovers_planner_written_tool_manifest() {
        let temp = tempfile::tempdir().expect("tempdir");
        let tools_root = temp.path().join("tools");
        let tool_dir = tools_root.join("multiqc");
        bijux_dna_infra::ensure_dir(&tool_dir).expect("tool dir");
        let artifact_path = temp.path().join("trim_report.json");
        bijux_dna_infra::write_bytes(&artifact_path, b"{}").expect("artifact");
        let manifest_path = governed_qc_inputs_manifest_path(&tool_dir);
        bijux_dna_infra::write_bytes(
            &manifest_path,
            serde_json::json!({
                "schema_version": GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION,
                "qc_inputs": [
                    {
                        "name": "fastq.trim_reads.fastp.report_json",
                        "path": artifact_path,
                        "role": "report_json",
                        "optional": false
                    }
                ]
            })
            .to_string(),
        )
        .expect("manifest");

        let governed = load_required_qc_inputs_manifest(
            &QcAggregationScope::GovernedQcArtifacts,
            None,
            temp.path(),
            &tools_root,
            &["multiqc".to_string()],
        )
        .expect("planner-written manifest should be discovered");

        assert_eq!(governed.qc_inputs.len(), 1);
        assert_eq!(governed.qc_inputs[0].path, artifact_path);
    }

    #[test]
    fn qc_post_scope_parser_defaults_to_governed_artifacts() {
        assert_eq!(
            parse_qc_aggregation_scope(None).expect("default scope"),
            QcAggregationScope::GovernedQcArtifacts
        );
        assert_eq!(
            parse_qc_aggregation_scope(Some("governed_qc_artifacts")).expect("explicit scope"),
            QcAggregationScope::GovernedQcArtifacts
        );
        assert!(parse_qc_aggregation_scope(Some("fastq_qc_inputs")).is_err());
    }

    #[test]
    fn qc_post_engine_parser_defaults_to_multiqc() {
        assert_eq!(
            parse_qc_aggregation_engine(None).expect("default engine"),
            QcAggregationEngine::Multiqc
        );
        assert_eq!(
            parse_qc_aggregation_engine(Some("auto")).expect("auto engine"),
            QcAggregationEngine::Multiqc
        );
        assert_eq!(
            parse_qc_aggregation_engine(Some("multiqc")).expect("explicit engine"),
            QcAggregationEngine::Multiqc
        );
    }

    #[test]
    fn qc_post_metrics_report_governed_fastqc_dir_when_present() {
        let temp = tempfile::tempdir().expect("tempdir");
        let multiqc_data = temp.path().join("multiqc_data");
        bijux_dna_infra::ensure_dir(&multiqc_data).expect("multiqc data dir");
        bijux_dna_infra::write_bytes(temp.path().join("multiqc_report.html"), b"report")
            .expect("report");
        let raw_fastqc_dir = temp.path().join("governed_qc_inputs/detect_adapters/fastqc/fastqc");
        bijux_dna_infra::ensure_dir(&raw_fastqc_dir).expect("fastqc dir");

        let metrics = derive_qc_post_metrics(
            &SeqkitMetrics { reads: 10, bases: 100, mean_q: 30.0, gc_percent: 50.0 },
            None,
            temp.path(),
            Some(raw_fastqc_dir.as_path()),
            &serde_json::json!({
                "aggregation_engine": "multiqc",
                "aggregation_scope": "governed_qc_artifacts"
            }),
            &GovernedQcInputs {
                qc_inputs: Vec::new(),
                contributors: Vec::new(),
                raw_fastqc_dir: Some(raw_fastqc_dir.clone()),
                lineage_hash: None,
            },
        );

        let expected_raw_fastqc_dir = raw_fastqc_dir.display().to_string();
        assert_eq!(metrics.raw_fastqc_dir.as_deref(), Some(expected_raw_fastqc_dir.as_str()));
        assert!(metrics.multiqc_report.is_some());
        assert!(metrics.multiqc_data.is_some());
    }

    #[test]
    fn governed_qc_manifest_loader_keeps_existing_artifacts() {
        let temp = tempfile::tempdir().expect("tempdir");
        let artifact_path = temp.path().join("trim_report.json");
        let fastqc_dir = temp.path().join("raw_fastqc");
        bijux_dna_infra::write_bytes(&artifact_path, b"{}").expect("artifact");
        bijux_dna_infra::ensure_dir(&fastqc_dir).expect("fastqc dir");
        let manifest_path = temp.path().join("qc_inputs.json");
        bijux_dna_infra::write_bytes(
            &manifest_path,
            serde_json::json!({
                "schema_version": GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION,
                "qc_inputs": [
                    {
                        "name": "fastq.trim_reads.fastp_branch.report_json",
                        "path": artifact_path,
                        "role": "report_json",
                        "optional": false
                    }
                ],
                "raw_fastqc_dir": fastqc_dir,
            })
            .to_string(),
        )
        .expect("manifest");

        let loaded = load_governed_qc_inputs_manifest(&manifest_path).expect("load manifest");
        assert_eq!(loaded.qc_inputs.len(), 1);
        assert_eq!(loaded.contributors.len(), 1);
        assert_eq!(loaded.qc_inputs[0].name.as_str(), "fastq.trim_reads.fastp_branch.report_json");
        assert_eq!(
            loaded.contributors[0],
            GovernedQcContributor {
                contributor_id: "fastq.trim_reads.fastp_branch".to_string(),
                stage_id: "fastq.trim_reads".to_string(),
                tool_id: "fastp_branch".to_string(),
                artifact_id: "report_json".to_string(),
                artifact_role: ArtifactRole::ReportJson,
                path: artifact_path.clone(),
            }
        );
        assert_eq!(loaded.raw_fastqc_dir.as_deref(), Some(fastqc_dir.as_path()));
        assert!(loaded.lineage_hash.as_deref().is_some_and(|lineage| {
            lineage.contains("fastq.trim_reads.fastp_branch:report_json:report_json=")
        }));
    }

    #[test]
    fn governed_qc_manifest_loader_restores_canonical_names_from_contributors() {
        let temp = tempfile::tempdir().expect("tempdir");
        let artifact_path = temp.path().join("trim_report.json");
        bijux_dna_infra::write_bytes(&artifact_path, b"{}").expect("artifact");
        let manifest_path = temp.path().join("qc_inputs.json");
        bijux_dna_infra::write_bytes(
            &manifest_path,
            serde_json::json!({
                "schema_version": GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION,
                "qc_inputs": [
                    {
                        "name": "report_json",
                        "path": artifact_path,
                        "role": "report_json",
                        "optional": false
                    }
                ],
                "contributors": [
                    {
                        "contributor_id": "fastq.trim_reads.fastp",
                        "stage_id": "fastq.trim_reads",
                        "tool_id": "fastp",
                        "artifact_id": "report_json",
                        "artifact_role": "report_json",
                        "path": artifact_path
                    }
                ]
            })
            .to_string(),
        )
        .expect("manifest");

        let loaded = load_governed_qc_inputs_manifest(&manifest_path).expect("load manifest");
        assert_eq!(loaded.qc_inputs.len(), 1);
        assert_eq!(loaded.qc_inputs[0].name.as_str(), "fastq.trim_reads.fastp.report_json");
    }

    #[test]
    fn governed_qc_manifest_loader_rejects_unknown_schema() {
        let temp = tempfile::tempdir().expect("tempdir");
        let artifact_path = temp.path().join("trim_report.json");
        bijux_dna_infra::write_bytes(&artifact_path, b"{}").expect("artifact");
        let manifest_path = temp.path().join("qc_inputs.json");
        bijux_dna_infra::write_bytes(
            &manifest_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.report_qc.inputs.v0",
                "qc_inputs": [
                    {
                        "name": "fastq.trim_reads.fastp_branch.report_json",
                        "path": artifact_path,
                        "role": "report_json",
                        "optional": false
                    }
                ]
            })
            .to_string(),
        )
        .expect("manifest");

        let error = load_governed_qc_inputs_manifest(&manifest_path)
            .expect_err("unknown schema must be rejected");
        assert!(error.to_string().contains("unsupported governed QC input manifest schema"));
    }

    #[test]
    fn derived_governed_qc_lineage_hash_is_stable_for_external_branch_sets() {
        let inputs = vec![
            ArtifactRef::required(
                ArtifactId::from_static("fastq.trim_reads.fastp.report_json"),
                PathBuf::from("/artifacts/runtime/fastp/report.json"),
                ArtifactRole::ReportJson,
            ),
            ArtifactRef::required(
                ArtifactId::from_static(
                    "fastq.validate_reads.fastqvalidator.validated_reads_manifest",
                ),
                PathBuf::from("/artifacts/runtime/validate/lineage.json"),
                ArtifactRole::StageReport,
            ),
        ];
        let raw_fastqc_dir = PathBuf::from("/artifacts/runtime/raw_fastqc");

        let lineage = derived_governed_qc_lineage_hash(
            &governed_qc_contributors(&inputs),
            Some(raw_fastqc_dir.as_path()),
        )
        .expect("derived lineage");
        assert!(lineage.contains(
            "fastq.trim_reads.fastp:report_json:report_json=/artifacts/runtime/fastp/report.json"
        ));
        assert!(lineage.contains(
            "fastq.validate_reads.fastqvalidator:validated_reads_manifest:stage_report=/artifacts/runtime/validate/lineage.json"
        ));
        assert!(lineage.contains("raw_fastqc_dir=/artifacts/runtime/raw_fastqc"));
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn qc_post_record_preserves_planner_written_governed_qc_manifest() {
        let temp = tempfile::tempdir().expect("tempdir");
        bijux_dna_infra::ensure_dir(temp.path().join("multiqc_data")).expect("multiqc data dir");
        bijux_dna_infra::write_bytes(temp.path().join("multiqc_report.html"), b"report")
            .expect("report");
        let input_artifact = temp.path().join("trim_report.json");
        let raw_fastqc_dir = temp.path().join("raw_fastqc");
        bijux_dna_infra::write_bytes(&input_artifact, b"{}").expect("input artifact");
        bijux_dna_infra::ensure_dir(&raw_fastqc_dir).expect("raw fastqc dir");

        let governed_qc = GovernedQcInputs {
            qc_inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("fastq.trim_reads.fastp.report_json"),
                input_artifact.clone(),
                ArtifactRole::ReportJson,
            )],
            contributors: vec![GovernedQcContributor {
                contributor_id: "fastq.trim_reads.fastp".to_string(),
                stage_id: "fastq.trim_reads".to_string(),
                tool_id: "fastp".to_string(),
                artifact_id: "report_json".to_string(),
                artifact_role: ArtifactRole::ReportJson,
                path: input_artifact.clone(),
            }],
            raw_fastqc_dir: Some(raw_fastqc_dir.clone()),
            lineage_hash: Some("fastq.trim_reads=fastp".to_string()),
        };
        let bench_inputs = super::QcPostBenchInputs {
            runner: RuntimeKind::Docker,
            r1: temp.path().join("reads_R1.fastq.gz"),
            r2: None,
            input_hash: "input-hash".to_string(),
            input_stats: SeqkitMetrics { reads: 10, bases: 100, mean_q: 30.0, gc_percent: 50.0 },
            input_stats_r2: None,
            bench_dir: temp.path().join("bench"),
            tools_root: temp.path().join("tools"),
        };
        let tool_spec = ToolExecutionSpecV1 {
            tool_id: ToolId::from_static("multiqc"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
            command: CommandSpecV1 { template: vec!["multiqc".to_string()] },
            resources: ToolConstraints::default(),
        };
        let execution = StageResultV1 {
            run_id: "run".to_string(),
            exit_code: 0,
            runtime_s: 1.0,
            memory_mb: 64.0,
            outputs: Vec::new(),
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: "multiqc".to_string(),
        };

        let manifest = serde_json::json!({
            "schema_version": GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION,
            "qc_inputs": governed_qc.qc_inputs,
            "contributors": governed_qc.contributors,
            "raw_fastqc_dir": raw_fastqc_dir,
            "lineage_hash": governed_qc.lineage_hash,
        });
        bijux_dna_infra::write_bytes(
            governed_qc_inputs_manifest_path(temp.path()),
            manifest.to_string(),
        )
        .expect("write manifest");

        build_qc_post_record(
            &PlatformSpec {
                name: "test".to_string(),
                runner: RuntimeKind::Docker,
                container_dir: PathBuf::from("/tmp"),
                image_prefix: "bijuxdna".to_string(),
                arch: "amd64".to_string(),
            },
            &bench_inputs,
            "multiqc",
            &tool_spec,
            &serde_json::json!({}),
            &governed_qc,
            temp.path(),
            &execution,
        )
        .expect("record");

        let preserved_manifest: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(governed_qc_inputs_manifest_path(temp.path()))
                .expect("manifest"),
        )
        .expect("parse manifest");
        assert_eq!(
            preserved_manifest["schema_version"],
            serde_json::json!(GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION)
        );
        assert_eq!(preserved_manifest["lineage_hash"], serde_json::json!("fastq.trim_reads=fastp"));
        assert_eq!(
            preserved_manifest["contributors"][0]["stage_id"],
            serde_json::json!("fastq.trim_reads")
        );
        assert_eq!(preserved_manifest["qc_inputs"][0]["path"], serde_json::json!(input_artifact));
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn qc_post_record_preserves_upstream_qc_summary_signals() {
        let temp = tempfile::tempdir().expect("tempdir");
        bijux_dna_infra::ensure_dir(temp.path().join("multiqc_data")).expect("multiqc data dir");
        bijux_dna_infra::write_bytes(temp.path().join("multiqc_report.html"), b"report")
            .expect("report");

        let detect_report = temp.path().join("adapter_report.json");
        bijux_dna_infra::write_bytes(
            &detect_report,
            serde_json::to_string(&bijux_dna_domain_fastq::DetectAdaptersReportV1 {
                schema_version: "bijux.fastq.detect_adapters.report.v3".to_string(),
                stage: "fastq.detect_adapters".to_string(),
                stage_id: "fastq.detect_adapters".to_string(),
                tool_id: "fastqc".to_string(),
                paired_mode: bijux_dna_domain_fastq::PairedMode::SingleEnd,
                threads: 1,
                inspection_mode:
                    bijux_dna_domain_fastq::params::detect_adapters::AdapterInspectionMode::EvidenceOnly,
                report_only: true,
                evidence_engine: "fastqc".to_string(),
                evidence_scope:
                    bijux_dna_domain_fastq::params::detect_adapters::AdapterEvidenceScope::FullInput,
                evidence_format:
                    bijux_dna_domain_fastq::params::detect_adapters::AdapterEvidenceFormat::FastqcSummary,
                evidence_artifact_id: "report_json".to_string(),
                detected_adapter_source: "normalized_fastqc_evidence".to_string(),
                detected_adapter_ids: vec!["truseq_universal".to_string()],
                detection_confidence: None,
                detection_threshold: None,
                input_r1: "reads_R1.fastq.gz".to_string(),
                input_r2: None,
                report_json: "adapter_report.json".to_string(),
                adapter_evidence_dir: "fastqc".to_string(),
                recommended_adapter_bank_id: None,
                recommended_adapter_bank_hash: None,
                recommended_adapter_preset: None,
                reads_in: 10,
                reads_out: 10,
                bases_in: 100,
                bases_out: 100,
                pairs_in: None,
                pairs_out: None,
                mean_q: 30.0,
                candidate_adapter_count: 1,
                adapter_trimmed_fraction: Some(0.05),
                adapter_content_max: Some(0.12),
                adapter_content_mean: Some(0.04),
                duplication_rate: Some(0.11),
                n_rate: Some(0.002),
                kmer_warning_count: Some(3),
                overrepresented_sequence_count: Some(2),
                runtime_s: Some(1.0),
                memory_mb: Some(32.0),
                exit_code: Some(0),
                raw_backend_report: None,
                raw_backend_report_format: None,
            })
            .expect("serialize detect report"),
        )
        .expect("detect report");

        let screen_report = temp.path().join("screen_report.json");
        bijux_dna_infra::write_bytes(
            &screen_report,
            serde_json::json!({
                "schema_version": "bijux.fastq.screen_taxonomy.report.v2",
                "stage": "fastq.screen_taxonomy",
                "stage_id": "fastq.screen_taxonomy",
                "tool_id": "kraken2",
                "paired_mode": "single_end",
                "threads": 1,
                "classifier": "kraken2",
                "report_format": "kraken_report",
                "assignment_format": "kraken_assignments",
                "database_catalog_id": "taxonomy_reference",
                "database_artifact_id": "taxonomy_db",
                "database_build_id": null,
                "database_digest": null,
                "database_namespace": "read_screening",
                "database_scope": "read_screening",
                "minimum_confidence": null,
                "emit_unclassified": true,
                "interpretation_boundary": "screening_only",
                "truth_conditions": [],
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": null,
                "screen_report_tsv": "kraken2.report.tsv",
                "classification_report_json": "kraken2.classifications.json",
                "reads_in": 10,
                "reads_out": 10,
                "bases_in": 100,
                "bases_out": 100,
                "pairs_in": null,
                "pairs_out": null,
                "contamination_rate": 0.23,
                "classified_fraction": 0.23,
                "unclassified_fraction": 0.77,
                "summary_entries": [],
                "top_taxa": [],
                "runtime_s": 1.0,
                "memory_mb": 32.0
            })
            .to_string(),
        )
        .expect("screen report");

        let governed_qc = GovernedQcInputs {
            qc_inputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("fastq.detect_adapters.fastqc.report_json"),
                    detect_report.clone(),
                    ArtifactRole::ReportJson,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("fastq.screen_taxonomy.kraken2.report_json"),
                    screen_report.clone(),
                    ArtifactRole::ReportJson,
                ),
            ],
            contributors: vec![
                GovernedQcContributor {
                    contributor_id: "fastq.detect_adapters.fastqc".to_string(),
                    stage_id: "fastq.detect_adapters".to_string(),
                    tool_id: "fastqc".to_string(),
                    artifact_id: "report_json".to_string(),
                    artifact_role: ArtifactRole::ReportJson,
                    path: detect_report.clone(),
                },
                GovernedQcContributor {
                    contributor_id: "fastq.screen_taxonomy.kraken2".to_string(),
                    stage_id: "fastq.screen_taxonomy".to_string(),
                    tool_id: "kraken2".to_string(),
                    artifact_id: "report_json".to_string(),
                    artifact_role: ArtifactRole::ReportJson,
                    path: screen_report.clone(),
                },
            ],
            raw_fastqc_dir: None,
            lineage_hash: Some("lineage".to_string()),
        };
        let bench_inputs = super::QcPostBenchInputs {
            runner: RuntimeKind::Docker,
            r1: temp.path().join("reads_R1.fastq.gz"),
            r2: None,
            input_hash: "input-hash".to_string(),
            input_stats: SeqkitMetrics { reads: 10, bases: 100, mean_q: 30.0, gc_percent: 50.0 },
            input_stats_r2: None,
            bench_dir: temp.path().join("bench"),
            tools_root: temp.path().join("tools"),
        };
        let tool_spec = ToolExecutionSpecV1 {
            tool_id: ToolId::from_static("multiqc"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
            command: CommandSpecV1 { template: vec!["multiqc".to_string()] },
            resources: ToolConstraints::default(),
        };
        let execution = StageResultV1 {
            run_id: "run".to_string(),
            exit_code: 0,
            runtime_s: 1.0,
            memory_mb: 64.0,
            outputs: Vec::new(),
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: "multiqc".to_string(),
        };

        let record = build_qc_post_record(
            &PlatformSpec {
                name: "test".to_string(),
                runner: RuntimeKind::Docker,
                container_dir: PathBuf::from("/tmp"),
                image_prefix: "bijuxdna".to_string(),
                arch: "amd64".to_string(),
            },
            &bench_inputs,
            "multiqc",
            &tool_spec,
            &serde_json::json!({}),
            &governed_qc,
            temp.path(),
            &execution,
        )
        .expect("record");

        assert!((record.metrics.metrics.contamination_rate - 0.23).abs() < f64::EPSILON);

        let governed_report: ReportQcReportV1 = serde_json::from_str(
            &std::fs::read_to_string(temp.path().join("report_qc_report.json"))
                .expect("report json"),
        )
        .expect("parse governed report");
        assert!((governed_report.contamination_rate - 0.23).abs() < f64::EPSILON);
        assert!(governed_report
            .adapter_content_max
            .is_some_and(|value| (value - 0.12).abs() < f64::EPSILON));
        assert!(governed_report
            .adapter_content_mean
            .is_some_and(|value| (value - 0.04).abs() < f64::EPSILON));
        assert!(governed_report
            .duplication_rate
            .is_some_and(|value| (value - 0.11).abs() < f64::EPSILON));
        assert!(governed_report.n_rate.is_some_and(|value| (value - 0.002).abs() < f64::EPSILON));
        assert_eq!(governed_report.kmer_warning_count, Some(3));
        assert_eq!(governed_report.overrepresented_sequence_count, Some(2));
    }

    #[test]
    fn governed_qc_contributors_capture_stage_and_artifact_ids() {
        let contributors = governed_qc_contributors(&[ArtifactRef::required(
            ArtifactId::from_static("fastq.validate_reads.fastqvalidator.validation_report"),
            PathBuf::from("/artifacts/runtime/validation.json"),
            ArtifactRole::ReportJson,
        )]);
        assert_eq!(contributors.len(), 1);
        assert_eq!(contributors[0].stage_id, "fastq.validate_reads");
        assert_eq!(contributors[0].contributor_id, "fastq.validate_reads.fastqvalidator");
        assert_eq!(contributors[0].artifact_id, "validation_report");
    }

    #[test]
    fn governed_qc_contributor_tool_ids_are_stable_and_unique() {
        let contributors = vec![
            GovernedQcContributor {
                contributor_id: "fastq.trim_reads.fastp".to_string(),
                stage_id: "fastq.trim_reads".to_string(),
                tool_id: "fastp".to_string(),
                artifact_id: "report_json".to_string(),
                artifact_role: ArtifactRole::ReportJson,
                path: PathBuf::from("/artifacts/runtime/trim/report.json"),
            },
            GovernedQcContributor {
                contributor_id: "fastq.validate_reads.fastqvalidator".to_string(),
                stage_id: "fastq.validate_reads".to_string(),
                tool_id: "fastqvalidator".to_string(),
                artifact_id: "validation_report".to_string(),
                artifact_role: ArtifactRole::ReportJson,
                path: PathBuf::from("/artifacts/runtime/validate/report.json"),
            },
            GovernedQcContributor {
                contributor_id: "fastq.trim_reads.fastp".to_string(),
                stage_id: "fastq.trim_reads".to_string(),
                tool_id: "fastp".to_string(),
                artifact_id: "adapter_report".to_string(),
                artifact_role: ArtifactRole::ReportJson,
                path: PathBuf::from("/artifacts/runtime/trim/adapter.json"),
            },
        ];

        assert_eq!(
            governed_qc_contributor_stage_ids(&contributors),
            vec!["fastq.trim_reads".to_string(), "fastq.validate_reads".to_string()]
        );
        assert_eq!(
            governed_qc_contributor_tool_ids(&contributors),
            vec!["fastp".to_string(), "fastqvalidator".to_string()]
        );
    }

    #[test]
    fn report_qc_aux_images_follow_governed_contributors() {
        let governed_qc = GovernedQcInputs {
            qc_inputs: Vec::new(),
            contributors: vec![
                GovernedQcContributor {
                    contributor_id: "fastq.trim_reads.fastp".to_string(),
                    stage_id: "fastq.trim_reads".to_string(),
                    tool_id: "fastp".to_string(),
                    artifact_id: "report_json".to_string(),
                    artifact_role: ArtifactRole::ReportJson,
                    path: PathBuf::from("/artifacts/runtime/trim/report.json"),
                },
                GovernedQcContributor {
                    contributor_id: "fastq.validate_reads.fastqvalidator".to_string(),
                    stage_id: "fastq.validate_reads".to_string(),
                    tool_id: "fastqvalidator".to_string(),
                    artifact_id: "validation_report".to_string(),
                    artifact_role: ArtifactRole::ReportJson,
                    path: PathBuf::from("/artifacts/runtime/validate/report.json"),
                },
            ],
            raw_fastqc_dir: None,
            lineage_hash: None,
        };
        let mut catalog = BTreeMap::new();
        catalog.insert(
            "fastp".to_string(),
            ToolImageSpec {
                tool: "fastp".to_string(),
                version: "1.0.0".to_string(),
                digest: Some("sha256:fastp".to_string()),
                enabled: Some(true),
                shipping_policy: None,
            },
        );
        catalog.insert(
            "fastqvalidator".to_string(),
            ToolImageSpec {
                tool: "fastqvalidator".to_string(),
                version: "1.0.0".to_string(),
                digest: Some("sha256:fastqvalidator".to_string()),
                enabled: Some(true),
                shipping_policy: None,
            },
        );

        let aux_images = resolve_qc_contributor_aux_images(
            &catalog,
            &PlatformSpec {
                name: "test".to_string(),
                runner: RuntimeKind::Docker,
                container_dir: PathBuf::from("/tmp"),
                image_prefix: "bijuxdna".to_string(),
                arch: "amd64".to_string(),
            },
            &governed_qc,
        )
        .expect("aux images");

        assert_eq!(aux_images.len(), 2);
        assert!(aux_images.contains_key("fastp"));
        assert!(aux_images.contains_key("fastqvalidator"));
    }

    #[test]
    fn governed_qc_contributor_validation_rejects_unmatched_records() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("report.json");
        bijux_dna_infra::write_bytes(&report_path, b"{}").expect("report");
        let qc_inputs = vec![ArtifactRef::required(
            ArtifactId::from_static("fastq.trim_reads.fastp.report_json"),
            report_path.clone(),
            ArtifactRole::ReportJson,
        )];
        let error = validate_governed_qc_contributors(
            &[GovernedQcContributor {
                contributor_id: "fastq.trim_reads.fastp".to_string(),
                stage_id: "fastq.trim_reads".to_string(),
                tool_id: "fastp".to_string(),
                artifact_id: "validation_report".to_string(),
                artifact_role: ArtifactRole::ReportJson,
                path: report_path,
            }],
            &qc_inputs,
            temp.path().join("governed_qc_inputs.json").as_path(),
        )
        .expect_err("unmatched contributor artifact ids must fail");
        assert!(error.to_string().contains("does not match any qc_inputs entry"));
    }

    #[test]
    fn governed_qc_contributor_validation_rejects_role_mismatch() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("report.json");
        bijux_dna_infra::write_bytes(&report_path, b"{}").expect("report");
        let qc_inputs = vec![ArtifactRef::required(
            ArtifactId::from_static("fastq.trim_reads.fastp.report_json"),
            report_path.clone(),
            ArtifactRole::ReportJson,
        )];
        let error = validate_governed_qc_contributors(
            &[GovernedQcContributor {
                contributor_id: "fastq.trim_reads.fastp".to_string(),
                stage_id: "fastq.trim_reads".to_string(),
                tool_id: "fastp".to_string(),
                artifact_id: "report_json".to_string(),
                artifact_role: ArtifactRole::StageReport,
                path: report_path,
            }],
            &qc_inputs,
            temp.path().join("governed_qc_inputs.json").as_path(),
        )
        .expect_err("mismatched contributor artifact roles must fail");
        assert!(error.to_string().contains("does not match any qc_inputs entry"));
    }
}
