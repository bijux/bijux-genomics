use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_workspace_registry};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::fetch_fastq_qc_post_v1;
use bijux_dna_analyze::quality::insert_fastq_qc_post_v1;
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkContext, BenchmarkRecord, FastqQcPostMetrics,
};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::ArtifactRef;
use bijux_dna_domain_fastq::params::{qc_post::QcAggregationScope, PairedMode};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::select_qc_post_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::report_qc::plan_qc_post_with_qc_inputs;
use bijux_dna_planner_fastq::stage_api::observer::{input_fastq_stats, parse_seqkit_stats};
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::backend::docker::executor::resolve_image_for_run;
use bijux_dna_runner::step_runner::{execute_observer_command, StageResultV1};
use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_REPORT_QC,
};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;

const GOVERNED_QC_INPUTS_SCHEMA_VERSION: &str = "bijux.fastq.report_qc.inputs.v1";

fn parse_qc_aggregation_scope(value: Option<&str>) -> Result<QcAggregationScope> {
    match value.unwrap_or("governed_qc_artifacts") {
        "governed_qc_artifacts" => Ok(QcAggregationScope::GovernedQcArtifacts),
        "fastq_qc_inputs" => Ok(QcAggregationScope::FastqQcInputs),
        other => Err(anyhow!(
            "unsupported fastq.report_qc aggregation_scope `{other}`; expected one of: governed_qc_artifacts, fastq_qc_inputs"
        )),
    }
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_qc_post<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqQcPostArgs,
) -> Result<BenchOutcome<bijux_dna_analyze::FastqQcPostMetrics>> {
    let aggregation_scope = parse_qc_aggregation_scope(args.aggregation_scope.as_deref())?;
    if aggregation_scope != QcAggregationScope::GovernedQcArtifacts {
        return Err(anyhow!(
            "fastq.report_qc benchmarking currently supports only aggregation_scope=governed_qc_artifacts because it consumes governed upstream QC artifacts rather than regenerating them from raw FASTQ inputs"
        ));
    }
    let governed_qc = load_required_governed_qc_inputs_manifest(args.governed_qc_manifest.as_deref())?;
    let tools = select_qc_post_tools(&args.tools)?;
    let artifact_kind = if args.r2.is_some() {
        FastqArtifactKind::PairedEnd
    } else {
        FastqArtifactKind::SingleEnd
    };
    preflight_stage(STAGE_REPORT_QC.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_REPORT_QC.as_str(), &header);

    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_REPORT_QC.as_str(), &tools, &registry, false)?;
    let bench_inputs = prepare_qc_post_bench(catalog, platform, runner_override, args)?;
    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_REPORT_QC.as_str());
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
        let plan = plan_qc_post_with_qc_inputs(
            &tool_spec,
            &governed_qc.qc_inputs,
            &out_dir,
            std::collections::BTreeMap::new(),
            paired_mode_for_bench_inputs(&bench_inputs),
            aggregation_scope.clone(),
            Some(&bench_inputs.r1),
            bench_inputs.r2.as_deref(),
        )?;
        let bench_params =
            benchmark_query_context(governed_qc.lineage_hash.as_deref())?
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
            vec![bijux_dna_stage_contract::execution_step_from_stage_plan(
                &plan,
            )],
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
            &out_dir,
            &execution,
            governed_qc.raw_fastqc_dir.as_deref(),
            &governed_qc,
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

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: bench_inputs.bench_dir,
        explain: args.explain,
    })
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
    raw_fastqc_dir: Option<PathBuf>,
    lineage_hash: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct GovernedQcInputsManifest {
    schema_version: String,
    qc_inputs: Vec<ArtifactRef>,
    #[serde(default)]
    raw_fastqc_dir: Option<PathBuf>,
    #[serde(default)]
    lineage_hash: Option<String>,
}

fn governed_qc_inputs_manifest_path(out_dir: &Path) -> PathBuf {
    out_dir.join("governed_qc_inputs_manifest.json")
}

fn load_required_governed_qc_inputs_manifest(
    manifest_path: Option<&Path>,
) -> Result<GovernedQcInputs> {
    let manifest_path = manifest_path.ok_or_else(|| {
        anyhow!(
            "fastq.report_qc benchmarking requires --governed-qc-manifest; this stage aggregates governed upstream QC artifacts and does not regenerate them from raw FASTQ inputs"
        )
    })?;
    load_governed_qc_inputs_manifest(manifest_path)
}

fn prepare_qc_post_bench<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
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
            "seqkit qc observer failed: {}",
            stats_output.stderr
        ));
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
            return Err(anyhow!(
                "seqkit qc observer failed for r2: {}",
                stats_output.stderr
            ));
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

fn build_qc_post_record(
    platform: &PlatformSpec,
    bench_inputs: &QcPostBenchInputs,
    tool: &str,
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    params: &serde_json::Value,
    out_dir: &Path,
    execution: &StageResultV1,
    raw_fastqc_dir: Option<&Path>,
    governed_qc: &GovernedQcInputs,
) -> Result<BenchmarkRecord<FastqQcPostMetrics>> {
    let metrics = derive_qc_post_metrics(
        &bench_inputs.input_stats,
        bench_inputs.input_stats_r2.as_ref(),
        out_dir,
        raw_fastqc_dir,
    );
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    let report = serde_json::json!({
        "schema_version": "bijux.fastq.report_qc.report.v1",
        "stage": STAGE_REPORT_QC.as_str(),
        "stage_id": STAGE_REPORT_QC.as_str(),
        "tool": tool,
        "tool_id": tool,
        "input_fastq_r1": bench_inputs.r1,
        "input_fastq_r2": bench_inputs.r2,
        "reads_in": metrics.reads_in,
        "reads_out": metrics.reads_out,
        "bases_in": metrics.bases_in,
        "bases_out": metrics.bases_out,
        "pairs_in": metrics.pairs_in,
        "pairs_out": metrics.pairs_out,
        "mean_q": metrics.mean_q,
        "contamination_rate": metrics.contamination_rate,
        "raw_fastqc_dir": metrics.raw_fastqc_dir,
        "trimmed_fastqc_dir": metrics.trimmed_fastqc_dir,
        "report_html": metrics.multiqc_report,
        "report_data_dir": metrics.multiqc_data,
        "multiqc_report": metrics.multiqc_report,
        "multiqc_data": metrics.multiqc_data,
        "runtime_s": execution.runtime_s,
        "memory_mb": execution.memory_mb,
        "exit_code": execution.exit_code,
    });
    bijux_dna_infra::atomic_write_json(&out_dir.join("qc_report.json"), &report)
        .context("write qc report")?;
    let governed_qc_manifest = GovernedQcInputsManifest {
        schema_version: GOVERNED_QC_INPUTS_SCHEMA_VERSION.to_string(),
        qc_inputs: governed_qc.qc_inputs.clone(),
        raw_fastqc_dir: governed_qc.raw_fastqc_dir.clone(),
        lineage_hash: governed_qc.lineage_hash.clone(),
    };
    bijux_dna_infra::atomic_write_json(
        &governed_qc_inputs_manifest_path(out_dir),
        &governed_qc_manifest,
    )
    .context("write governed QC inputs manifest")?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write qc metrics")?;

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

fn derive_qc_post_metrics(
    input_stats: &SeqkitMetrics,
    input_stats_r2: Option<&SeqkitMetrics>,
    out_dir: &Path,
    raw_fastqc_dir: Option<&Path>,
) -> FastqQcPostMetrics {
    let multiqc_report = out_dir.join("multiqc_report.html");
    let multiqc_data = out_dir.join("multiqc_data");
    let trimmed_fastqc_dir = out_dir.join("fastqc_trimmed");
    let reads_in = input_stats.reads + input_stats_r2.map_or(0, |stats| stats.reads);
    let bases_in = input_stats.bases + input_stats_r2.map_or(0, |stats| stats.bases);
    let weighted_q_sum = input_stats.mean_q * input_stats.bases as f64
        + input_stats_r2.map_or(0.0, |stats| stats.mean_q * stats.bases as f64);
    let mean_q = if bases_in == 0 {
        0.0
    } else {
        weighted_q_sum / bases_in as f64
    };
    FastqQcPostMetrics {
        reads_in,
        reads_out: reads_in,
        bases_in,
        bases_out: bases_in,
        pairs_in: input_stats_r2.map(|stats| input_stats.reads.min(stats.reads)),
        pairs_out: input_stats_r2.map(|stats| input_stats.reads.min(stats.reads)),
        mean_q,
        contamination_rate: 0.0,
        raw_fastqc_dir: raw_fastqc_dir.and_then(path_if_exists),
        trimmed_fastqc_dir: path_if_exists(&trimmed_fastqc_dir),
        multiqc_report: path_if_exists(&multiqc_report),
        multiqc_data: path_if_exists(&multiqc_data),
    }
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

fn load_governed_qc_inputs_manifest(path: &Path) -> Result<GovernedQcInputs> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read governed QC input manifest {}", path.display()))?;
    let manifest: GovernedQcInputsManifest = serde_json::from_str(&raw)
        .with_context(|| format!("parse governed QC input manifest {}", path.display()))?;
    if manifest.schema_version != GOVERNED_QC_INPUTS_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported governed QC input manifest schema `{}` in {}",
            manifest.schema_version,
            path.display()
        ));
    }
    if manifest.qc_inputs.is_empty() {
        return Err(anyhow!(
            "governed QC input manifest {} must declare at least one qc_inputs entry",
            path.display()
        ));
    }
    for artifact in &manifest.qc_inputs {
        if !artifact.path.exists() {
            return Err(anyhow!(
                "governed QC input artifact {} does not exist at {}",
                artifact.name.as_str(),
                artifact.path.display()
            ));
        }
    }
    if let Some(raw_fastqc_dir) = manifest.raw_fastqc_dir.as_ref() {
        if !raw_fastqc_dir.exists() {
            return Err(anyhow!(
                "governed QC raw_fastqc_dir does not exist at {}",
                raw_fastqc_dir.display()
            ));
        }
    }
    let mut qc_inputs = manifest.qc_inputs;
    qc_inputs.sort_by(|left, right| {
        left.name
            .as_str()
            .cmp(right.name.as_str())
            .then_with(|| left.path.cmp(&right.path))
    });
    qc_inputs.dedup_by(|left, right| left.name == right.name && left.path == right.path);
    Ok(GovernedQcInputs {
        lineage_hash: manifest.lineage_hash.or_else(|| {
            derived_governed_qc_lineage_hash(&qc_inputs, manifest.raw_fastqc_dir.as_deref())
        }),
        qc_inputs,
        raw_fastqc_dir: manifest.raw_fastqc_dir,
    })
}

fn derived_governed_qc_lineage_hash(
    qc_inputs: &[ArtifactRef],
    raw_fastqc_dir: Option<&Path>,
) -> Option<String> {
    let mut lineage_parts = qc_inputs
        .iter()
        .map(|artifact| format!("{}={}", artifact.name.as_str(), artifact.path.display()))
        .collect::<Vec<_>>();
    if let Some(raw_fastqc_dir) = raw_fastqc_dir {
        lineage_parts.push(format!("raw_fastqc_dir={}", raw_fastqc_dir.display()));
    }
    lineage_parts.sort();
    (!lineage_parts.is_empty()).then(|| lineage_parts.join("|"))
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
mod tests {
    use super::{
        build_qc_post_record, derive_qc_post_metrics, derived_governed_qc_lineage_hash,
        governed_qc_inputs_manifest_path, load_governed_qc_inputs_manifest,
        load_required_governed_qc_inputs_manifest, parse_qc_aggregation_scope, GovernedQcInputs,
        GOVERNED_QC_INPUTS_SCHEMA_VERSION,
    };
    use std::path::PathBuf;

    use bijux_dna_core::contract::{ArtifactRole, ToolConstraints};
    use bijux_dna_core::ids::{ArtifactId, ToolId};
    use bijux_dna_core::prelude::measure::SeqkitMetrics;
    use bijux_dna_core::prelude::{
        ArtifactRef, CommandSpecV1, ContainerImageRefV1, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_fastq::params::qc_post::QcAggregationScope;
    use bijux_dna_environment::api::{PlatformSpec, RuntimeKind};
    use bijux_dna_runner::step_runner::StageResultV1;

    #[test]
    fn required_governed_qc_manifest_is_enforced() {
        let error = load_required_governed_qc_inputs_manifest(None)
            .expect_err("manifest requirement must be enforced");
        assert!(error
            .to_string()
            .contains("requires --governed-qc-manifest"));
    }

    #[test]
    fn qc_post_scope_parser_defaults_to_governed_artifacts() {
        assert_eq!(
            parse_qc_aggregation_scope(None).expect("default scope"),
            QcAggregationScope::GovernedQcArtifacts
        );
        assert_eq!(
            parse_qc_aggregation_scope(Some("fastq_qc_inputs")).expect("explicit scope"),
            QcAggregationScope::FastqQcInputs
        );
    }

    #[test]
    fn qc_post_metrics_report_governed_fastqc_dir_when_present() {
        let temp = tempfile::tempdir().expect("tempdir");
        let multiqc_data = temp.path().join("multiqc_data");
        std::fs::create_dir_all(&multiqc_data).expect("multiqc data dir");
        std::fs::write(temp.path().join("multiqc_report.html"), b"report").expect("report");
        let raw_fastqc_dir = temp
            .path()
            .join("governed_qc_inputs/detect_adapters/fastqc/fastqc");
        std::fs::create_dir_all(&raw_fastqc_dir).expect("fastqc dir");

        let metrics = derive_qc_post_metrics(
            &SeqkitMetrics {
                reads: 10,
                bases: 100,
                mean_q: 30.0,
                gc_percent: 50.0,
            },
            None,
            temp.path(),
            Some(raw_fastqc_dir.as_path()),
        );

        let expected_raw_fastqc_dir = raw_fastqc_dir.display().to_string();
        assert_eq!(
            metrics.raw_fastqc_dir.as_deref(),
            Some(expected_raw_fastqc_dir.as_str())
        );
        assert!(metrics.multiqc_report.is_some());
        assert!(metrics.multiqc_data.is_some());
    }

    #[test]
    fn governed_qc_manifest_loader_keeps_existing_artifacts() {
        let temp = tempfile::tempdir().expect("tempdir");
        let artifact_path = temp.path().join("trim_report.json");
        let fastqc_dir = temp.path().join("raw_fastqc");
        std::fs::write(&artifact_path, b"{}").expect("artifact");
        std::fs::create_dir_all(&fastqc_dir).expect("fastqc dir");
        let manifest_path = temp.path().join("qc_inputs.json");
        std::fs::write(
            &manifest_path,
            serde_json::json!({
                "schema_version": GOVERNED_QC_INPUTS_SCHEMA_VERSION,
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
        assert_eq!(
            loaded.qc_inputs[0].name.as_str(),
            "fastq.trim_reads.fastp_branch.report_json"
        );
        assert_eq!(loaded.raw_fastqc_dir.as_deref(), Some(fastqc_dir.as_path()));
        assert!(loaded
            .lineage_hash
            .as_deref()
            .is_some_and(|lineage| lineage.contains("fastq.trim_reads.fastp_branch.report_json")));
    }

    #[test]
    fn governed_qc_manifest_loader_rejects_unknown_schema() {
        let temp = tempfile::tempdir().expect("tempdir");
        let artifact_path = temp.path().join("trim_report.json");
        std::fs::write(&artifact_path, b"{}").expect("artifact");
        let manifest_path = temp.path().join("qc_inputs.json");
        std::fs::write(
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
        assert!(error
            .to_string()
            .contains("unsupported governed QC input manifest schema"));
    }

    #[test]
    fn derived_governed_qc_lineage_hash_is_stable_for_external_branch_sets() {
        let inputs = vec![
            ArtifactRef::required(
                ArtifactId::from_static("fastq.trim_reads.fastp.report_json"),
                PathBuf::from("/tmp/fastp/report.json"),
                ArtifactRole::ReportJson,
            ),
            ArtifactRef::required(
                ArtifactId::from_static(
                    "fastq.validate_reads.fastqvalidator.validated_reads_manifest",
                ),
                PathBuf::from("/tmp/validate/lineage.json"),
                ArtifactRole::StageReport,
            ),
        ];
        let raw_fastqc_dir = PathBuf::from("/tmp/raw_fastqc");

        let lineage = derived_governed_qc_lineage_hash(&inputs, Some(raw_fastqc_dir.as_path()))
            .expect("derived lineage");
        assert!(lineage.contains("fastq.trim_reads.fastp.report_json=/tmp/fastp/report.json"));
        assert!(lineage.contains(
            "fastq.validate_reads.fastqvalidator.validated_reads_manifest=/tmp/validate/lineage.json"
        ));
        assert!(lineage.contains("raw_fastqc_dir=/tmp/raw_fastqc"));
    }

    #[test]
    fn qc_post_record_writes_governed_qc_inputs_manifest() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(temp.path().join("multiqc_data")).expect("multiqc data dir");
        std::fs::write(temp.path().join("multiqc_report.html"), b"report").expect("report");
        let input_artifact = temp.path().join("trim_report.json");
        let raw_fastqc_dir = temp.path().join("raw_fastqc");
        std::fs::write(&input_artifact, b"{}").expect("input artifact");
        std::fs::create_dir_all(&raw_fastqc_dir).expect("raw fastqc dir");

        let governed_qc = GovernedQcInputs {
            qc_inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("fastq.trim_reads.fastp.report_json"),
                input_artifact.clone(),
                ArtifactRole::ReportJson,
            )],
            raw_fastqc_dir: Some(raw_fastqc_dir.clone()),
            lineage_hash: Some("fastq.trim_reads=fastp".to_string()),
        };
        let bench_inputs = super::QcPostBenchInputs {
            runner: RuntimeKind::Docker,
            r1: temp.path().join("reads_R1.fastq.gz"),
            r2: None,
            input_hash: "input-hash".to_string(),
            input_stats: SeqkitMetrics {
                reads: 10,
                bases: 100,
                mean_q: 30.0,
                gc_percent: 50.0,
            },
            input_stats_r2: None,
            bench_dir: temp.path().join("bench"),
            tools_root: temp.path().join("tools"),
        };
        let tool_spec = ToolExecutionSpecV1 {
            tool_id: ToolId::from_static("multiqc"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: vec!["multiqc".to_string()],
            },
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
            temp.path(),
            &execution,
            Some(raw_fastqc_dir.as_path()),
            &governed_qc,
        )
        .expect("record");

        let manifest: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(governed_qc_inputs_manifest_path(temp.path()))
                .expect("manifest"),
        )
        .expect("parse manifest");
        assert_eq!(
            manifest["schema_version"],
            serde_json::json!(GOVERNED_QC_INPUTS_SCHEMA_VERSION)
        );
        assert_eq!(manifest["lineage_hash"], serde_json::json!("fastq.trim_reads=fastp"));
        assert_eq!(manifest["qc_inputs"][0]["path"], serde_json::json!(input_artifact));
    }
}
