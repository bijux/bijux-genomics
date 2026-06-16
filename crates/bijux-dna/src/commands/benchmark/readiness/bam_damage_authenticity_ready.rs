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

pub(crate) const DEFAULT_BAM_DAMAGE_AUTHENTICITY_READY_PATH: &str =
    "benchmarks/readiness/bam/damage-authenticity-ready.json";
const BAM_DAMAGE_AUTHENTICITY_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_damage_authenticity_ready.v1";
const COVERAGE_STATUS_COMPLETE: &str = "complete";
const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
const CHECKED_SURFACE_COUNT: usize = 8;
const EXPECTED_FIXTURE_ID: &str = "corpus-01-adna-damage-mini";
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
struct BamBindingSpec {
    stage_id: &'static str,
    tool_id: &'static str,
    required_output_ids: &'static [&'static str],
    normalized_metrics_output_id: &'static str,
    schema_extension_id: &'static str,
    required_local_smoke_fields: &'static [&'static str],
    report_section_id: &'static str,
    summary_table_id: &'static str,
    report_anchor_tool_id: &'static str,
}

const DAMAGE_REQUIRED_OUTPUT_IDS: [&str; 4] =
    ["damage_report", "terminal_position_metrics", "parser_output", "stage_metrics"];
const AUTHENTICITY_REQUIRED_OUTPUT_IDS: [&str; 3] =
    ["authenticity_report", "summary", "stage_metrics"];

const DAMAGE_REQUIRED_LOCAL_SMOKE_FIELDS: [&str; 8] = [
    "terminal_c_to_t_5p",
    "terminal_g_to_a_3p",
    "short_fragment_fraction",
    "damage_signal",
    "damage_report",
    "terminal_position_metrics",
    "parser_output",
    "stage_metrics",
];
const AUTHENTICITY_REQUIRED_LOCAL_SMOKE_FIELDS: [&str; 13] = [
    "score",
    "confidence",
    "pmd_like_signal_present",
    "consumed_metrics",
    "missing_metrics",
    "authenticity_report",
    "authenticity_summary",
    "authenticity_composite",
    "stage_metrics",
    "damage_unified_metrics",
    "contamination_summary",
    "coverage_regime",
    "mapping_summary",
];

const BAM_BINDING_SPECS: [BamBindingSpec; 9] = [
    BamBindingSpec {
        stage_id: "bam.authenticity",
        tool_id: "authenticct",
        required_output_ids: &AUTHENTICITY_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "authenticity_report",
        schema_extension_id: "bam_authenticity_normalized_v1",
        required_local_smoke_fields: &AUTHENTICITY_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "ancient_signal",
        summary_table_id: "damage_authenticity",
        report_anchor_tool_id: "authenticct",
    },
    BamBindingSpec {
        stage_id: "bam.authenticity",
        tool_id: "damageprofiler",
        required_output_ids: &AUTHENTICITY_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "authenticity_report",
        schema_extension_id: "bam_authenticity_normalized_v1",
        required_local_smoke_fields: &AUTHENTICITY_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "ancient_signal",
        summary_table_id: "damage_authenticity",
        report_anchor_tool_id: "authenticct",
    },
    BamBindingSpec {
        stage_id: "bam.authenticity",
        tool_id: "pmdtools",
        required_output_ids: &AUTHENTICITY_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "authenticity_report",
        schema_extension_id: "bam_authenticity_normalized_v1",
        required_local_smoke_fields: &AUTHENTICITY_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "ancient_signal",
        summary_table_id: "damage_authenticity",
        report_anchor_tool_id: "authenticct",
    },
    BamBindingSpec {
        stage_id: "bam.damage",
        tool_id: "addeam",
        required_output_ids: &DAMAGE_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "damage_report",
        schema_extension_id: "bam_damage_normalized_v1",
        required_local_smoke_fields: &DAMAGE_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "ancient_signal",
        summary_table_id: "damage_authenticity",
        report_anchor_tool_id: "mapdamage2",
    },
    BamBindingSpec {
        stage_id: "bam.damage",
        tool_id: "damageprofiler",
        required_output_ids: &DAMAGE_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "damage_report",
        schema_extension_id: "bam_damage_normalized_v1",
        required_local_smoke_fields: &DAMAGE_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "ancient_signal",
        summary_table_id: "damage_authenticity",
        report_anchor_tool_id: "mapdamage2",
    },
    BamBindingSpec {
        stage_id: "bam.damage",
        tool_id: "mapdamage2",
        required_output_ids: &DAMAGE_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "damage_report",
        schema_extension_id: "bam_damage_normalized_v1",
        required_local_smoke_fields: &DAMAGE_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "ancient_signal",
        summary_table_id: "damage_authenticity",
        report_anchor_tool_id: "mapdamage2",
    },
    BamBindingSpec {
        stage_id: "bam.damage",
        tool_id: "ngsbriggs",
        required_output_ids: &DAMAGE_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "damage_report",
        schema_extension_id: "bam_damage_normalized_v1",
        required_local_smoke_fields: &DAMAGE_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "ancient_signal",
        summary_table_id: "damage_authenticity",
        report_anchor_tool_id: "mapdamage2",
    },
    BamBindingSpec {
        stage_id: "bam.damage",
        tool_id: "pmdtools",
        required_output_ids: &DAMAGE_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "damage_report",
        schema_extension_id: "bam_damage_normalized_v1",
        required_local_smoke_fields: &DAMAGE_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "ancient_signal",
        summary_table_id: "damage_authenticity",
        report_anchor_tool_id: "mapdamage2",
    },
    BamBindingSpec {
        stage_id: "bam.damage",
        tool_id: "pydamage",
        required_output_ids: &DAMAGE_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "damage_report",
        schema_extension_id: "bam_damage_normalized_v1",
        required_local_smoke_fields: &DAMAGE_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "ancient_signal",
        summary_table_id: "damage_authenticity",
        report_anchor_tool_id: "mapdamage2",
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
pub(crate) struct BamDamageAuthenticityReadyRow {
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
pub(crate) struct BamDamageAuthenticityReadyReport {
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
    pub(crate) rows: Vec<BamDamageAuthenticityReadyRow>,
    pub(crate) violations: Vec<BamDamageAuthenticityReadyRow>,
}

pub(crate) fn run_render_bam_damage_authenticity_ready(
    args: &parse::BenchReadinessRenderBamDamageAuthenticityReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_damage_authenticity_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_DAMAGE_AUTHENTICITY_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_damage_authenticity_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamDamageAuthenticityReadyReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let report = build_bam_damage_authenticity_ready_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam damage and authenticity readiness must keep active scope, command, output, parser, expected-result, report, schema, and local-smoke proof"
        ));
    }
    Ok(report)
}

fn build_bam_damage_authenticity_ready_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<BamDamageAuthenticityReadyReport> {
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
        .filter(|row| bam_binding_spec(&row.stage_id, &row.tool_id).is_some())
        .collect::<Vec<_>>();
    ensure_expected_bindings(
        "BAM damage/authenticity active rows",
        active_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let command_rows = command_report
        .rows
        .into_iter()
        .filter(|row| bam_binding_spec(&row.stage_id, &row.tool_id).is_some())
        .collect::<Vec<_>>();
    ensure_expected_bindings(
        "BAM damage/authenticity command rows",
        command_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let output_rows = output_report
        .rows
        .into_iter()
        .filter(|row| bam_binding_spec(&row.stage_id, &row.tool_id).is_some())
        .collect::<Vec<_>>();
    ensure_expected_bindings(
        "BAM damage/authenticity output-contract rows",
        output_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let parser_rows = parser_report
        .rows
        .into_iter()
        .filter(|row| bam_binding_spec(&row.stage_id, &row.tool_id).is_some())
        .collect::<Vec<_>>();
    ensure_expected_bindings(
        "BAM damage/authenticity parser rows",
        parser_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let expected_rows = expected_report
        .rows
        .into_iter()
        .filter(|row| {
            row.domain == "bam" && bam_binding_spec(&row.stage_id, &row.tool_id).is_some()
        })
        .collect::<Vec<_>>();
    ensure_expected_bindings(
        "BAM damage/authenticity expected-result rows",
        expected_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let report_map_rows = report_map_report
        .rows
        .into_iter()
        .filter(|row| expected_stage_ids().contains(row.stage_id.as_str()))
        .collect::<Vec<_>>();
    ensure_expected_stages(
        "BAM damage/authenticity report-map rows",
        report_map_rows.iter().map(|row| row.stage_id.as_str()),
    )?;
    ensure_expected_stages(
        "BAM damage/authenticity schema rows",
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

    let mut rows = Vec::with_capacity(BAM_BINDING_SPECS.len());
    for spec in BAM_BINDING_SPECS {
        let key = binding_key(spec.stage_id, spec.tool_id);
        let active_row = active_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing BAM damage/authenticity active row for `{}` / `{}`",
                spec.stage_id,
                spec.tool_id
            )
        })?;
        let command_row = command_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing BAM damage/authenticity command row for `{}` / `{}`",
                spec.stage_id,
                spec.tool_id
            )
        })?;
        let output_row = output_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing BAM damage/authenticity output-contract row for `{}` / `{}`",
                spec.stage_id,
                spec.tool_id
            )
        })?;
        let parser_row = parser_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing BAM damage/authenticity parser row for `{}` / `{}`",
                spec.stage_id,
                spec.tool_id
            )
        })?;
        let expected_row = expected_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing BAM damage/authenticity expected-result row for `{}` / `{}`",
                spec.stage_id,
                spec.tool_id
            )
        })?;
        let report_map_row = report_map_by_stage.get(spec.stage_id).ok_or_else(|| {
            anyhow!("missing BAM damage/authenticity report-map row for `{}`", spec.stage_id)
        })?;
        let schema_contract = schema_contracts.get(spec.stage_id).ok_or_else(|| {
            anyhow!("missing BAM damage/authenticity schema row for `{}`", spec.stage_id)
        })?;
        let local_smoke = local_smoke_by_stage.get(spec.stage_id).ok_or_else(|| {
            anyhow!("missing BAM damage/authenticity local-smoke proof for `{}`", spec.stage_id)
        })?;

        rows.push(build_bam_damage_authenticity_ready_row(
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

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });

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

    Ok(BamDamageAuthenticityReadyReport {
        schema_version: BAM_DAMAGE_AUTHENTICITY_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: rows.len(),
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: CHECKED_SURFACE_COUNT,
        stage_count: expected_stage_ids().len(),
        expected_tool_ids_by_stage: expected_tool_ids_by_stage(),
        required_output_ids_by_stage: required_output_ids_by_stage(),
        required_local_smoke_fields_by_stage: required_local_smoke_fields_by_stage(),
        coverage_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_bam_damage_authenticity_ready_row(
    active_row: &ToolServingMapRow,
    command_row: &BamCommandAdapterCoverageRow,
    output_row: &BamAdapterOutputContractRow,
    parser_row: &super::bam_parser_coverage::BamParserCoverageRow,
    expected_row: &ExpectedBenchmarkResultRow,
    report_map_row: &BamReportMapRow,
    schema_contract: &BamStageSchemaContract,
    local_smoke: &LocalSmokeProof,
    spec: BamBindingSpec,
) -> BamDamageAuthenticityReadyRow {
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
    let expected_result_ready = expected_row.fixture_id == EXPECTED_FIXTURE_ID
        && expected_row.normalized_metrics_output_id.as_deref()
            == Some(spec.normalized_metrics_output_id)
        && !expected_row.result_root.is_empty()
        && !expected_row.stage_result_manifest_path.is_empty()
        && !expected_row.stdout_path.is_empty()
        && !expected_row.stderr_path.is_empty();
    let report_ready = report_map_row.report_section_id == spec.report_section_id
        && report_map_row.summary_table_id == spec.summary_table_id
        && report_map_row.anchor_tool_id == spec.report_anchor_tool_id
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

    BamDamageAuthenticityReadyRow {
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
    for stage_id in expected_stage_ids() {
        let stage_contract = stage_defs
            .get(stage_id)
            .ok_or_else(|| anyhow!("BAM normalized metrics schema is missing `{stage_id}`"))?
            .get("allOf")
            .and_then(serde_json::Value::as_array)
            .and_then(|items| items.get(1))
            .ok_or_else(|| {
                anyhow!("BAM normalized metrics stage `{stage_id}` is missing stage extension")
            })?;
        let extension_id = stage_contract
            .get("x-bijux-extension-id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                anyhow!(
                    "BAM normalized metrics stage `{stage_id}` is missing string `x-bijux-extension-id`"
                )
            })?;
        let required_keys = stage_contract
            .get("required")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| {
                anyhow!("BAM normalized metrics stage `{stage_id}` is missing `required` keys")
            })?
            .iter()
            .map(|value| {
                value.as_str().map(str::to_string).ok_or_else(|| {
                    anyhow!("BAM normalized metrics stage `{stage_id}` has non-string required key")
                })
            })
            .collect::<Result<Vec<_>>>()?;
        contracts.insert(
            stage_id.to_string(),
            BamStageSchemaContract { extension_id: extension_id.to_string(), required_keys },
        );
    }

    Ok(contracts)
}

fn collect_local_smoke_proofs(repo_root: &Path) -> Result<BTreeMap<String, LocalSmokeProof>> {
    let damage_report_path = bijux_dna_api::v1::api::bam::write_local_damage_smoke_report()?;
    let damage_payload: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&damage_report_path)
            .with_context(|| format!("read {}", damage_report_path.display()))?,
    )
    .with_context(|| format!("parse {}", damage_report_path.display()))?;
    let damage_report_json_path =
        repo_root.join(required_json_path(&damage_payload, "damage_report", &damage_report_path)?);
    let damage_terminal_position_metrics_path = repo_root.join(required_json_path(
        &damage_payload,
        "terminal_position_metrics",
        &damage_report_path,
    )?);
    let damage_parser_output_path =
        repo_root.join(required_json_path(&damage_payload, "parser_output", &damage_report_path)?);
    let damage_stage_metrics_path =
        repo_root.join(required_json_path(&damage_payload, "stage_metrics", &damage_report_path)?);
    let damage_report_json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&damage_report_json_path)
            .with_context(|| format!("read {}", damage_report_json_path.display()))?,
    )
    .with_context(|| format!("parse {}", damage_report_json_path.display()))?;
    let damage_terminal_position_metrics: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&damage_terminal_position_metrics_path)
            .with_context(|| format!("read {}", damage_terminal_position_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", damage_terminal_position_metrics_path.display()))?;
    let damage_parser_output: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&damage_parser_output_path)
            .with_context(|| format!("read {}", damage_parser_output_path.display()))?,
    )
    .with_context(|| format!("parse {}", damage_parser_output_path.display()))?;
    let damage_stage_metrics: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&damage_stage_metrics_path)
            .with_context(|| format!("read {}", damage_stage_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", damage_stage_metrics_path.display()))?;

    let authenticity_report_path =
        bijux_dna_api::v1::api::bam::write_local_authenticity_smoke_report()?;
    let authenticity_payload: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&authenticity_report_path)
            .with_context(|| format!("read {}", authenticity_report_path.display()))?,
    )
    .with_context(|| format!("parse {}", authenticity_report_path.display()))?;
    let authenticity_report_json_path = repo_root.join(required_json_path(
        &authenticity_payload,
        "authenticity_report",
        &authenticity_report_path,
    )?);
    let authenticity_summary_path = repo_root.join(required_json_path(
        &authenticity_payload,
        "authenticity_summary",
        &authenticity_report_path,
    )?);
    let authenticity_composite_path = repo_root.join(required_json_path(
        &authenticity_payload,
        "authenticity_composite",
        &authenticity_report_path,
    )?);
    let authenticity_stage_metrics_path = repo_root.join(required_json_path(
        &authenticity_payload,
        "stage_metrics",
        &authenticity_report_path,
    )?);
    let authenticity_report_json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&authenticity_report_json_path)
            .with_context(|| format!("read {}", authenticity_report_json_path.display()))?,
    )
    .with_context(|| format!("parse {}", authenticity_report_json_path.display()))?;
    let authenticity_summary: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&authenticity_summary_path)
            .with_context(|| format!("read {}", authenticity_summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", authenticity_summary_path.display()))?;
    let authenticity_composite: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&authenticity_composite_path)
            .with_context(|| format!("read {}", authenticity_composite_path.display()))?,
    )
    .with_context(|| format!("parse {}", authenticity_composite_path.display()))?;
    let authenticity_stage_metrics: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&authenticity_stage_metrics_path)
            .with_context(|| format!("read {}", authenticity_stage_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", authenticity_stage_metrics_path.display()))?;

    let mut proofs = BTreeMap::new();
    proofs.insert(
        "bam.damage".to_string(),
        LocalSmokeProof {
            proof_path: path_relative_to_repo(repo_root, &damage_report_path),
            sample_id: required_string(&damage_payload, "sample_id", &damage_report_path)?,
            artifact_paths: [
                path_relative_to_repo(repo_root, &damage_report_path),
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(&damage_payload, "damage_report", &damage_report_path)?,
                )?,
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(
                        &damage_payload,
                        "terminal_position_metrics",
                        &damage_report_path,
                    )?,
                )?,
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(&damage_payload, "parser_output", &damage_report_path)?,
                )?,
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(&damage_payload, "advisory_boundary", &damage_report_path)?,
                )?,
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(&damage_payload, "udg_regime", &damage_report_path)?,
                )?,
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(&damage_payload, "stage_metrics", &damage_report_path)?,
                )?,
            ]
            .into_iter()
            .collect(),
            observed_fields: collect_local_smoke_fields(&[
                damage_payload,
                damage_report_json,
                damage_terminal_position_metrics,
                damage_parser_output,
                damage_stage_metrics,
            ]),
        },
    );

    proofs.insert(
        "bam.authenticity".to_string(),
        LocalSmokeProof {
            proof_path: path_relative_to_repo(repo_root, &authenticity_report_path),
            sample_id: required_string(
                &authenticity_payload,
                "sample_id",
                &authenticity_report_path,
            )?,
            artifact_paths: [
                path_relative_to_repo(repo_root, &authenticity_report_path),
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(
                        &authenticity_payload,
                        "authenticity_report",
                        &authenticity_report_path,
                    )?,
                )?,
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(
                        &authenticity_payload,
                        "authenticity_summary",
                        &authenticity_report_path,
                    )?,
                )?,
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(
                        &authenticity_payload,
                        "authenticity_composite",
                        &authenticity_report_path,
                    )?,
                )?,
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(
                        &authenticity_payload,
                        "advisory_boundary",
                        &authenticity_report_path,
                    )?,
                )?,
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(
                        &authenticity_payload,
                        "stage_metrics",
                        &authenticity_report_path,
                    )?,
                )?,
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(
                        &authenticity_payload,
                        "damage_unified_metrics",
                        &authenticity_report_path,
                    )?,
                )?,
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(
                        &authenticity_payload,
                        "contamination_summary",
                        &authenticity_report_path,
                    )?,
                )?,
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(
                        &authenticity_payload,
                        "complexity_summary",
                        &authenticity_report_path,
                    )?,
                )?,
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(
                        &authenticity_payload,
                        "coverage_regime",
                        &authenticity_report_path,
                    )?,
                )?,
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(
                        &authenticity_payload,
                        "mapping_summary",
                        &authenticity_report_path,
                    )?,
                )?,
            ]
            .into_iter()
            .collect(),
            observed_fields: collect_local_smoke_fields(&[
                authenticity_payload,
                authenticity_report_json,
                authenticity_summary,
                authenticity_composite,
                authenticity_stage_metrics,
            ]),
        },
    );

    Ok(proofs)
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

fn collect_rows_by_binding<T>(rows: Vec<T>) -> Result<BTreeMap<BindingKey, T>>
where
    T: BindingRow,
{
    let mut by_binding = BTreeMap::new();
    for row in rows {
        let key = binding_key(row.stage_id(), row.tool_id());
        if by_binding.insert(key.clone(), row).is_some() {
            return Err(anyhow!(
                "duplicate BAM damage/authenticity binding `{}` / `{}`",
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
    let expected = BAM_BINDING_SPECS
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
    let expected = expected_stage_ids().into_iter().map(str::to_string).collect::<BTreeSet<_>>();
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

fn bam_binding_spec(stage_id: &str, tool_id: &str) -> Option<BamBindingSpec> {
    BAM_BINDING_SPECS
        .iter()
        .copied()
        .find(|spec| spec.stage_id == stage_id && spec.tool_id == tool_id)
}

fn binding_key(stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey { stage_id: stage_id.to_string(), tool_id: tool_id.to_string() }
}

fn expected_stage_ids() -> BTreeSet<&'static str> {
    BAM_BINDING_SPECS.iter().map(|spec| spec.stage_id).collect()
}

fn expected_tool_ids_by_stage() -> BTreeMap<String, Vec<String>> {
    let mut by_stage = BTreeMap::<String, Vec<String>>::new();
    for spec in BAM_BINDING_SPECS {
        by_stage.entry(spec.stage_id.to_string()).or_default().push(spec.tool_id.to_string());
    }
    by_stage
}

fn required_output_ids_by_stage() -> BTreeMap<String, Vec<String>> {
    let mut by_stage = BTreeMap::<String, BTreeSet<String>>::new();
    for spec in BAM_BINDING_SPECS {
        let stage_outputs = by_stage.entry(spec.stage_id.to_string()).or_default();
        for output_id in spec.required_output_ids {
            stage_outputs.insert((*output_id).to_string());
        }
    }
    by_stage
        .into_iter()
        .map(|(stage_id, output_ids)| (stage_id, output_ids.into_iter().collect()))
        .collect()
}

fn required_local_smoke_fields_by_stage() -> BTreeMap<String, Vec<String>> {
    let mut by_stage = BTreeMap::<String, BTreeSet<String>>::new();
    for spec in BAM_BINDING_SPECS {
        let stage_fields = by_stage.entry(spec.stage_id.to_string()).or_default();
        for field in spec.required_local_smoke_fields {
            stage_fields.insert((*field).to_string());
        }
    }
    by_stage
        .into_iter()
        .map(|(stage_id, fields)| (stage_id, fields.into_iter().collect()))
        .collect()
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
        render_bam_damage_authenticity_ready, BAM_DAMAGE_AUTHENTICITY_READY_SCHEMA_VERSION,
        DEFAULT_BAM_DAMAGE_AUTHENTICITY_READY_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_bam_damage_authenticity_ready_reports_governed_rows() {
        let root = repo_root();
        let report = render_bam_damage_authenticity_ready(
            &root,
            PathBuf::from(DEFAULT_BAM_DAMAGE_AUTHENTICITY_READY_PATH),
        )
        .expect("render BAM damage/authenticity readiness");

        assert_eq!(report.schema_version, BAM_DAMAGE_AUTHENTICITY_READY_SCHEMA_VERSION);
        assert_eq!(report.active_row_count, 9);
        assert_eq!(report.complete_row_count, 9);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 8);
        assert_eq!(report.stage_count, 2);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);
        assert_eq!(report.rows.len(), 9);
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

        let authenticity_row = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.authenticity" && row.tool_id == "authenticct")
            .expect("bam.authenticity authenticct row");
        assert_eq!(authenticity_row.expected_normalized_metrics_output_id, "authenticity_report");
        assert_eq!(authenticity_row.report_section_id, "ancient_signal");
        assert_eq!(authenticity_row.summary_table_id, "damage_authenticity");
        assert!(authenticity_row.required_local_smoke_fields.iter().any(|field| field == "score"));
        assert!(authenticity_row
            .required_local_smoke_fields
            .iter()
            .any(|field| field == "mapping_summary"));
        assert!(authenticity_row
            .local_smoke_artifact_paths
            .iter()
            .any(|path| { path == "runs/bench/local-smoke/bam.authenticity/authenticity.json" }));

        let damage_row = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.damage" && row.tool_id == "mapdamage2")
            .expect("bam.damage mapdamage2 row");
        assert_eq!(damage_row.expected_normalized_metrics_output_id, "damage_report");
        assert_eq!(damage_row.report_section_id, "ancient_signal");
        assert_eq!(damage_row.summary_table_id, "damage_authenticity");
        assert!(damage_row
            .required_local_smoke_fields
            .iter()
            .any(|field| field == "terminal_c_to_t_5p"));
        assert!(damage_row
            .required_local_smoke_fields
            .iter()
            .any(|field| field == "damage_report"));
        assert!(damage_row
            .local_smoke_artifact_paths
            .iter()
            .any(|path| path == "runs/bench/local-smoke/bam.damage/damage.json"));
    }
}
