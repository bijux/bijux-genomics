use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{
    fetch_fastq_normalize_abundance_v1, insert_fastq_normalize_abundance_v1,
};
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkRecord, FastqNormalizeAbundanceMetrics, StageMetricSchema,
};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::ExecutionMetrics;
use bijux_dna_domain_fastq::params::edna::AbundanceNormalizationEffectiveParams;
use bijux_dna_domain_fastq::{
    NormalizeAbundanceReportV1, NORMALIZE_ABUNDANCE_REPORT_SCHEMA_VERSION,
};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::RawFailure;
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;

use crate::internal::fastq::stages::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context,
};
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{write_explain_md, write_explain_plan_json, BenchOutcome};

const STAGE_ID: &str = "fastq.normalize_abundance";
const LOCAL_NORMALIZE_ABUNDANCE_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.normalize_abundance.local_smoke.report.v1";

#[derive(Debug, Clone, serde::Serialize)]
struct LocalNormalizeAbundanceSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    planned_tool_id: String,
    report_tool_id: String,
    method: String,
    table_rows: u64,
    sample_count: u64,
    feature_count: u64,
    zero_fraction: f64,
    normalized_abundance_tsv: String,
    case_report_json: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NormalizedAbundanceTableMetrics {
    pub table_rows: u64,
    pub sample_count: u64,
    pub feature_count: u64,
    pub zero_fraction: f64,
    pub per_sample_sums: Vec<(String, f64)>,
}

struct NormalizeAbundanceOutputs {
    normalized_table: std::path::PathBuf,
    report_json: std::path::PathBuf,
}

impl NormalizedAbundanceTableMetrics {
    fn to_benchmark_metrics(
        &self,
        normalization_method: &str,
    ) -> Result<FastqNormalizeAbundanceMetrics> {
        let metrics = FastqNormalizeAbundanceMetrics {
            table_rows: self.table_rows,
            sample_count: self.sample_count,
            zero_fraction: self.zero_fraction,
            normalization_method: normalization_method.to_string(),
        };
        metrics.validate()?;
        Ok(metrics)
    }
}

pub(crate) fn normalize_abundance_effective_params(
    method: &str,
) -> Result<AbundanceNormalizationEffectiveParams> {
    let (normalized_value_column, compositional_rule, scale_factor) = match method {
        "relative_abundance" => {
            ("normalized_abundance".to_string(), "per_sample_sum_to_one".to_string(), None)
        }
        "counts_per_million" => (
            "counts_per_million".to_string(),
            "per_sample_sum_to_one_million".to_string(),
            Some(1_000_000.0),
        ),
        _ => return Err(anyhow!("unsupported fastq.normalize_abundance method `{method}`")),
    };
    Ok(AbundanceNormalizationEffectiveParams {
        schema_version: bijux_dna_domain_fastq::params::edna::EDNA_SCHEMA_VERSION.to_string(),
        method: method.to_string(),
        expected_columns: vec![
            "sample_id".to_string(),
            "feature_id".to_string(),
            "abundance".to_string(),
        ],
        input_value_column: "abundance".to_string(),
        normalized_value_column,
        compositional_rule,
        scale_factor,
        report_artifact: "report_json".to_string(),
    })
}

/// Benchmark FASTQ abundance normalization tools under governed contracts.
///
/// # Errors
/// Returns an error if planning, execution, normalization, or persistence fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_normalize_abundance<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqNormalizeAbundanceArgs,
) -> Result<BenchOutcome<FastqNormalizeAbundanceMetrics>> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = bijux_dna_planner_fastq::select_normalize_abundance_tools(&args.tools)?;
    let tools = filter_tools_by_role(STAGE_ID, &tools, &registry, false)?;
    let runner = ensure_bench_runner(platform, runner_override)?;
    let input_hash = hash_file_sha256(&args.table).context("hash normalize abundance table")?;
    let bench_dir_name =
        bench_dir_name(&bijux_dna_domain_fastq::stages::ids::STAGE_NORMALIZE_ABUNDANCE)
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
        let plan_options =
            bijux_dna_planner_fastq::tool_adapters::fastq::normalize_abundance::NormalizeAbundancePlanOptions {
                method: args
                    .method
                    .clone()
                    .unwrap_or_else(|| "relative_abundance".to_string()),
            };
        let plan =
            bijux_dna_planner_fastq::tool_adapters::fastq::normalize_abundance::plan_with_options(
                &tool_spec,
                &args.table,
                &out_dir,
                &plan_options,
            )?;
        let params_hash = stable_params_hash(&plan.params);
        let image_digest = benchmark_image_identity(&tool_spec);
        if let Ok(Some(record)) = fetch_fastq_normalize_abundance_v1(
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
            vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&plan)],
            runner,
            jobs,
        )?
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
        let effective_params: AbundanceNormalizationEffectiveParams =
            serde_json::from_value(plan.effective_params.clone())
                .context("decode abundance normalization effective params")?;
        let outputs = resolve_normalize_abundance_outputs(&plan)?;
        let used_fallback = !outputs.normalized_table.exists();
        let table_metrics = if used_fallback {
            materialize_normalized_table(&args.table, &outputs.normalized_table, &effective_params)?
        } else {
            read_normalized_table_metrics(&outputs.normalized_table, &effective_params)?
        };
        let metrics = table_metrics.to_benchmark_metrics(&effective_params.method)?;
        let metric_set = metric_set(metrics);
        let report = canonical_normalize_abundance_report(
            STAGE_ID,
            tool,
            &args.table,
            &outputs.normalized_table,
            &effective_params,
            &table_metrics,
            Some(&ExecutionMetrics {
                runtime_s: execution.runtime_s,
                memory_mb: execution.memory_mb,
                exit_code: execution.exit_code,
            }),
            None,
            None,
            used_fallback,
            Some(serde_json::json!({
                "metric_set": serde_json::to_value(&metric_set)?,
                "execution_exit_code": execution.exit_code,
            })),
        );
        validate_normalize_abundance_report_identity(tool, &report)?;
        validate_normalize_abundance_report_metrics(&report, &metric_set.metrics)?;
        validate_normalize_abundance_report_execution(
            &report,
            execution.runtime_s,
            execution.memory_mb,
        )?;
        validate_normalize_abundance_report_composition(&report)?;
        write_normalize_abundance_artifacts(&out_dir, &outputs, &report, &metric_set)?;
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
        insert_fastq_normalize_abundance_v1(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome { records, failures, bench_dir, explain: args.explain })
}

fn resolve_normalize_abundance_outputs(
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<NormalizeAbundanceOutputs> {
    let outputs = NormalizeAbundanceOutputs {
        normalized_table: required_output_path(plan, "normalized_abundance_tsv")?,
        report_json: required_output_path(plan, "report_json")?,
    };
    validate_normalize_abundance_output_paths(&outputs)?;
    Ok(outputs)
}

fn validate_normalize_abundance_output_paths(outputs: &NormalizeAbundanceOutputs) -> Result<()> {
    if outputs.normalized_table == outputs.report_json {
        return Err(anyhow!(
            "normalize_abundance output path reused: {}",
            outputs.normalized_table.display()
        ));
    }
    Ok(())
}

fn required_output_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    artifact_name: &str,
) -> Result<std::path::PathBuf> {
    output_path_for(plan, artifact_name)
        .ok_or_else(|| anyhow!("normalize_abundance plan missing {artifact_name} output"))
}

fn validate_normalize_abundance_report_identity(
    tool: &str,
    report: &NormalizeAbundanceReportV1,
) -> Result<()> {
    if report.schema_version != NORMALIZE_ABUNDANCE_REPORT_SCHEMA_VERSION {
        return Err(anyhow!(
            "normalize_abundance report schema mismatch: expected {}, observed {}",
            NORMALIZE_ABUNDANCE_REPORT_SCHEMA_VERSION,
            report.schema_version
        ));
    }
    if report.stage != STAGE_ID || report.stage_id != STAGE_ID {
        return Err(anyhow!(
            "normalize_abundance report stage mismatch: observed stage={} stage_id={}",
            report.stage,
            report.stage_id
        ));
    }
    if report.tool_id != tool {
        return Err(anyhow!(
            "normalize_abundance report tool mismatch: expected {}, observed {}",
            tool,
            report.tool_id
        ));
    }
    Ok(())
}

fn validate_normalize_abundance_report_metrics(
    report: &NormalizeAbundanceReportV1,
    metrics: &FastqNormalizeAbundanceMetrics,
) -> Result<()> {
    if report.table_rows != metrics.table_rows {
        return Err(anyhow!(
            "normalize_abundance report table_rows mismatch: expected {}, observed {}",
            metrics.table_rows,
            report.table_rows
        ));
    }
    if report.sample_count != metrics.sample_count {
        return Err(anyhow!(
            "normalize_abundance report sample_count mismatch: expected {}, observed {}",
            metrics.sample_count,
            report.sample_count
        ));
    }
    if (report.zero_fraction - metrics.zero_fraction).abs() > f64::EPSILON {
        return Err(anyhow!(
            "normalize_abundance report zero_fraction mismatch: expected {}, observed {}",
            metrics.zero_fraction,
            report.zero_fraction
        ));
    }
    if report.method != metrics.normalization_method {
        return Err(anyhow!(
            "normalize_abundance report method mismatch: expected {}, observed {}",
            metrics.normalization_method,
            report.method
        ));
    }
    Ok(())
}

fn validate_normalize_abundance_report_execution(
    report: &NormalizeAbundanceReportV1,
    runtime_s: f64,
    memory_mb: f64,
) -> Result<()> {
    if report.runtime_s.is_none_or(|observed| (observed - runtime_s).abs() > f64::EPSILON) {
        return Err(anyhow!(
            "normalize_abundance report runtime mismatch: expected {}, observed {:?}",
            runtime_s,
            report.runtime_s
        ));
    }
    if report.memory_mb.is_none_or(|observed| (observed - memory_mb).abs() > f64::EPSILON) {
        return Err(anyhow!(
            "normalize_abundance report memory mismatch: expected {}, observed {:?}",
            memory_mb,
            report.memory_mb
        ));
    }
    Ok(())
}

fn validate_normalize_abundance_report_composition(
    report: &NormalizeAbundanceReportV1,
) -> Result<()> {
    if report.per_sample_sums.len() as u64 != report.sample_count {
        return Err(anyhow!(
            "normalize_abundance report sample sum count mismatch: expected {}, observed {}",
            report.sample_count,
            report.per_sample_sums.len()
        ));
    }
    let expected_sum = report.scale_factor.unwrap_or(1.0);
    for (sample_id, sum) in &report.per_sample_sums {
        if !sum.is_finite() || *sum < 0.0 {
            return Err(anyhow!(
                "normalize_abundance report invalid sample sum for {sample_id}: {sum}"
            ));
        }
        if *sum > 0.0 && (*sum - expected_sum).abs() > 1e-3 {
            return Err(anyhow!(
                "normalize_abundance report sample sum mismatch for {sample_id}: expected {expected_sum}, observed {sum}"
            ));
        }
    }
    Ok(())
}

fn write_normalize_abundance_artifacts(
    out_dir: &Path,
    outputs: &NormalizeAbundanceOutputs,
    report: &NormalizeAbundanceReportV1,
    metric_set: &bijux_dna_analyze::MetricSet<FastqNormalizeAbundanceMetrics>,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(&outputs.report_json, report)?;
    bijux_dna_infra::atomic_write_json(
        &out_dir.join("metrics.json"),
        &serde_json::to_value(metric_set)?,
    )?;
    validate_normalize_abundance_written_artifacts(out_dir, outputs)
}

fn validate_normalize_abundance_written_artifacts(
    out_dir: &Path,
    outputs: &NormalizeAbundanceOutputs,
) -> Result<()> {
    let metrics_json = out_dir.join("metrics.json");
    for path in
        [outputs.normalized_table.as_path(), outputs.report_json.as_path(), metrics_json.as_path()]
    {
        let metadata = std::fs::metadata(path)
            .with_context(|| format!("read normalize_abundance artifact {}", path.display()))?;
        if metadata.len() == 0 {
            return Err(anyhow!("normalize_abundance artifact is empty: {}", path.display()));
        }
    }
    Ok(())
}

pub(crate) fn materialize_normalized_table(
    input: &Path,
    output: &Path,
    effective_params: &AbundanceNormalizationEffectiveParams,
) -> Result<NormalizedAbundanceTableMetrics> {
    let raw =
        std::fs::read_to_string(input).with_context(|| format!("read {}", input.display()))?;
    let expected_header = effective_params.expected_columns.join("\t");
    if let Some(header) = raw.lines().next() {
        if header.trim() != expected_header {
            return Err(anyhow!(
                "abundance table header mismatch: expected `{expected_header}`, got `{header}`"
            ));
        }
    }
    let mut by_sample = BTreeMap::<String, Vec<(String, f64)>>::new();
    let mut feature_ids = BTreeSet::<String>::new();
    for line in raw.lines().skip(1) {
        let cols = line.split('\t').collect::<Vec<_>>();
        if cols.len() < 3 {
            continue;
        }
        let sample = cols[0].trim().to_string();
        let feature = cols[1].trim().to_string();
        let abundance = cols[2].trim().parse::<f64>().unwrap_or(0.0);
        feature_ids.insert(feature.clone());
        by_sample.entry(sample).or_default().push((feature, abundance));
    }
    let mut out = format!("sample_id\tfeature_id\t{}\n", effective_params.normalized_value_column);
    let scale_factor = effective_params.scale_factor.unwrap_or(1.0);
    let sample_count = by_sample.len() as u64;
    let mut table_rows = 0_u64;
    let mut zeros = 0_u64;
    let mut per_sample_sums = Vec::new();
    for (sample, rows) in by_sample {
        let total = rows.iter().map(|(_, abundance)| *abundance).sum::<f64>();
        let mut sample_sum = 0.0_f64;
        for (feature, abundance) in rows {
            let normalized = if total > 0.0 { (abundance * scale_factor) / total } else { 0.0 };
            table_rows += 1;
            if normalized == 0.0 {
                zeros += 1;
            }
            sample_sum += normalized;
            out.push_str(&sample);
            out.push('\t');
            out.push_str(&feature);
            out.push('\t');
            let normalized_text = format!("{normalized:.6}");
            out.push_str(&normalized_text);
            out.push('\n');
        }
        per_sample_sums.push((sample, sample_sum));
    }
    bijux_dna_infra::atomic_write_bytes(output, out.as_bytes())?;
    Ok(NormalizedAbundanceTableMetrics {
        table_rows,
        sample_count,
        feature_count: feature_ids.len() as u64,
        zero_fraction: if table_rows == 0 {
            0.0
        } else {
            u64_to_f64(zeros) / u64_to_f64(table_rows)
        },
        per_sample_sums,
    })
}

pub(crate) fn read_normalized_table_metrics(
    path: &Path,
    effective_params: &AbundanceNormalizationEffectiveParams,
) -> Result<NormalizedAbundanceTableMetrics> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let expected_header =
        format!("sample_id\tfeature_id\t{}", effective_params.normalized_value_column);
    if let Some(header) = raw.lines().next() {
        if header.trim() != expected_header {
            return Err(anyhow!(
                "normalized abundance table header mismatch: expected `{expected_header}`, got `{header}`"
            ));
        }
    }
    let mut rows = 0_u64;
    let mut zeros = 0_u64;
    let mut samples = BTreeMap::<String, f64>::new();
    let mut feature_ids = BTreeSet::<String>::new();
    for line in raw.lines().skip(1) {
        let cols = line.split('\t').collect::<Vec<_>>();
        if cols.len() < 3 {
            continue;
        }
        rows += 1;
        feature_ids.insert(cols[1].trim().to_string());
        let value = cols[2].trim().parse::<f64>().unwrap_or(0.0);
        *samples.entry(cols[0].trim().to_string()).or_default() += value;
        if value == 0.0 {
            zeros += 1;
        }
    }
    Ok(NormalizedAbundanceTableMetrics {
        table_rows: rows,
        sample_count: samples.len() as u64,
        feature_count: feature_ids.len() as u64,
        zero_fraction: if rows == 0 { 0.0 } else { u64_to_f64(zeros) / u64_to_f64(rows) },
        per_sample_sums: samples.into_iter().collect(),
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn canonical_normalize_abundance_report(
    stage_id: &str,
    tool_id: &str,
    input_table: &Path,
    normalized_table: &Path,
    effective_params: &AbundanceNormalizationEffectiveParams,
    table_metrics: &NormalizedAbundanceTableMetrics,
    execution: Option<&ExecutionMetrics>,
    raw_backend_report: Option<String>,
    raw_backend_report_format: Option<String>,
    used_fallback: bool,
    backend_metrics: Option<serde_json::Value>,
) -> NormalizeAbundanceReportV1 {
    NormalizeAbundanceReportV1 {
        schema_version: NORMALIZE_ABUNDANCE_REPORT_SCHEMA_VERSION.to_string(),
        stage: stage_id.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        method: effective_params.method.clone(),
        input_table: input_table.display().to_string(),
        normalized_abundance_tsv: normalized_table.display().to_string(),
        expected_columns: effective_params.expected_columns.clone(),
        input_value_column: effective_params.input_value_column.clone(),
        normalized_value_column: effective_params.normalized_value_column.clone(),
        compositional_rule: effective_params.compositional_rule.clone(),
        scale_factor: effective_params.scale_factor,
        table_rows: table_metrics.table_rows,
        sample_count: table_metrics.sample_count,
        feature_count: table_metrics.feature_count,
        zero_fraction: table_metrics.zero_fraction,
        per_sample_sums: table_metrics.per_sample_sums.clone(),
        runtime_s: execution.map(|metrics| metrics.runtime_s),
        memory_mb: execution.map(|metrics| metrics.memory_mb),
        raw_backend_report,
        raw_backend_report_format,
        used_fallback,
        backend_metrics,
    }
}

fn output_path_for(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    artifact_name: &str,
) -> Option<std::path::PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_name)
        .map(|artifact| artifact.path.clone())
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

/// Materialize the governed local-smoke `fastq.normalize_abundance` artifact bundle.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_normalize_abundance_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases =
        bijux_dna_planner_fastq::stage_api::local_normalize_abundance_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "governed fastq.normalize_abundance local smoke must resolve exactly one case"
        ));
    };

    let output_root = repo_root.join("target/local-smoke/fastq.normalize_abundance");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let summary = materialize_local_normalize_abundance_smoke_case(&repo_root, case, &output_root)?;
    let report_path = output_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    Ok(output_root.join("normalized_abundance.tsv"))
}

fn materialize_local_normalize_abundance_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_fastq::LocalNormalizeAbundanceSmokeCasePlan,
    output_root: &Path,
) -> Result<LocalNormalizeAbundanceSmokeReport> {
    let effective_params = serde_json::from_value::<AbundanceNormalizationEffectiveParams>(
        case.plan.effective_params.clone(),
    )
    .map_err(|error| anyhow!("decode normalize_abundance local-smoke effective params: {error}"))?;

    let input_table = repo_root.join(&case.abundance_table);
    let outputs = resolve_normalize_abundance_outputs(&case.plan)?;
    let case_normalized_table = resolve_smoke_output_path(repo_root, &outputs.normalized_table);
    let case_report_json = resolve_smoke_output_path(repo_root, &outputs.report_json);

    for path in [&case_normalized_table, &case_report_json] {
        if let Some(parent) = path.parent() {
            bijux_dna_infra::ensure_dir(parent)?;
        }
    }

    let runtime_report = bijux_dna_domain_fastq::stages::contract::normalize_abundance(
        &input_table,
        &effective_params,
        &case_normalized_table,
    )?;
    let table_metrics = read_normalized_table_metrics(&case_normalized_table, &effective_params)?;

    let mut report = canonical_normalize_abundance_report(
        STAGE_ID,
        "bijux",
        &input_table,
        &case_normalized_table,
        &effective_params,
        &table_metrics,
        None,
        None,
        None,
        false,
        runtime_report.backend_metrics.clone(),
    );
    report.input_table = case.abundance_table.display().to_string();
    report.normalized_abundance_tsv = path_relative_to_repo(repo_root, &case_normalized_table);
    bijux_dna_infra::atomic_write_json(&case_report_json, &report)?;

    let top_level_table = output_root.join("normalized_abundance.tsv");
    copy_smoke_artifact(&case_normalized_table, &top_level_table)?;

    Ok(LocalNormalizeAbundanceSmokeReport {
        schema_version: LOCAL_NORMALIZE_ABUNDANCE_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: STAGE_ID.to_string(),
        sample_id: case.sample_id.clone(),
        planned_tool_id: case.plan.tool_id.as_str().to_string(),
        report_tool_id: report.tool_id,
        method: report.method,
        table_rows: report.table_rows,
        sample_count: report.sample_count,
        feature_count: report.feature_count,
        zero_fraction: report.zero_fraction,
        normalized_abundance_tsv: path_relative_to_repo(repo_root, &top_level_table),
        case_report_json: path_relative_to_repo(repo_root, &case_report_json),
    })
}

fn resolve_smoke_output_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

fn copy_smoke_artifact(source: &Path, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    std::fs::copy(source, destination).map(|_| ()).with_context(|| {
        format!(
            "copy local normalize_abundance artifact {} -> {}",
            source.display(),
            destination.display()
        )
    })
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::{materialize_normalized_table, read_normalized_table_metrics};
    use anyhow::Result;
    use bijux_dna_domain_fastq::params::edna::AbundanceNormalizationEffectiveParams;
    use bijux_dna_testkit::TestPaths;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        let test_paths = TestPaths::new("normalize-abundance");
        test_paths.child(name)
    }

    #[test]
    fn materialized_table_uses_declared_output_column() -> Result<()> {
        let input = temp_path("input.tsv");
        let output = temp_path("output.tsv");
        bijux_dna_infra::ensure_dir(input.parent().expect("temp parent"))?;
        bijux_dna_infra::write_bytes(
            &input,
            "sample_id\tfeature_id\tabundance\ns1\tf1\t2\ns1\tf2\t2\n",
        )?;

        let effective_params = AbundanceNormalizationEffectiveParams {
            schema_version: "bijux.params.edna.v1".to_string(),
            method: "relative_abundance".to_string(),
            expected_columns: vec![
                "sample_id".to_string(),
                "feature_id".to_string(),
                "abundance".to_string(),
            ],
            input_value_column: "abundance".to_string(),
            normalized_value_column: "relative_abundance_value".to_string(),
            compositional_rule: "per_sample_sum_to_one".to_string(),
            scale_factor: None,
            report_artifact: "report_json".to_string(),
        };

        let metrics = materialize_normalized_table(&input, &output, &effective_params)?;

        let raw = std::fs::read_to_string(&output)?;
        assert!(raw.starts_with("sample_id\tfeature_id\trelative_abundance_value\n"));
        assert_eq!(metrics.sample_count, 1);
        assert_eq!(metrics.feature_count, 2);
        Ok(())
    }

    #[test]
    fn metrics_reader_rejects_header_drift() {
        let output = temp_path("bad_output.tsv");
        bijux_dna_infra::ensure_dir(output.parent().expect("temp parent"))
            .expect("create temp dir");
        bijux_dna_infra::write_bytes(
            &output,
            "sample_id\tfeature_id\tnormalized_abundance\ns1\tf1\t1.0\n",
        )
        .expect("write bad normalized table");

        let effective_params = AbundanceNormalizationEffectiveParams {
            schema_version: "bijux.params.edna.v1".to_string(),
            method: "relative_abundance".to_string(),
            expected_columns: vec![
                "sample_id".to_string(),
                "feature_id".to_string(),
                "abundance".to_string(),
            ],
            input_value_column: "abundance".to_string(),
            normalized_value_column: "relative_abundance_value".to_string(),
            compositional_rule: "per_sample_sum_to_one".to_string(),
            scale_factor: None,
            report_artifact: "report_json".to_string(),
        };

        let error = read_normalized_table_metrics(&output, &effective_params)
            .expect_err("header mismatch must be rejected");
        assert!(error.to_string().contains("header mismatch"));
    }

    #[test]
    fn materialized_table_honors_counts_per_million_scale() -> Result<()> {
        let input = temp_path("counts_input.tsv");
        let output = temp_path("counts_output.tsv");
        bijux_dna_infra::ensure_dir(input.parent().expect("temp parent"))?;
        bijux_dna_infra::write_bytes(
            &input,
            "sample_id\tfeature_id\tabundance\ns1\tf1\t25\ns1\tf2\t75\n",
        )?;

        let effective_params = AbundanceNormalizationEffectiveParams {
            schema_version: "bijux.params.edna.v1".to_string(),
            method: "counts_per_million".to_string(),
            expected_columns: vec![
                "sample_id".to_string(),
                "feature_id".to_string(),
                "abundance".to_string(),
            ],
            input_value_column: "abundance".to_string(),
            normalized_value_column: "counts_per_million".to_string(),
            compositional_rule: "per_sample_sum_to_one_million".to_string(),
            scale_factor: Some(1_000_000.0),
            report_artifact: "report_json".to_string(),
        };

        let metrics = materialize_normalized_table(&input, &output, &effective_params)?;
        let raw = std::fs::read_to_string(&output)?;
        assert!(raw.contains("s1\tf1\t250000.000000"));
        assert!(raw.contains("s1\tf2\t750000.000000"));
        assert_eq!(metrics.per_sample_sums, vec![("s1".to_string(), 1_000_000.0)]);
        Ok(())
    }
}
