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

pub(crate) const DEFAULT_BAM_RECALIBRATION_GENOTYPING_READY_PATH: &str =
    "benchmarks/readiness/bam/recalibration-genotyping-ready.json";
const BAM_RECALIBRATION_GENOTYPING_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_recalibration_genotyping_ready.v1";
const COVERAGE_STATUS_COMPLETE: &str = "complete";
const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
const CHECKED_SURFACE_COUNT: usize = 8;
const LOCAL_PROOF_KIND_SMOKE: &str = "local_smoke";
const LOCAL_PROOF_KIND_READY_PLAN: &str = "local_ready_plan";
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
    fixture_id: &'static str,
    required_output_ids: &'static [&'static str],
    normalized_metrics_output_id: &'static str,
    schema_extension_id: &'static str,
    expected_local_proof_kind: &'static str,
    expected_local_proof_tool_id: &'static str,
    required_local_proof_fields: &'static [&'static str],
    required_local_proof_output_ids: &'static [&'static str],
    required_local_proof_command_fragments: &'static [&'static str],
    report_section_id: &'static str,
    summary_table_id: &'static str,
    report_anchor_tool_id: &'static str,
}

const RECALIBRATION_REQUIRED_OUTPUT_IDS: [&str; 5] =
    ["recal_bam", "recal_bai", "recal_report", "summary", "stage_metrics"];
#[cfg(feature = "bam_downstream")]
const GENOTYPING_REQUIRED_OUTPUT_IDS: [&str; 3] = ["genotyping_report", "summary", "stage_metrics"];

const RECALIBRATION_REQUIRED_LOCAL_PROOF_FIELDS: [&str; 14] = [
    "requested_mode",
    "effective_mode",
    "status",
    "reason",
    "known_sites",
    "coverage_gate",
    "observed_mean_coverage",
    "observed_breadth_1x",
    "output_bam_present",
    "recalibration_report_present",
    "recalibrated_bam",
    "recalibration_report",
    "recalibration_summary",
    "stage_metrics",
];
#[cfg(feature = "bam_downstream")]
const GENOTYPING_REQUIRED_LOCAL_PROOF_FIELDS: [&str; 9] = [
    "sample_id",
    "producer_contract",
    "min_call_rate",
    "min_posterior",
    "reference",
    "regions",
    "sites",
    "caller",
    "tool",
];

const RECALIBRATION_REQUIRED_LOCAL_PROOF_OUTPUT_IDS: [&str; 4] =
    ["recalibrated_bam", "recalibration_report", "recalibration_summary", "stage_metrics"];
#[cfg(feature = "bam_downstream")]
const GENOTYPING_REQUIRED_LOCAL_PROOF_OUTPUT_IDS: [&str; 7] = [
    "genotyping_report",
    "summary",
    "stage_metrics",
    "genotyping_bcf",
    "genotyping_vcf",
    "genotyping_vcf_tbi",
    "genotyping_gl",
];

#[cfg(feature = "bam_downstream")]
const GENOTYPING_REQUIRED_LOCAL_PROOF_COMMAND_FRAGMENTS: [&str; 7] = [
    "human_like_genotyping_candidate_panel.sam.bai",
    "corpus_01_bam_reference.fasta",
    "human_like_genotyping_candidate_sites.vcf",
    "human_like_genotyping_target_regions.txt",
    "benchmarks/readiness/local-ready/bam.genotyping/genotyping.bcf",
    "benchmarks/readiness/local-ready/bam.genotyping/genotyping.vcf.gz",
    "benchmarks/readiness/local-ready/bam.genotyping/genotyping.gl.json",
];

fn bam_binding_specs() -> Vec<BamBindingSpec> {
    let mut specs = vec![BamBindingSpec {
        stage_id: "bam.recalibration",
        tool_id: "gatk",
        fixture_id: "corpus-01-bam-mini",
        required_output_ids: &RECALIBRATION_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "recal_report",
        schema_extension_id: "bam_recalibration_normalized_v1",
        expected_local_proof_kind: LOCAL_PROOF_KIND_SMOKE,
        expected_local_proof_tool_id: "gatk",
        required_local_proof_fields: &RECALIBRATION_REQUIRED_LOCAL_PROOF_FIELDS,
        required_local_proof_output_ids: &RECALIBRATION_REQUIRED_LOCAL_PROOF_OUTPUT_IDS,
        required_local_proof_command_fragments: &[],
        report_section_id: "downstream_readiness",
        summary_table_id: "variant_readiness",
        report_anchor_tool_id: "gatk",
    }];
    #[cfg(feature = "bam_downstream")]
    specs.push(BamBindingSpec {
        stage_id: "bam.genotyping",
        tool_id: "angsd",
        fixture_id: "corpus-01-genotyping-mini",
        required_output_ids: &GENOTYPING_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "genotyping_report",
        schema_extension_id: "bam_genotyping_normalized_v1",
        expected_local_proof_kind: LOCAL_PROOF_KIND_READY_PLAN,
        expected_local_proof_tool_id: "angsd",
        required_local_proof_fields: &GENOTYPING_REQUIRED_LOCAL_PROOF_FIELDS,
        required_local_proof_output_ids: &GENOTYPING_REQUIRED_LOCAL_PROOF_OUTPUT_IDS,
        required_local_proof_command_fragments: &GENOTYPING_REQUIRED_LOCAL_PROOF_COMMAND_FRAGMENTS,
        report_section_id: "downstream_readiness",
        summary_table_id: "variant_readiness",
        report_anchor_tool_id: "angsd",
    });
    specs
}

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
struct LocalProof {
    proof_kind: String,
    proof_path: String,
    tool_id: String,
    sample_id: String,
    artifact_paths: Vec<String>,
    observed_fields: Vec<String>,
    declared_output_ids: Vec<String>,
    command_shell: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamRecalibrationGenotypingReadyRow {
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
    pub(crate) expected_local_proof_kind: String,
    pub(crate) local_proof_kind: String,
    pub(crate) expected_local_proof_tool_id: String,
    pub(crate) local_proof_tool_id: String,
    pub(crate) required_local_proof_fields: Vec<String>,
    pub(crate) local_proof_sample_id: String,
    pub(crate) local_proof_artifact_paths: Vec<String>,
    pub(crate) local_proof_observed_fields: Vec<String>,
    pub(crate) required_local_proof_output_ids: Vec<String>,
    pub(crate) local_proof_declared_output_ids: Vec<String>,
    pub(crate) required_local_proof_command_fragments: Vec<String>,
    pub(crate) local_proof_command_shell: Option<String>,
    pub(crate) active_scope_proof_path: String,
    pub(crate) command_proof_path: String,
    pub(crate) output_contract_proof_path: String,
    pub(crate) parser_proof_path: String,
    pub(crate) expected_result_proof_path: String,
    pub(crate) report_map_proof_path: String,
    pub(crate) schema_proof_path: String,
    pub(crate) local_proof_path: String,
    pub(crate) active_scope_ready: bool,
    pub(crate) command_ready: bool,
    pub(crate) output_ready: bool,
    pub(crate) parser_ready: bool,
    pub(crate) expected_result_ready: bool,
    pub(crate) report_ready: bool,
    pub(crate) schema_ready: bool,
    pub(crate) local_proof_ready: bool,
    pub(crate) coverage_status: String,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamRecalibrationGenotypingReadyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) active_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) incomplete_row_count: usize,
    pub(crate) checked_surface_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) expected_tool_ids_by_stage: BTreeMap<String, Vec<String>>,
    pub(crate) required_output_ids_by_stage: BTreeMap<String, Vec<String>>,
    pub(crate) expected_local_proof_kind_by_stage: BTreeMap<String, String>,
    pub(crate) required_local_proof_fields_by_stage: BTreeMap<String, Vec<String>>,
    pub(crate) coverage_status_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<BamRecalibrationGenotypingReadyRow>,
    pub(crate) violations: Vec<BamRecalibrationGenotypingReadyRow>,
}

pub(crate) fn run_render_bam_recalibration_genotyping_ready(
    args: &parse::BenchReadinessRenderBamRecalibrationGenotypingReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_recalibration_genotyping_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_RECALIBRATION_GENOTYPING_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_recalibration_genotyping_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamRecalibrationGenotypingReadyReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let report = build_bam_recalibration_genotyping_ready_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam recalibration and genotyping readiness must keep active scope, command, output, parser, expected-result, report, schema, and local proof"
        ));
    }
    Ok(report)
}

fn build_bam_recalibration_genotyping_ready_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<BamRecalibrationGenotypingReadyReport> {
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
    let local_proofs_by_stage = collect_local_proofs(repo_root)?;

    let active_rows = active_scope_report
        .rows
        .into_iter()
        .filter(|row| bam_binding_spec(&row.stage_id, &row.tool_id).is_some())
        .collect::<Vec<_>>();
    ensure_expected_bindings(
        "BAM recalibration/genotyping active rows",
        active_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let command_rows = command_report
        .rows
        .into_iter()
        .filter(|row| bam_binding_spec(&row.stage_id, &row.tool_id).is_some())
        .collect::<Vec<_>>();
    ensure_expected_bindings(
        "BAM recalibration/genotyping command rows",
        command_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let output_rows = output_report
        .rows
        .into_iter()
        .filter(|row| bam_binding_spec(&row.stage_id, &row.tool_id).is_some())
        .collect::<Vec<_>>();
    ensure_expected_bindings(
        "BAM recalibration/genotyping output-contract rows",
        output_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let parser_rows = parser_report
        .rows
        .into_iter()
        .filter(|row| bam_binding_spec(&row.stage_id, &row.tool_id).is_some())
        .collect::<Vec<_>>();
    ensure_expected_bindings(
        "BAM recalibration/genotyping parser rows",
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
        "BAM recalibration/genotyping expected-result rows",
        expected_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let report_map_rows = report_map_report
        .rows
        .into_iter()
        .filter(|row| expected_stage_ids().contains(row.stage_id.as_str()))
        .collect::<Vec<_>>();
    ensure_expected_stages(
        "BAM recalibration/genotyping report-map rows",
        report_map_rows.iter().map(|row| row.stage_id.as_str()),
    )?;
    ensure_expected_stages(
        "BAM recalibration/genotyping schema rows",
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

    let mut rows = Vec::new();
    for spec in bam_binding_specs() {
        let key = binding_key(spec.stage_id, spec.tool_id);
        let active_row = active_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing BAM recalibration/genotyping active row for `{}` / `{}`",
                spec.stage_id,
                spec.tool_id
            )
        })?;
        let command_row = command_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing BAM recalibration/genotyping command row for `{}` / `{}`",
                spec.stage_id,
                spec.tool_id
            )
        })?;
        let output_row = output_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing BAM recalibration/genotyping output-contract row for `{}` / `{}`",
                spec.stage_id,
                spec.tool_id
            )
        })?;
        let parser_row = parser_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing BAM recalibration/genotyping parser row for `{}` / `{}`",
                spec.stage_id,
                spec.tool_id
            )
        })?;
        let expected_row = expected_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing BAM recalibration/genotyping expected-result row for `{}` / `{}`",
                spec.stage_id,
                spec.tool_id
            )
        })?;
        let report_map_row = report_map_by_stage.get(spec.stage_id).ok_or_else(|| {
            anyhow!("missing BAM recalibration/genotyping report-map row for `{}`", spec.stage_id)
        })?;
        let schema_contract = schema_contracts.get(spec.stage_id).ok_or_else(|| {
            anyhow!("missing BAM recalibration/genotyping schema row for `{}`", spec.stage_id)
        })?;
        let local_proof = local_proofs_by_stage.get(spec.stage_id).ok_or_else(|| {
            anyhow!("missing BAM recalibration/genotyping local proof for `{}`", spec.stage_id)
        })?;

        rows.push(build_bam_recalibration_genotyping_ready_row(
            active_row,
            command_row,
            output_row,
            parser_row,
            expected_row,
            report_map_row,
            schema_contract,
            local_proof,
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

    Ok(BamRecalibrationGenotypingReadyReport {
        schema_version: BAM_RECALIBRATION_GENOTYPING_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: rows.len(),
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: CHECKED_SURFACE_COUNT,
        stage_count: expected_stage_ids().len(),
        expected_tool_ids_by_stage: expected_tool_ids_by_stage(),
        required_output_ids_by_stage: required_output_ids_by_stage(),
        expected_local_proof_kind_by_stage: expected_local_proof_kind_by_stage(),
        required_local_proof_fields_by_stage: required_local_proof_fields_by_stage(),
        coverage_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_bam_recalibration_genotyping_ready_row(
    active_row: &ToolServingMapRow,
    command_row: &BamCommandAdapterCoverageRow,
    output_row: &BamAdapterOutputContractRow,
    parser_row: &super::bam_parser_coverage::BamParserCoverageRow,
    expected_row: &ExpectedBenchmarkResultRow,
    report_map_row: &BamReportMapRow,
    schema_contract: &BamStageSchemaContract,
    local_proof: &LocalProof,
    spec: BamBindingSpec,
) -> BamRecalibrationGenotypingReadyRow {
    let fixture_status = format!("fixture:{}", spec.fixture_id);
    let active_scope_ready = active_row.support_status == "supported"
        && active_row.adapter_status == "runnable"
        && active_row.parser_status == "parser_fixture_validated"
        && active_row.corpus_status == fixture_status;
    let command_ready = command_row.benchmark_status == BamBenchmarkStatus::BenchmarkReady
        && command_row.adapter_coverage == BamAdapterCoverageKind::Covered
        && command_row.readiness_gap == BamReadinessGapKind::None
        && command_row.corpus_status == fixture_status;
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
    let parser_ready = parser_row.parser_coverage
        == super::bam_parser_coverage::BamParserCoverageKind::Covered
        && parser_row.corpus_status == fixture_status;
    let expected_result_ready = expected_row.fixture_id == spec.fixture_id
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
    let local_proof_ready = local_proof.proof_kind == spec.expected_local_proof_kind
        && local_proof.tool_id == spec.expected_local_proof_tool_id
        && spec
            .required_local_proof_fields
            .iter()
            .all(|field| local_proof.observed_fields.iter().any(|candidate| candidate == field))
        && spec.required_local_proof_output_ids.iter().all(|output_id| {
            local_proof.declared_output_ids.iter().any(|candidate| candidate == output_id)
        })
        && spec.required_local_proof_command_fragments.iter().all(|fragment| {
            local_proof.command_shell.as_deref().is_some_and(|shell| shell.contains(fragment))
        })
        && !local_proof.artifact_paths.is_empty();

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
    if !local_proof_ready {
        missing_surfaces.push("local_proof".to_string());
    }

    let coverage_status = if missing_surfaces.is_empty() {
        COVERAGE_STATUS_COMPLETE.to_string()
    } else {
        COVERAGE_STATUS_INCOMPLETE.to_string()
    };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "binding `{}` / `{}` keeps active scope, command, output, parser, expected-result, report, schema, and {} proof via `{}`",
            spec.stage_id, spec.tool_id, local_proof.proof_kind, local_proof.tool_id
        )
    } else {
        format!(
            "binding `{}` / `{}` is missing readiness proof for {}",
            spec.stage_id,
            spec.tool_id,
            missing_surfaces.join(", ")
        )
    };

    BamRecalibrationGenotypingReadyRow {
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
        expected_local_proof_kind: spec.expected_local_proof_kind.to_string(),
        local_proof_kind: local_proof.proof_kind.clone(),
        expected_local_proof_tool_id: spec.expected_local_proof_tool_id.to_string(),
        local_proof_tool_id: local_proof.tool_id.clone(),
        required_local_proof_fields: spec
            .required_local_proof_fields
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        local_proof_sample_id: local_proof.sample_id.clone(),
        local_proof_artifact_paths: local_proof.artifact_paths.clone(),
        local_proof_observed_fields: local_proof.observed_fields.clone(),
        required_local_proof_output_ids: spec
            .required_local_proof_output_ids
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        local_proof_declared_output_ids: local_proof.declared_output_ids.clone(),
        required_local_proof_command_fragments: spec
            .required_local_proof_command_fragments
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        local_proof_command_shell: local_proof.command_shell.clone(),
        active_scope_proof_path: DEFAULT_BAM_TOOL_SERVING_MAP_PATH.to_string(),
        command_proof_path: DEFAULT_BAM_COMMAND_ADAPTER_COVERAGE_PATH.to_string(),
        output_contract_proof_path: DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH.to_string(),
        parser_proof_path: super::bam_parser_coverage::DEFAULT_BAM_PARSER_COVERAGE_PATH.to_string(),
        expected_result_proof_path: DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH.to_string(),
        report_map_proof_path: DEFAULT_BAM_REPORT_MAP_PATH.to_string(),
        schema_proof_path:
            crate::commands::benchmark::schema_paths::DEFAULT_BAM_NORMALIZED_METRICS_SCHEMA_PATH
                .to_string(),
        local_proof_path: local_proof.proof_path.clone(),
        active_scope_ready,
        command_ready,
        output_ready,
        parser_ready,
        expected_result_ready,
        report_ready,
        schema_ready,
        local_proof_ready,
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

fn collect_local_proofs(repo_root: &Path) -> Result<BTreeMap<String, LocalProof>> {
    let recalibration_report_path =
        bijux_dna_api::v1::api::bam::write_local_recalibration_smoke_report()?;
    let recalibration_payload: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&recalibration_report_path)
            .with_context(|| format!("read {}", recalibration_report_path.display()))?,
    )
    .with_context(|| format!("parse {}", recalibration_report_path.display()))?;
    let recalibration_summary_path = repo_root.join(required_json_path(
        &recalibration_payload,
        "recalibration_summary",
        &recalibration_report_path,
    )?);
    let recalibration_stage_metrics_path = repo_root.join(required_json_path(
        &recalibration_payload,
        "stage_metrics",
        &recalibration_report_path,
    )?);
    let recalibration_summary_json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&recalibration_summary_path)
            .with_context(|| format!("read {}", recalibration_summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", recalibration_summary_path.display()))?;
    let recalibration_stage_metrics_json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&recalibration_stage_metrics_path)
            .with_context(|| format!("read {}", recalibration_stage_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", recalibration_stage_metrics_path.display()))?;

    let mut proofs = BTreeMap::new();
    proofs.insert(
        "bam.recalibration".to_string(),
        LocalProof {
            proof_kind: LOCAL_PROOF_KIND_SMOKE.to_string(),
            proof_path: path_relative_to_repo(repo_root, &recalibration_report_path),
            tool_id: "gatk".to_string(),
            sample_id: required_string(
                &recalibration_payload,
                "sample_id",
                &recalibration_report_path,
            )?,
            artifact_paths: [
                path_relative_to_repo(repo_root, &recalibration_report_path),
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(
                        &recalibration_payload,
                        "recalibrated_bam",
                        &recalibration_report_path,
                    )?,
                )?,
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(
                        &recalibration_payload,
                        "recalibration_report",
                        &recalibration_report_path,
                    )?,
                )?,
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(
                        &recalibration_payload,
                        "recalibration_summary",
                        &recalibration_report_path,
                    )?,
                )?,
                ensure_repo_relative_file(
                    repo_root,
                    &required_json_path(
                        &recalibration_payload,
                        "stage_metrics",
                        &recalibration_report_path,
                    )?,
                )?,
            ]
            .into_iter()
            .collect(),
            observed_fields: collect_json_fields(&[
                recalibration_payload.clone(),
                recalibration_summary_json,
                recalibration_stage_metrics_json,
            ]),
            declared_output_ids: RECALIBRATION_REQUIRED_LOCAL_PROOF_OUTPUT_IDS
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            command_shell: None,
        },
    );

    #[cfg(feature = "bam_downstream")]
    {
        let genotyping_plan_path = bijux_dna_api::v1::api::bam::write_local_genotyping_plan()?;
        let genotyping_plan: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&genotyping_plan_path)
                .with_context(|| format!("read {}", genotyping_plan_path.display()))?,
        )
        .with_context(|| format!("parse {}", genotyping_plan_path.display()))?;
        proofs.insert(
            "bam.genotyping".to_string(),
            LocalProof {
                proof_kind: LOCAL_PROOF_KIND_READY_PLAN.to_string(),
                proof_path: path_relative_to_repo(repo_root, &genotyping_plan_path),
                tool_id: required_string(&genotyping_plan, "tool_id", &genotyping_plan_path)?,
                sample_id: required_nested_string(
                    &genotyping_plan,
                    &["params", "sample_id"],
                    &genotyping_plan_path,
                )?,
                artifact_paths: collect_plan_artifact_paths(
                    repo_root,
                    &genotyping_plan,
                    &genotyping_plan_path,
                )?,
                observed_fields: collect_json_fields(&[genotyping_plan.clone()]),
                declared_output_ids: collect_plan_output_ids(
                    &genotyping_plan,
                    &genotyping_plan_path,
                )?,
                command_shell: Some(required_command_shell(
                    &genotyping_plan,
                    &genotyping_plan_path,
                )?),
            },
        );
    }

    Ok(proofs)
}

fn collect_json_fields(values: &[serde_json::Value]) -> Vec<String> {
    fn visit(value: &serde_json::Value, fields: &mut BTreeSet<String>) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    fields.insert(key.clone());
                    visit(value, fields);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    visit(item, fields);
                }
            }
            serde_json::Value::Null
            | serde_json::Value::Bool(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::String(_) => {}
        }
    }

    let mut fields = BTreeSet::new();
    for value in values {
        visit(value, &mut fields);
    }
    fields.into_iter().collect()
}

fn collect_plan_output_ids(payload: &serde_json::Value, path: &Path) -> Result<Vec<String>> {
    let outputs = payload
        .get("io")
        .and_then(|value| value.get("outputs"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing array `io.outputs`", path.display()))?;
    let mut output_ids = outputs
        .iter()
        .map(|artifact| {
            artifact
                .get("name")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| anyhow!("{} has output without string `name`", path.display()))
        })
        .collect::<Result<Vec<_>>>()?;
    output_ids.sort();
    Ok(output_ids)
}

fn collect_plan_artifact_paths(
    repo_root: &Path,
    payload: &serde_json::Value,
    plan_path: &Path,
) -> Result<Vec<String>> {
    let outputs = payload
        .get("io")
        .and_then(|value| value.get("outputs"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing array `io.outputs`", plan_path.display()))?;

    let mut artifact_paths = vec![path_relative_to_repo(repo_root, plan_path)];
    for artifact in outputs {
        let path = artifact
            .get("path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow!("{} has output without string `path`", plan_path.display()))?;
        if path.is_empty() {
            return Err(anyhow!("{} has empty output `path`", plan_path.display()));
        }
        artifact_paths.push(path.to_string());
    }
    artifact_paths.sort();
    artifact_paths.dedup();
    Ok(artifact_paths)
}

fn required_command_shell(payload: &serde_json::Value, path: &Path) -> Result<String> {
    let template = payload
        .get("command")
        .and_then(|value| value.get("template"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing array `command.template`", path.display()))?;
    let shell = template.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>().join(" ");
    if shell.trim().is_empty() {
        return Err(anyhow!("{} has empty command template shell", path.display()));
    }
    Ok(shell)
}

fn required_string(payload: &serde_json::Value, key: &str, path: &Path) -> Result<String> {
    payload
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("{} is missing string `{key}`", path.display()))
}

fn required_nested_string(
    payload: &serde_json::Value,
    keys: &[&str],
    path: &Path,
) -> Result<String> {
    let mut current = payload;
    for key in keys {
        current = current.get(key).ok_or_else(|| {
            anyhow!("{} is missing nested key `{}`", path.display(), keys.join("."))
        })?;
    }
    current
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| anyhow!("{} is missing string `{}`", path.display(), keys.join(".")))
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
    if !path.exists() {
        return Err(anyhow!("governed local-proof artifact is missing: {}", path.display()));
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
                "duplicate BAM recalibration/genotyping binding `{}` / `{}`",
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
    let expected = bam_binding_specs()
        .into_iter()
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
    bam_binding_specs()
        .into_iter()
        .find(|spec| spec.stage_id == stage_id && spec.tool_id == tool_id)
}

fn binding_key(stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey { stage_id: stage_id.to_string(), tool_id: tool_id.to_string() }
}

fn expected_stage_ids() -> BTreeSet<&'static str> {
    bam_binding_specs().into_iter().map(|spec| spec.stage_id).collect()
}

fn expected_tool_ids_by_stage() -> BTreeMap<String, Vec<String>> {
    let mut by_stage = BTreeMap::<String, Vec<String>>::new();
    for spec in bam_binding_specs() {
        by_stage.entry(spec.stage_id.to_string()).or_default().push(spec.tool_id.to_string());
    }
    by_stage
}

fn required_output_ids_by_stage() -> BTreeMap<String, Vec<String>> {
    let mut by_stage = BTreeMap::<String, BTreeSet<String>>::new();
    for spec in bam_binding_specs() {
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

fn expected_local_proof_kind_by_stage() -> BTreeMap<String, String> {
    let mut by_stage = BTreeMap::<String, String>::new();
    for spec in bam_binding_specs() {
        by_stage
            .entry(spec.stage_id.to_string())
            .or_insert_with(|| spec.expected_local_proof_kind.to_string());
    }
    by_stage
}

fn required_local_proof_fields_by_stage() -> BTreeMap<String, Vec<String>> {
    let mut by_stage = BTreeMap::<String, BTreeSet<String>>::new();
    for spec in bam_binding_specs() {
        let stage_fields = by_stage.entry(spec.stage_id.to_string()).or_default();
        for field in spec.required_local_proof_fields {
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
        render_bam_recalibration_genotyping_ready,
        BAM_RECALIBRATION_GENOTYPING_READY_SCHEMA_VERSION,
        DEFAULT_BAM_RECALIBRATION_GENOTYPING_READY_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn render_bam_recalibration_genotyping_ready_reports_governed_rows() {
        let report = render_bam_recalibration_genotyping_ready(
            &repo_root(),
            PathBuf::from(DEFAULT_BAM_RECALIBRATION_GENOTYPING_READY_PATH),
        )
        .expect("render BAM recalibration/genotyping readiness");

        #[cfg(feature = "bam_downstream")]
        let expected_row_count = 2;
        #[cfg(not(feature = "bam_downstream"))]
        let expected_row_count = 1;

        assert_eq!(report.schema_version, BAM_RECALIBRATION_GENOTYPING_READY_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_BAM_RECALIBRATION_GENOTYPING_READY_PATH);
        assert_eq!(report.active_row_count, expected_row_count);
        assert_eq!(report.complete_row_count, expected_row_count);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 8);
        assert_eq!(report.stage_count, expected_row_count);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);

        let recalibration_row = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.recalibration" && row.tool_id == "gatk")
            .expect("bam.recalibration gatk row");
        assert_eq!(recalibration_row.expected_normalized_metrics_output_id, "recal_report");
        assert_eq!(recalibration_row.expected_local_proof_kind, "local_smoke");
        assert_eq!(recalibration_row.local_proof_tool_id, "gatk");
        assert!(recalibration_row
            .required_local_proof_fields
            .iter()
            .any(|field| field == "known_sites"));
        assert!(recalibration_row
            .local_proof_declared_output_ids
            .iter()
            .any(|output| output == "recalibration_summary"));

        #[cfg(feature = "bam_downstream")]
        {
            let genotyping_row = report
                .rows
                .iter()
                .find(|row| row.stage_id == "bam.genotyping" && row.tool_id == "angsd")
                .expect("bam.genotyping angsd row");
            assert_eq!(genotyping_row.expected_normalized_metrics_output_id, "genotyping_report");
            assert_eq!(genotyping_row.expected_local_proof_kind, "local_ready_plan");
            assert_eq!(genotyping_row.local_proof_tool_id, "angsd");
            assert!(genotyping_row
                .required_local_proof_fields
                .iter()
                .any(|field| field == "min_call_rate"));
            assert!(genotyping_row
                .local_proof_declared_output_ids
                .iter()
                .any(|output| output == "genotyping_bcf"));
        }
    }
}
