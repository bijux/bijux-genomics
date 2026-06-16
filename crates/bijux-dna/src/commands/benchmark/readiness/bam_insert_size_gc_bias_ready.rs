use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::bam_adapter_output_contract::{
    render_bam_adapter_output_contract, BamAdapterOutputContractRow,
    BamAdapterOutputContractStatus, DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH,
};
use super::bam_command_adapter_coverage::{
    render_bam_command_adapter_coverage, BamAdapterCoverageKind, BamBenchmarkStatus,
    BamCommandAdapterCoverageRow, BamReadinessGapKind, DEFAULT_BAM_COMMAND_ADAPTER_COVERAGE_PATH,
};
use super::bam_report_map::{render_bam_report_map, BamReportMapRow, DEFAULT_BAM_REPORT_MAP_PATH};
use super::expected_benchmark_results::{
    render_expected_benchmark_results, ExpectedBenchmarkResultRow,
    DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use super::tool_serving_map::{
    render_bam_tool_serving_map, ToolServingMapRow, DEFAULT_BAM_TOOL_SERVING_MAP_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_INSERT_SIZE_GC_BIAS_READY_PATH: &str =
    "benchmarks/readiness/bam/insert-size-gc-bias-ready.json";
const BAM_INSERT_SIZE_GC_BIAS_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_insert_size_gc_bias_ready.v1";
const COVERAGE_STATUS_COMPLETE: &str = "complete";
const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
const CHECKED_SURFACE_COUNT: usize = 8;
const REQUIRED_REPORT_SECTION_ID: &str = "coverage_quality";
const REQUIRED_SUMMARY_TABLE_ID: &str = "coverage_bias_qc";
const EXPECTED_SCHEMA_REQUIRED_KEYS: [&str; 9] = [
    "schema_version",
    "stage_id",
    "tool_id",
    "tool_version",
    "execution",
    "outputs_count",
    "artifacts",
    "contracts",
    "normalized_keys",
];

#[derive(Debug, Clone, Copy)]
struct BamStageSpec {
    stage_id: &'static str,
    tool_id: &'static str,
    required_output_ids: &'static [&'static str],
    normalized_metrics_output_id: &'static str,
    schema_extension_id: &'static str,
    required_local_smoke_fields: &'static [&'static str],
}

const INSERT_SIZE_REQUIRED_OUTPUT_IDS: [&str; 4] =
    ["insert_size_report", "insert_size_histogram", "summary", "stage_metrics"];
const GC_BIAS_REQUIRED_OUTPUT_IDS: [&str; 4] =
    ["gc_bias_report", "gc_bias_plot", "summary", "stage_metrics"];

const INSERT_SIZE_REQUIRED_LOCAL_SMOKE_FIELDS: [&str; 6] = [
    "read_pairs",
    "median_insert_size",
    "mean_insert_size",
    "standard_deviation",
    "min_insert_size",
    "max_insert_size",
];
const GC_BIAS_REQUIRED_LOCAL_SMOKE_FIELDS: [&str; 7] = [
    "gc_bin",
    "normalized_coverage",
    "windows",
    "read_starts",
    "gc_bias_score",
    "at_dropout",
    "gc_dropout",
];

const BAM_STAGE_SPECS: [BamStageSpec; 2] = [
    BamStageSpec {
        stage_id: "bam.gc_bias",
        tool_id: "picard",
        required_output_ids: &GC_BIAS_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "gc_bias_report",
        schema_extension_id: "bam_gc_bias_normalized_v1",
        required_local_smoke_fields: &GC_BIAS_REQUIRED_LOCAL_SMOKE_FIELDS,
    },
    BamStageSpec {
        stage_id: "bam.insert_size",
        tool_id: "picard",
        required_output_ids: &INSERT_SIZE_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "insert_size_report",
        schema_extension_id: "bam_insert_size_normalized_v1",
        required_local_smoke_fields: &INSERT_SIZE_REQUIRED_LOCAL_SMOKE_FIELDS,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone)]
struct BamStageSchemaContract {
    extension_id: String,
    required_keys: Vec<String>,
}

#[derive(Debug, Clone)]
struct LocalSmokeProof {
    proof_path: String,
    sample_id: String,
    artifact_paths: Vec<String>,
    observed_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamInsertSizeGcBiasReadyRow {
    pub(crate) result_id: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) sample_scope: String,
    pub(crate) benchmark_status: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
    pub(crate) report_section_id: String,
    pub(crate) summary_table_id: String,
    pub(crate) command_readiness_kind: String,
    pub(crate) required_output_ids: Vec<String>,
    pub(crate) stage_output_ids: Vec<String>,
    pub(crate) expected_normalized_metrics_output_id: String,
    pub(crate) normalized_metrics_output_id: Option<String>,
    pub(crate) expected_schema_extension_id: String,
    pub(crate) schema_extension_id: String,
    pub(crate) required_schema_keys: Vec<String>,
    pub(crate) schema_required_keys: Vec<String>,
    pub(crate) required_local_smoke_fields: Vec<String>,
    pub(crate) local_smoke_sample_id: String,
    pub(crate) local_smoke_artifact_paths: Vec<String>,
    pub(crate) local_smoke_observed_fields: Vec<String>,
    pub(crate) active_scope_proof_path: String,
    pub(crate) command_proof_path: String,
    pub(crate) output_contract_proof_path: String,
    pub(crate) parser_proof_path: String,
    pub(crate) expected_result_proof_path: String,
    pub(crate) report_map_proof_path: String,
    pub(crate) schema_proof_path: String,
    pub(crate) local_smoke_proof_path: String,
    pub(crate) active_scope_ready: bool,
    pub(crate) command_ready: bool,
    pub(crate) output_ready: bool,
    pub(crate) parser_ready: bool,
    pub(crate) expected_result_ready: bool,
    pub(crate) report_ready: bool,
    pub(crate) schema_ready: bool,
    pub(crate) local_smoke_ready: bool,
    pub(crate) coverage_status: String,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamInsertSizeGcBiasReadyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) active_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) incomplete_row_count: usize,
    pub(crate) checked_surface_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) expected_tool_ids_by_stage: BTreeMap<String, Vec<String>>,
    pub(crate) required_output_ids_by_stage: BTreeMap<String, Vec<String>>,
    pub(crate) required_local_smoke_fields_by_stage: BTreeMap<String, Vec<String>>,
    pub(crate) coverage_status_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<BamInsertSizeGcBiasReadyRow>,
    pub(crate) violations: Vec<BamInsertSizeGcBiasReadyRow>,
}

pub(crate) fn run_render_bam_insert_size_gc_bias_ready(
    args: &parse::BenchReadinessRenderBamInsertSizeGcBiasReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_insert_size_gc_bias_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_INSERT_SIZE_GC_BIAS_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_insert_size_gc_bias_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamInsertSizeGcBiasReadyReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let report = build_bam_insert_size_gc_bias_ready_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam insert-size and gc-bias readiness must keep active scope, command, output, parser, expected-result, report, schema, and local-smoke proof"
        ));
    }
    Ok(report)
}

fn build_bam_insert_size_gc_bias_ready_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<BamInsertSizeGcBiasReadyReport> {
    let active_scope_report =
        render_bam_tool_serving_map(repo_root, PathBuf::from(DEFAULT_BAM_TOOL_SERVING_MAP_PATH))?;
    let command_report = render_bam_command_adapter_coverage(
        repo_root,
        PathBuf::from(DEFAULT_BAM_COMMAND_ADAPTER_COVERAGE_PATH),
    )?;
    let output_report = render_bam_adapter_output_contract(
        repo_root,
        PathBuf::from(DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH),
    )?;
    let parser_report = super::bam_parser_coverage::render_bam_parser_coverage(
        repo_root,
        PathBuf::from(super::bam_parser_coverage::DEFAULT_BAM_PARSER_COVERAGE_PATH),
    )?;
    let expected_report = render_expected_benchmark_results(
        repo_root,
        PathBuf::from(DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH),
    )?;
    let report_map_report =
        render_bam_report_map(repo_root, PathBuf::from(DEFAULT_BAM_REPORT_MAP_PATH))?;
    let schema_contracts = collect_bam_stage_schema_contracts()?;
    let local_smoke_by_stage = collect_local_smoke_proofs(repo_root)?;

    let active_rows = active_scope_report
        .rows
        .into_iter()
        .filter(|row| bam_stage_spec(&row.stage_id, &row.tool_id).is_some())
        .collect::<Vec<_>>();
    ensure_expected_bindings(
        "BAM insert-size/gc-bias active rows",
        active_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let command_rows = command_report
        .rows
        .into_iter()
        .filter(|row| bam_stage_spec(&row.stage_id, &row.tool_id).is_some())
        .collect::<Vec<_>>();
    ensure_expected_bindings(
        "BAM insert-size/gc-bias command rows",
        command_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let output_rows = output_report
        .rows
        .into_iter()
        .filter(|row| bam_stage_spec(&row.stage_id, &row.tool_id).is_some())
        .collect::<Vec<_>>();
    ensure_expected_bindings(
        "BAM insert-size/gc-bias output-contract rows",
        output_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let parser_rows = parser_report
        .rows
        .into_iter()
        .filter(|row| bam_stage_spec(&row.stage_id, &row.tool_id).is_some())
        .collect::<Vec<_>>();
    ensure_expected_bindings(
        "BAM insert-size/gc-bias parser rows",
        parser_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let expected_rows = expected_report
        .rows
        .into_iter()
        .filter(|row| row.domain == "bam" && bam_stage_spec(&row.stage_id, &row.tool_id).is_some())
        .collect::<Vec<_>>();
    ensure_expected_bindings(
        "BAM insert-size/gc-bias expected-result rows",
        expected_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let report_map_rows = report_map_report
        .rows
        .into_iter()
        .filter(|row| BAM_STAGE_SPECS.iter().any(|spec| spec.stage_id == row.stage_id))
        .collect::<Vec<_>>();
    ensure_expected_stages(
        "BAM insert-size/gc-bias report-map rows",
        report_map_rows.iter().map(|row| row.stage_id.as_str()),
    )?;
    ensure_expected_stages(
        "BAM insert-size/gc-bias schema rows",
        schema_contracts.keys().map(String::as_str),
    )?;

    let active_by_binding = collect_rows_by_binding(active_rows)?;
    let command_by_binding = collect_rows_by_binding(command_rows)?;
    let output_by_binding = collect_rows_by_binding(output_rows)?;
    let parser_by_binding = collect_rows_by_binding(parser_rows)?;
    let expected_by_binding = collect_rows_by_binding(expected_rows)?;
    let report_map_by_stage = report_map_rows
        .into_iter()
        .map(|row| (row.stage_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::with_capacity(BAM_STAGE_SPECS.len());
    for spec in BAM_STAGE_SPECS {
        let key = binding_key(spec.stage_id, spec.tool_id);
        let active_row = active_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing BAM insert-size/gc-bias active row for `{}` / `{}`",
                spec.stage_id,
                spec.tool_id
            )
        })?;
        let command_row = command_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing BAM insert-size/gc-bias command row for `{}` / `{}`",
                spec.stage_id,
                spec.tool_id
            )
        })?;
        let output_row = output_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing BAM insert-size/gc-bias output-contract row for `{}` / `{}`",
                spec.stage_id,
                spec.tool_id
            )
        })?;
        let parser_row = parser_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing BAM insert-size/gc-bias parser row for `{}` / `{}`",
                spec.stage_id,
                spec.tool_id
            )
        })?;
        let expected_row = expected_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing BAM insert-size/gc-bias expected-result row for `{}` / `{}`",
                spec.stage_id,
                spec.tool_id
            )
        })?;
        let report_map_row = report_map_by_stage.get(spec.stage_id).ok_or_else(|| {
            anyhow!("missing BAM insert-size/gc-bias report-map row for `{}`", spec.stage_id)
        })?;
        let schema_contract = schema_contracts.get(spec.stage_id).ok_or_else(|| {
            anyhow!("missing BAM insert-size/gc-bias schema row for `{}`", spec.stage_id)
        })?;
        let local_smoke = local_smoke_by_stage.get(spec.stage_id).ok_or_else(|| {
            anyhow!("missing BAM insert-size/gc-bias local-smoke proof for `{}`", spec.stage_id)
        })?;

        rows.push(build_bam_insert_size_gc_bias_ready_row(
            active_row,
            command_row,
            output_row,
            parser_row,
            expected_row,
            report_map_row,
            schema_contract,
            local_smoke,
            spec,
        ));
    }

    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));

    let complete_row_count =
        rows.iter().filter(|row| row.coverage_status == COVERAGE_STATUS_COMPLETE).count();
    let incomplete_row_count = rows.len().saturating_sub(complete_row_count);
    let violations = rows
        .iter()
        .filter(|row| row.coverage_status != COVERAGE_STATUS_COMPLETE)
        .cloned()
        .collect::<Vec<_>>();
    let mut coverage_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *coverage_status_counts.entry(row.coverage_status.clone()).or_default() += 1;
    }

    Ok(BamInsertSizeGcBiasReadyReport {
        schema_version: BAM_INSERT_SIZE_GC_BIAS_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: rows.len(),
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: CHECKED_SURFACE_COUNT,
        stage_count: BAM_STAGE_SPECS.len(),
        expected_tool_ids_by_stage: BAM_STAGE_SPECS
            .iter()
            .map(|spec| (spec.stage_id.to_string(), vec![spec.tool_id.to_string()]))
            .collect(),
        required_output_ids_by_stage: BAM_STAGE_SPECS
            .iter()
            .map(|spec| {
                (
                    spec.stage_id.to_string(),
                    spec.required_output_ids.iter().map(|value| (*value).to_string()).collect(),
                )
            })
            .collect(),
        required_local_smoke_fields_by_stage: BAM_STAGE_SPECS
            .iter()
            .map(|spec| {
                (
                    spec.stage_id.to_string(),
                    spec.required_local_smoke_fields
                        .iter()
                        .map(|value| (*value).to_string())
                        .collect(),
                )
            })
            .collect(),
        coverage_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_bam_insert_size_gc_bias_ready_row(
    active_row: &ToolServingMapRow,
    command_row: &BamCommandAdapterCoverageRow,
    output_row: &BamAdapterOutputContractRow,
    parser_row: &super::bam_parser_coverage::BamParserCoverageRow,
    expected_row: &ExpectedBenchmarkResultRow,
    report_map_row: &BamReportMapRow,
    schema_contract: &BamStageSchemaContract,
    local_smoke: &LocalSmokeProof,
    spec: BamStageSpec,
) -> BamInsertSizeGcBiasReadyRow {
    let active_scope_ready = active_row.support_status == "supported"
        && active_row.adapter_status == "runnable"
        && active_row.parser_status == "parser_fixture_validated"
        && active_row.corpus_status.starts_with("fixture:");
    let command_ready = command_row.benchmark_status == BamBenchmarkStatus::BenchmarkReady
        && command_row.adapter_coverage == BamAdapterCoverageKind::Covered
        && command_row.readiness_gap == BamReadinessGapKind::None;
    let output_ready = output_row.output_contract_status
        == BamAdapterOutputContractStatus::Complete
        && output_row.normalized_metrics_output_id.as_deref()
            == Some(spec.normalized_metrics_output_id)
        && spec.required_output_ids.iter().all(|output_id| {
            output_row.stage_output_ids.iter().any(|candidate| candidate == output_id)
        })
        && output_row.stdout_path_template.is_some()
        && output_row.stderr_path_template.is_some()
        && output_row.stage_result_manifest_path_template.is_some();
    let parser_ready =
        parser_row.parser_coverage == super::bam_parser_coverage::BamParserCoverageKind::Covered;
    let expected_result_ready = expected_row.fixture_id == "corpus-01-bam-mini"
        && expected_row.normalized_metrics_output_id.as_deref()
            == Some(spec.normalized_metrics_output_id)
        && !expected_row.result_root.is_empty()
        && !expected_row.stage_result_manifest_path.is_empty()
        && !expected_row.stdout_path.is_empty()
        && !expected_row.stderr_path.is_empty();
    let report_ready = report_map_row.report_section_id == REQUIRED_REPORT_SECTION_ID
        && report_map_row.summary_table_id == REQUIRED_SUMMARY_TABLE_ID
        && report_map_row.anchor_tool_id == spec.tool_id
        && report_map_row.anchor_support_status == "supported";
    let schema_ready = schema_contract.extension_id == spec.schema_extension_id
        && EXPECTED_SCHEMA_REQUIRED_KEYS
            .iter()
            .all(|field| schema_contract.required_keys.iter().any(|candidate| candidate == field));
    let local_smoke_ready = spec
        .required_local_smoke_fields
        .iter()
        .all(|field| local_smoke.observed_fields.iter().any(|candidate| candidate == field))
        && !local_smoke.artifact_paths.is_empty();

    let mut missing_surfaces = Vec::new();
    if !active_scope_ready {
        missing_surfaces.push("active_scope".to_string());
    }
    if !command_ready {
        missing_surfaces.push("command".to_string());
    }
    if !output_ready {
        missing_surfaces.push("output".to_string());
    }
    if !parser_ready {
        missing_surfaces.push("parser".to_string());
    }
    if !expected_result_ready {
        missing_surfaces.push("expected_result".to_string());
    }
    if !report_ready {
        missing_surfaces.push("report".to_string());
    }
    if !schema_ready {
        missing_surfaces.push("schema".to_string());
    }
    if !local_smoke_ready {
        missing_surfaces.push("local_smoke".to_string());
    }

    let coverage_status = if missing_surfaces.is_empty() {
        COVERAGE_STATUS_COMPLETE.to_string()
    } else {
        COVERAGE_STATUS_INCOMPLETE.to_string()
    };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "binding `{}` / `{}` keeps active scope, command, output, parser, expected-result, report, schema, and local-smoke proof",
            spec.stage_id, spec.tool_id
        )
    } else {
        format!(
            "binding `{}` / `{}` is missing readiness proof for {}",
            spec.stage_id,
            spec.tool_id,
            missing_surfaces.join(", ")
        )
    };

    BamInsertSizeGcBiasReadyRow {
        result_id: expected_row.result_row_id.clone(),
        stage_id: spec.stage_id.to_string(),
        tool_id: spec.tool_id.to_string(),
        sample_scope: expected_row.sample_scope.clone(),
        benchmark_status: "benchmark_ready".to_string(),
        support_status: active_row.support_status.clone(),
        adapter_status: active_row.adapter_status.clone(),
        parser_status: active_row.parser_status.clone(),
        corpus_status: active_row.corpus_status.clone(),
        report_section_id: report_map_row.report_section_id.clone(),
        summary_table_id: report_map_row.summary_table_id.clone(),
        command_readiness_kind: expected_row.readiness_kind.clone(),
        required_output_ids: spec
            .required_output_ids
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        stage_output_ids: output_row.stage_output_ids.clone(),
        expected_normalized_metrics_output_id: spec.normalized_metrics_output_id.to_string(),
        normalized_metrics_output_id: output_row.normalized_metrics_output_id.clone(),
        expected_schema_extension_id: spec.schema_extension_id.to_string(),
        schema_extension_id: schema_contract.extension_id.clone(),
        required_schema_keys: EXPECTED_SCHEMA_REQUIRED_KEYS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        schema_required_keys: schema_contract.required_keys.clone(),
        required_local_smoke_fields: spec
            .required_local_smoke_fields
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        local_smoke_sample_id: local_smoke.sample_id.clone(),
        local_smoke_artifact_paths: local_smoke.artifact_paths.clone(),
        local_smoke_observed_fields: local_smoke.observed_fields.clone(),
        active_scope_proof_path: DEFAULT_BAM_TOOL_SERVING_MAP_PATH.to_string(),
        command_proof_path: DEFAULT_BAM_COMMAND_ADAPTER_COVERAGE_PATH.to_string(),
        output_contract_proof_path: DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH.to_string(),
        parser_proof_path: super::bam_parser_coverage::DEFAULT_BAM_PARSER_COVERAGE_PATH.to_string(),
        expected_result_proof_path: DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH.to_string(),
        report_map_proof_path: DEFAULT_BAM_REPORT_MAP_PATH.to_string(),
        schema_proof_path:
            crate::commands::benchmark::schema_paths::DEFAULT_BAM_NORMALIZED_METRICS_SCHEMA_PATH
                .to_string(),
        local_smoke_proof_path: local_smoke.proof_path.clone(),
        active_scope_ready,
        command_ready,
        output_ready,
        parser_ready,
        expected_result_ready,
        report_ready,
        schema_ready,
        local_smoke_ready,
        coverage_status,
        missing_surfaces,
        reason,
    }
}

fn collect_bam_stage_schema_contracts() -> Result<BTreeMap<String, BamStageSchemaContract>> {
    let schema = bijux_dna_api::v1::api::bench::render_bam_normalized_metrics_schema();
    let stage_defs = schema
        .get("$defs")
        .and_then(|value| value.get("stages"))
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| anyhow!("BAM normalized metrics schema is missing object `$defs.stages`"))?;

    let mut contracts = BTreeMap::new();
    for spec in BAM_STAGE_SPECS {
        let stage_contract = stage_defs
            .get(spec.stage_id)
            .ok_or_else(|| anyhow!("BAM normalized metrics schema is missing `{}`", spec.stage_id))?
            .get("allOf")
            .and_then(serde_json::Value::as_array)
            .and_then(|items| items.get(1))
            .ok_or_else(|| {
                anyhow!(
                    "BAM normalized metrics stage `{}` is missing stage extension",
                    spec.stage_id
                )
            })?;
        let extension_id = stage_contract
            .get("x-bijux-extension-id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                anyhow!(
                    "BAM normalized metrics stage `{}` is missing string `x-bijux-extension-id`",
                    spec.stage_id
                )
            })?;
        let required_keys = stage_contract
            .get("required")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| {
                anyhow!(
                    "BAM normalized metrics stage `{}` is missing `required` keys",
                    spec.stage_id
                )
            })?
            .iter()
            .map(|value| {
                value.as_str().map(str::to_string).ok_or_else(|| {
                    anyhow!(
                        "BAM normalized metrics stage `{}` has non-string required key",
                        spec.stage_id
                    )
                })
            })
            .collect::<Result<Vec<_>>>()?;
        contracts.insert(
            spec.stage_id.to_string(),
            BamStageSchemaContract { extension_id: extension_id.to_string(), required_keys },
        );
    }

    Ok(contracts)
}

fn collect_local_smoke_proofs(repo_root: &Path) -> Result<BTreeMap<String, LocalSmokeProof>> {
    let insert_size_report_path =
        bijux_dna_api::v1::api::bam::write_local_insert_size_smoke_report()?;
    let insert_size_payload: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&insert_size_report_path)
            .with_context(|| format!("read {}", insert_size_report_path.display()))?,
    )
    .with_context(|| format!("parse {}", insert_size_report_path.display()))?;

    let insert_size_summary_path = repo_root.join(required_json_path(
        &insert_size_payload,
        "insert_size_summary",
        &insert_size_report_path,
    )?);
    let insert_size_stage_metrics_path = repo_root.join(required_json_path(
        &insert_size_payload,
        "stage_metrics",
        &insert_size_report_path,
    )?);
    let insert_size_summary: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&insert_size_summary_path)
            .with_context(|| format!("read {}", insert_size_summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", insert_size_summary_path.display()))?;
    let insert_size_stage_metrics: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&insert_size_stage_metrics_path)
            .with_context(|| format!("read {}", insert_size_stage_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", insert_size_stage_metrics_path.display()))?;

    let mut insert_size_artifact_paths =
        vec![path_relative_to_repo(repo_root, &insert_size_report_path)];
    for relative in [
        required_json_path(&insert_size_payload, "insert_size_report", &insert_size_report_path)?,
        required_json_path(
            &insert_size_payload,
            "insert_size_histogram",
            &insert_size_report_path,
        )?,
        required_json_path(&insert_size_payload, "insert_size_summary", &insert_size_report_path)?,
        required_json_path(&insert_size_payload, "stage_metrics", &insert_size_report_path)?,
    ] {
        insert_size_artifact_paths.push(ensure_repo_relative_file(repo_root, &relative)?);
    }

    let mut proofs = BTreeMap::new();
    proofs.insert(
        "bam.insert_size".to_string(),
        LocalSmokeProof {
            proof_path: path_relative_to_repo(repo_root, &insert_size_report_path),
            sample_id: required_string(
                &insert_size_payload,
                "sample_id",
                &insert_size_report_path,
            )?,
            artifact_paths: insert_size_artifact_paths,
            observed_fields: collect_local_smoke_fields(&[
                insert_size_payload.clone(),
                insert_size_summary,
                insert_size_stage_metrics,
            ]),
        },
    );

    let gc_bias_tsv_path = bijux_dna_api::v1::api::bam::write_local_gc_bias_smoke_summary()?;
    let gc_bias_tsv = fs::read_to_string(&gc_bias_tsv_path)
        .with_context(|| format!("read {}", gc_bias_tsv_path.display()))?;
    let gc_bias_rows = parse_gc_bias_local_smoke_rows(&gc_bias_tsv, &gc_bias_tsv_path)?;
    let first_gc_bias_row = gc_bias_rows
        .first()
        .ok_or_else(|| anyhow!("{} is missing gc-bias rows", gc_bias_tsv_path.display()))?;
    let gc_bias_summary_path = repo_root.join(required_row_value(
        first_gc_bias_row,
        "gc_bias_summary_json",
        &gc_bias_tsv_path,
    )?);
    let gc_bias_stage_metrics_path =
        repo_root.join(required_row_value(first_gc_bias_row, "stage_metrics", &gc_bias_tsv_path)?);
    let gc_bias_summary: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&gc_bias_summary_path)
            .with_context(|| format!("read {}", gc_bias_summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", gc_bias_summary_path.display()))?;
    let gc_bias_stage_metrics: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&gc_bias_stage_metrics_path)
            .with_context(|| format!("read {}", gc_bias_stage_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", gc_bias_stage_metrics_path.display()))?;

    let mut gc_bias_artifact_paths = Vec::new();
    for relative in [
        required_row_value(first_gc_bias_row, "gc_bias_tsv", &gc_bias_tsv_path)?,
        required_row_value(first_gc_bias_row, "gc_bias_summary_json", &gc_bias_tsv_path)?,
        required_row_value(first_gc_bias_row, "gc_bias_metrics", &gc_bias_tsv_path)?,
        required_row_value(first_gc_bias_row, "gc_bias_plot", &gc_bias_tsv_path)?,
        required_row_value(first_gc_bias_row, "stage_metrics", &gc_bias_tsv_path)?,
    ] {
        gc_bias_artifact_paths.push(ensure_repo_relative_file(repo_root, &relative)?);
    }

    let mut gc_bias_observed_fields = collect_local_smoke_fields(&[
        gc_bias_summary,
        gc_bias_stage_metrics,
        serde_json::json!({
            "gc_bin": gc_bias_rows.iter().map(|row| row.get("gc_bin").cloned().unwrap_or_default()).collect::<Vec<_>>(),
            "normalized_coverage": gc_bias_rows.iter().map(|row| row.get("normalized_coverage").cloned().unwrap_or_default()).collect::<Vec<_>>(),
            "windows": gc_bias_rows.iter().map(|row| row.get("windows").cloned().unwrap_or_default()).collect::<Vec<_>>(),
            "read_starts": gc_bias_rows.iter().map(|row| row.get("read_starts").cloned().unwrap_or_default()).collect::<Vec<_>>(),
        }),
    ]);
    gc_bias_observed_fields.sort();
    gc_bias_observed_fields.dedup();

    proofs.insert(
        "bam.gc_bias".to_string(),
        LocalSmokeProof {
            proof_path: path_relative_to_repo(repo_root, &gc_bias_tsv_path),
            sample_id: required_row_value(first_gc_bias_row, "sample_id", &gc_bias_tsv_path)?,
            artifact_paths: gc_bias_artifact_paths,
            observed_fields: gc_bias_observed_fields,
        },
    );

    Ok(proofs)
}

fn parse_gc_bias_local_smoke_rows(
    body: &str,
    proof_path: &Path,
) -> Result<Vec<BTreeMap<String, String>>> {
    let mut lines = body.lines();
    let header =
        lines.next().ok_or_else(|| anyhow!("{} is missing a header row", proof_path.display()))?;
    let columns = header.split('\t').map(str::to_string).collect::<Vec<_>>();
    let mut rows = Vec::new();

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').map(str::to_string).collect::<Vec<_>>();
        if fields.len() != columns.len() {
            return Err(anyhow!(
                "{} has a malformed gc-bias local-smoke row",
                proof_path.display()
            ));
        }
        let row = columns.iter().cloned().zip(fields.into_iter()).collect::<BTreeMap<_, _>>();
        rows.push(row);
    }

    if !rows.iter().any(|row| row.get("gc_bin").is_some_and(|value| value == "0"))
        || !rows.iter().any(|row| row.get("gc_bin").is_some_and(|value| value == "50"))
        || !rows.iter().any(|row| row.get("gc_bin").is_some_and(|value| value == "100"))
    {
        return Err(anyhow!(
            "{} must keep the governed 0/50/100 gc-bias rows",
            proof_path.display()
        ));
    }

    Ok(rows)
}

fn collect_local_smoke_fields(values: &[serde_json::Value]) -> Vec<String> {
    let mut fields = values
        .iter()
        .filter_map(serde_json::Value::as_object)
        .flat_map(|value| value.keys().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    fields.sort();
    fields
}

fn required_string(payload: &serde_json::Value, key: &str, path: &Path) -> Result<String> {
    payload
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("{} is missing string `{key}`", path.display()))
}

fn required_json_path(payload: &serde_json::Value, key: &str, path: &Path) -> Result<String> {
    let value = required_string(payload, key, path)?;
    if value.is_empty() {
        return Err(anyhow!("{} has empty path field `{key}`", path.display()));
    }
    Ok(value)
}

fn ensure_repo_relative_file(repo_root: &Path, relative: &str) -> Result<String> {
    let path = repo_root.join(relative);
    if !path.is_file() {
        return Err(anyhow!("governed local-smoke artifact is missing: {}", path.display()));
    }
    Ok(relative.to_string())
}

fn required_row_value(row: &BTreeMap<String, String>, key: &str, path: &Path) -> Result<String> {
    let value = row
        .get(key)
        .cloned()
        .ok_or_else(|| anyhow!("{} is missing column `{key}`", path.display()))?;
    if value.is_empty() {
        return Err(anyhow!("{} has empty value for `{key}`", path.display()));
    }
    Ok(value)
}

fn collect_rows_by_binding<T>(rows: Vec<T>) -> Result<BTreeMap<BindingKey, T>>
where
    T: BindingRow,
{
    let mut by_binding = BTreeMap::new();
    for row in rows {
        let key = binding_key(row.stage_id(), row.tool_id());
        if by_binding.insert(key.clone(), row).is_some() {
            return Err(anyhow!(
                "duplicate BAM insert-size/gc-bias binding `{}` / `{}`",
                key.stage_id,
                key.tool_id
            ));
        }
    }
    Ok(by_binding)
}

trait BindingRow {
    fn stage_id(&self) -> &str;
    fn tool_id(&self) -> &str;
}

impl BindingRow for ToolServingMapRow {
    fn stage_id(&self) -> &str {
        &self.stage_id
    }

    fn tool_id(&self) -> &str {
        &self.tool_id
    }
}

impl BindingRow for BamCommandAdapterCoverageRow {
    fn stage_id(&self) -> &str {
        &self.stage_id
    }

    fn tool_id(&self) -> &str {
        &self.tool_id
    }
}

impl BindingRow for BamAdapterOutputContractRow {
    fn stage_id(&self) -> &str {
        &self.stage_id
    }

    fn tool_id(&self) -> &str {
        &self.tool_id
    }
}

impl BindingRow for super::bam_parser_coverage::BamParserCoverageRow {
    fn stage_id(&self) -> &str {
        &self.stage_id
    }

    fn tool_id(&self) -> &str {
        &self.tool_id
    }
}

impl BindingRow for ExpectedBenchmarkResultRow {
    fn stage_id(&self) -> &str {
        &self.stage_id
    }

    fn tool_id(&self) -> &str {
        &self.tool_id
    }
}

fn ensure_expected_bindings<'a>(
    label: &str,
    bindings: impl Iterator<Item = (&'a str, &'a str)>,
) -> Result<()> {
    let observed =
        bindings.map(|(stage_id, tool_id)| binding_key(stage_id, tool_id)).collect::<BTreeSet<_>>();
    let expected = BAM_STAGE_SPECS
        .iter()
        .map(|spec| binding_key(spec.stage_id, spec.tool_id))
        .collect::<BTreeSet<_>>();
    if observed != expected {
        return Err(anyhow!(
            "{} must cover [{}], found [{}]",
            label,
            expected
                .iter()
                .map(|binding| format!("{} / {}", binding.stage_id, binding.tool_id))
                .collect::<Vec<_>>()
                .join(", "),
            observed
                .iter()
                .map(|binding| format!("{} / {}", binding.stage_id, binding.tool_id))
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }
    Ok(())
}

fn ensure_expected_stages<'a>(label: &str, stage_ids: impl Iterator<Item = &'a str>) -> Result<()> {
    let observed = stage_ids.map(str::to_string).collect::<BTreeSet<_>>();
    let expected =
        BAM_STAGE_SPECS.iter().map(|spec| spec.stage_id.to_string()).collect::<BTreeSet<_>>();
    if observed != expected {
        return Err(anyhow!(
            "{} must cover [{}], found [{}]",
            label,
            expected.iter().cloned().collect::<Vec<_>>().join(", "),
            observed.iter().cloned().collect::<Vec<_>>().join(", "),
        ));
    }
    Ok(())
}

fn bam_stage_spec(stage_id: &str, tool_id: &str) -> Option<BamStageSpec> {
    BAM_STAGE_SPECS
        .iter()
        .copied()
        .find(|spec| spec.stage_id == stage_id && spec.tool_id == tool_id)
}

fn binding_key(stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey { stage_id: stage_id.to_string(), tool_id: tool_id.to_string() }
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_insert_size_gc_bias_ready, BAM_INSERT_SIZE_GC_BIAS_READY_SCHEMA_VERSION,
        DEFAULT_BAM_INSERT_SIZE_GC_BIAS_READY_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_bam_insert_size_gc_bias_ready_reports_governed_rows() {
        let root = repo_root();
        let report = render_bam_insert_size_gc_bias_ready(
            &root,
            PathBuf::from(DEFAULT_BAM_INSERT_SIZE_GC_BIAS_READY_PATH),
        )
        .expect("render BAM insert-size/gc-bias readiness");

        assert_eq!(report.schema_version, BAM_INSERT_SIZE_GC_BIAS_READY_SCHEMA_VERSION);
        assert_eq!(report.active_row_count, 2);
        assert_eq!(report.complete_row_count, 2);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 8);
        assert_eq!(report.stage_count, 2);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);
        assert_eq!(report.rows.len(), 2);
        assert!(report.rows.iter().all(|row| {
            row.active_scope_ready
                && row.command_ready
                && row.output_ready
                && row.parser_ready
                && row.expected_result_ready
                && row.report_ready
                && row.schema_ready
                && row.local_smoke_ready
                && row.coverage_status == "complete"
        }));

        let insert_size_row = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.insert_size")
            .expect("bam.insert_size row");
        assert_eq!(insert_size_row.expected_normalized_metrics_output_id, "insert_size_report");
        assert!(insert_size_row
            .required_local_smoke_fields
            .iter()
            .any(|field| field == "read_pairs"));
        assert!(insert_size_row
            .local_smoke_artifact_paths
            .iter()
            .any(|path| path == "runs/bench/local-smoke/bam.insert_size/insert_size.json"));

        let gc_bias_row =
            report.rows.iter().find(|row| row.stage_id == "bam.gc_bias").expect("bam.gc_bias row");
        assert_eq!(gc_bias_row.expected_normalized_metrics_output_id, "gc_bias_report");
        assert!(gc_bias_row
            .required_local_smoke_fields
            .iter()
            .any(|field| field == "gc_bias_score"));
        assert!(gc_bias_row
            .local_smoke_artifact_paths
            .iter()
            .any(|path| path == "runs/bench/local-smoke/bam.gc_bias/gc_bias.tsv"));
    }
}
