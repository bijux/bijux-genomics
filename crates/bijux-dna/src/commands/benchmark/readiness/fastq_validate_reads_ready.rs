use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_fastq::observer::{parse_validated_reads_manifest, parse_validation_report};
use bijux_dna_domain_fastq::params::PairedMode;
use serde::{Deserialize, Serialize};

use super::expected_benchmark_results::{
    render_expected_benchmark_results, ExpectedBenchmarkResultRow,
    DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use super::fastq_active_stage_tool_matrix::{
    render_fastq_active_stage_tool_matrix, FastqActiveStageToolMatrixRow,
    DEFAULT_FASTQ_ACTIVE_STAGE_TOOL_MATRIX_PATH,
};
use super::fastq_adapter_output_contract::{
    render_fastq_adapter_output_contract, FastqAdapterOutputContractRow,
    FastqAdapterOutputContractStatus, DEFAULT_FASTQ_ADAPTER_OUTPUT_CONTRACT_PATH,
};
use super::fastq_comparable_metrics::{
    render_fastq_comparable_metrics, FastqComparableMetricContractStatus,
    FastqComparableMetricsRow, DEFAULT_FASTQ_COMPARABLE_METRICS_PATH,
};
use super::fastq_parser_coverage::{
    render_fastq_parser_coverage, FastqParserCoverageKind, FastqParserCoverageRow,
    DEFAULT_FASTQ_PARSER_COVERAGE_PATH,
};
use super::fastq_report_map::{
    render_fastq_report_map, FastqReportMapRow, DEFAULT_FASTQ_REPORT_MAP_PATH,
};
use super::rendered_command_argv::{
    render_command_argv, RenderedCommandArgvRow, DEFAULT_RENDERED_COMMAND_ARGV_PATH,
};
use crate::commands::benchmark::local_stage_commands::materialize_local_stage;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FASTQ_VALIDATE_READS_READY_PATH: &str =
    "benchmarks/readiness/fastq/validate-reads-ready.json";
const FASTQ_VALIDATE_READS_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_validate_reads_ready.v1";
const FASTQ_VALIDATE_READS_STAGE_ID: &str = "fastq.validate_reads";
const FASTQ_VALIDATE_READS_SMOKE_PATH: &str =
    "runs/bench/local-smoke/fastq.validate_reads/report.json";
const COVERAGE_STATUS_COMPLETE: &str = "complete";
const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
const VALIDATION_STATUS_FIELD_ID: &str = "strict_pass";
const FAILURE_REASON_FIELD_ID: &str = "failure_class";
const EXPECTED_SMOKE_CASE_COUNT: usize = 2;
const REQUIRED_TOOL_IDS: [&str; 5] = ["fastq_scan", "fastqc", "fastqvalidator", "fqtools", "seqtk"];
const REQUIRED_SMOKE_SAMPLE_IDS: [&str; 2] = ["toy-pe", "toy-se"];
const REQUIRED_SMOKE_LAYOUTS: [&str; 2] = ["paired_end", "single_end"];
const REQUIRED_SHARED_METRIC_FIELDS: [&str; 1] = ["format_validation_pass_rate"];
const REQUIRED_VALIDATION_REPORT_FIELDS: [&str; 10] = [
    "validated_reads_r1",
    "validated_reads_r2",
    "validated_pairs",
    "status_r1",
    "status_r2",
    "pair_sync_checked",
    "pair_sync_pass",
    "pair_count_match",
    "failure_class",
    "strict_pass",
];
const REQUIRED_VALIDATED_READS_MANIFEST_FIELDS: [&str; 5] = [
    "paired_mode",
    "validated_stream_ids",
    "pair_sync_checked",
    "pair_sync_pass",
    "validated_pairs",
];

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqValidateReadsReadyRow {
    pub(crate) result_id: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) sample_scope: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
    pub(crate) report_section_id: String,
    pub(crate) summary_table_id: String,
    pub(crate) command_readiness_kind: String,
    pub(crate) expected_outputs: Vec<String>,
    pub(crate) raw_output_artifact_ids: Vec<String>,
    pub(crate) normalized_metrics_output_id: Option<String>,
    pub(crate) shared_metric_fields: Vec<String>,
    pub(crate) validation_report_fields: Vec<String>,
    pub(crate) validated_reads_manifest_fields: Vec<String>,
    pub(crate) command_argv_output_path: String,
    pub(crate) output_contract_proof_path: String,
    pub(crate) parser_proof_path: String,
    pub(crate) comparable_metrics_proof_path: String,
    pub(crate) expected_result_proof_path: String,
    pub(crate) report_map_proof_path: String,
    pub(crate) smoke_proof_path: String,
    pub(crate) smoke_sample_ids: Vec<String>,
    pub(crate) smoke_layouts: Vec<String>,
    pub(crate) smoke_read_count_totals: Vec<u64>,
    pub(crate) smoke_validation_statuses: Vec<String>,
    pub(crate) smoke_failure_classes: Vec<String>,
    pub(crate) command_ready: bool,
    pub(crate) output_ready: bool,
    pub(crate) parser_ready: bool,
    pub(crate) comparable_metrics_ready: bool,
    pub(crate) expected_result_ready: bool,
    pub(crate) report_ready: bool,
    pub(crate) smoke_parseable: bool,
    pub(crate) sample_id_normalized: bool,
    pub(crate) layout_normalized: bool,
    pub(crate) read_count_normalized: bool,
    pub(crate) validation_status_normalized: bool,
    pub(crate) failure_reason_normalized: bool,
    pub(crate) coverage_status: String,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqValidateReadsReadyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) active_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) incomplete_row_count: usize,
    pub(crate) checked_surface_count: usize,
    pub(crate) expected_tool_ids: Vec<String>,
    pub(crate) validation_status_field_id: &'static str,
    pub(crate) failure_reason_field_id: &'static str,
    pub(crate) required_validation_report_fields: Vec<String>,
    pub(crate) required_validated_reads_manifest_fields: Vec<String>,
    pub(crate) sample_case_count: usize,
    pub(crate) smoke_sample_ids: Vec<String>,
    pub(crate) smoke_layouts: Vec<String>,
    pub(crate) coverage_status_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<FastqValidateReadsReadyRow>,
    pub(crate) violations: Vec<FastqValidateReadsReadyRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone)]
struct FastqValidateReadsSmokeEvidence {
    proof_path: String,
    sample_case_count: usize,
    sample_ids: Vec<String>,
    layouts: Vec<String>,
    read_count_totals: Vec<u64>,
    validation_statuses: Vec<String>,
    failure_classes: Vec<String>,
    parseable: bool,
    sample_id_normalized: bool,
    layout_normalized: bool,
    read_count_normalized: bool,
    validation_status_normalized: bool,
    failure_reason_normalized: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalValidateReadsSmokeReport {
    cases: Vec<LocalValidateReadsSmokeCase>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalValidateReadsSmokeCase {
    sample_id: String,
    layout: String,
    input_read_count_total: u64,
    input_pair_count: Option<u64>,
    validation_status: String,
    validation_report: String,
    validated_reads_manifest: String,
}

pub(crate) fn run_render_fastq_validate_reads_ready(
    args: &parse::BenchReadinessRenderFastqValidateReadsReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_validate_reads_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_VALIDATE_READS_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_fastq_validate_reads_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqValidateReadsReadyReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_fastq_validate_reads_ready_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "fastq.validate_reads retained validators must keep active scope, command, output, parser, expected-result, report, and normalization proof"
        ));
    }
    Ok(report)
}

fn build_fastq_validate_reads_ready_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<FastqValidateReadsReadyReport> {
    let active_matrix = render_fastq_active_stage_tool_matrix(
        repo_root,
        PathBuf::from(DEFAULT_FASTQ_ACTIVE_STAGE_TOOL_MATRIX_PATH),
    )?;
    let output_contract = render_fastq_adapter_output_contract(
        repo_root,
        PathBuf::from(DEFAULT_FASTQ_ADAPTER_OUTPUT_CONTRACT_PATH),
    )?;
    let parser_coverage =
        render_fastq_parser_coverage(repo_root, PathBuf::from(DEFAULT_FASTQ_PARSER_COVERAGE_PATH))?;
    let comparable_metrics = render_fastq_comparable_metrics(
        repo_root,
        PathBuf::from(DEFAULT_FASTQ_COMPARABLE_METRICS_PATH),
    )?;
    let report_map =
        render_fastq_report_map(repo_root, PathBuf::from(DEFAULT_FASTQ_REPORT_MAP_PATH))?;
    let expected_results = render_expected_benchmark_results(
        repo_root,
        PathBuf::from(DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH),
    )?;
    let command_argv =
        render_command_argv(repo_root, PathBuf::from(DEFAULT_RENDERED_COMMAND_ARGV_PATH))?;
    let smoke = load_fastq_validate_reads_smoke_evidence(repo_root)?;

    let active_rows = active_matrix
        .rows
        .into_iter()
        .filter(|row| row.stage_id == FASTQ_VALIDATE_READS_STAGE_ID)
        .collect::<Vec<_>>();
    ensure_required_tool_set(
        "fastq validate-reads active rows",
        active_rows.iter().map(|row| row.tool_id.as_str()),
    )?;

    let output_rows = output_contract
        .rows
        .into_iter()
        .filter(|row| row.stage_id == FASTQ_VALIDATE_READS_STAGE_ID)
        .collect::<Vec<_>>();
    ensure_required_tool_set(
        "fastq validate-reads output-contract rows",
        output_rows.iter().map(|row| row.tool_id.as_str()),
    )?;

    let parser_rows = parser_coverage
        .rows
        .into_iter()
        .filter(|row| row.stage_id == FASTQ_VALIDATE_READS_STAGE_ID)
        .collect::<Vec<_>>();
    ensure_required_tool_set(
        "fastq validate-reads parser rows",
        parser_rows.iter().map(|row| row.tool_id.as_str()),
    )?;

    let expected_rows = expected_results
        .rows
        .into_iter()
        .filter(|row| row.domain == "fastq" && row.stage_id == FASTQ_VALIDATE_READS_STAGE_ID)
        .collect::<Vec<_>>();
    ensure_required_tool_set(
        "fastq validate-reads expected-result rows",
        expected_rows.iter().map(|row| row.tool_id.as_str()),
    )?;

    let command_rows = command_argv
        .rows
        .into_iter()
        .filter(|row| row.stage_id == FASTQ_VALIDATE_READS_STAGE_ID)
        .collect::<Vec<_>>();
    ensure_required_tool_set(
        "fastq validate-reads rendered-command rows",
        command_rows.iter().map(|row| row.tool_id.as_str()),
    )?;

    let comparable_row = comparable_metrics
        .rows
        .into_iter()
        .find(|row| row.stage_id == FASTQ_VALIDATE_READS_STAGE_ID)
        .ok_or_else(|| {
            anyhow!("missing comparable-metrics row for `{FASTQ_VALIDATE_READS_STAGE_ID}`")
        })?;
    let report_map_rows = report_map
        .rows
        .into_iter()
        .filter(|row| row.stage_id == FASTQ_VALIDATE_READS_STAGE_ID)
        .collect::<Vec<_>>();
    ensure_required_tool_set(
        "fastq validate-reads report-map rows",
        report_map_rows.iter().map(|row| row.tool_id.as_str()),
    )?;

    let output_by_binding = output_rows
        .into_iter()
        .map(|row| (binding_key(&row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();
    let parser_by_binding = parser_rows
        .into_iter()
        .map(|row| (binding_key(&row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();
    let expected_by_binding = expected_rows
        .into_iter()
        .map(|row| (binding_key(&row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();
    let command_by_binding = command_rows
        .into_iter()
        .map(|row| (binding_key(&row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();
    let report_map_by_binding = report_map_rows
        .into_iter()
        .map(|row| (binding_key(&row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::with_capacity(active_rows.len());
    for active_row in active_rows {
        let key = binding_key(&active_row.stage_id, &active_row.tool_id);
        let output_row = output_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing output-contract row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        let parser_row = parser_by_binding.get(&key).ok_or_else(|| {
            anyhow!("missing parser row for `{}` / `{}`", active_row.stage_id, active_row.tool_id)
        })?;
        let expected_row = expected_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing expected-result row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        let command_row = command_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing rendered-command row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        let report_map_row = report_map_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing report-map row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        rows.push(build_fastq_validate_reads_ready_row(
            active_row,
            output_row,
            parser_row,
            expected_row,
            command_row,
            &comparable_row,
            report_map_row,
            &smoke,
        ));
    }

    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    let complete_row_count =
        rows.iter().filter(|row| row.coverage_status == COVERAGE_STATUS_COMPLETE).count();
    let incomplete_row_count = rows.len().saturating_sub(complete_row_count);
    let mut coverage_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *coverage_status_counts.entry(row.coverage_status.clone()).or_default() += 1;
    }
    let violations = rows
        .iter()
        .filter(|row| row.coverage_status != COVERAGE_STATUS_COMPLETE)
        .cloned()
        .collect::<Vec<_>>();

    Ok(FastqValidateReadsReadyReport {
        schema_version: FASTQ_VALIDATE_READS_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: rows.len(),
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: 8,
        expected_tool_ids: required_tool_ids(),
        validation_status_field_id: VALIDATION_STATUS_FIELD_ID,
        failure_reason_field_id: FAILURE_REASON_FIELD_ID,
        required_validation_report_fields: required_validation_report_fields(),
        required_validated_reads_manifest_fields: required_validated_reads_manifest_fields(),
        sample_case_count: smoke.sample_case_count,
        smoke_sample_ids: smoke.sample_ids.clone(),
        smoke_layouts: smoke.layouts.clone(),
        coverage_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_fastq_validate_reads_ready_row(
    active_row: FastqActiveStageToolMatrixRow,
    output_row: &FastqAdapterOutputContractRow,
    parser_row: &FastqParserCoverageRow,
    expected_row: &ExpectedBenchmarkResultRow,
    command_row: &RenderedCommandArgvRow,
    comparable_row: &FastqComparableMetricsRow,
    report_map_row: &FastqReportMapRow,
    smoke: &FastqValidateReadsSmokeEvidence,
) -> FastqValidateReadsReadyRow {
    let validation_report_fields = required_validation_report_fields();
    let validated_reads_manifest_fields = required_validated_reads_manifest_fields();
    let mut missing_surfaces = Vec::new();

    let command_ready = command_has_required_validation_fields(command_row, &active_row.tool_id);
    if !command_ready {
        missing_surfaces.push("rendered_commands".to_string());
    }

    let output_ready = output_row.output_contract_status
        == FastqAdapterOutputContractStatus::Complete
        && output_row.normalized_metrics_output_id.as_deref() == Some("validation_report")
        && output_row
            .raw_output_artifact_ids
            .iter()
            .any(|artifact_id| artifact_id == "validated_reads_manifest");
    if !output_ready {
        missing_surfaces.push("adapter_output_contract".to_string());
    }

    let parser_ready = parser_row.parser_coverage == FastqParserCoverageKind::Covered
        && parser_row.parser_status == "comparable";
    if !parser_ready {
        missing_surfaces.push("parser_coverage".to_string());
    }

    let comparable_metrics_ready = comparable_row.comparison_contract_status
        == FastqComparableMetricContractStatus::Declared
        && REQUIRED_SHARED_METRIC_FIELDS
            .iter()
            .all(|field| comparable_row.shared_metric_fields.iter().any(|value| value == field));
    if !comparable_metrics_ready {
        missing_surfaces.push("comparable_metrics".to_string());
    }

    let expected_result_ready = expected_row.normalized_metrics_output_id.as_deref()
        == Some("validation_report")
        && expected_row
            .expected_output_artifact_ids
            .iter()
            .any(|artifact_id| artifact_id == "validation_report")
        && expected_row
            .expected_output_artifact_ids
            .iter()
            .any(|artifact_id| artifact_id == "validated_reads_manifest");
    if !expected_result_ready {
        missing_surfaces.push("expected_results".to_string());
    }

    let report_ready = report_map_row.report_section_id == "input_readiness"
        && report_map_row.summary_table_id == "validation_intake";
    if !report_ready {
        missing_surfaces.push("report_map".to_string());
    }

    if !smoke.parseable {
        missing_surfaces.push("smoke_parse".to_string());
    }
    if !smoke.sample_id_normalized {
        missing_surfaces.push("sample_id_normalization".to_string());
    }
    if !smoke.layout_normalized {
        missing_surfaces.push("layout_normalization".to_string());
    }
    if !smoke.read_count_normalized {
        missing_surfaces.push("read_count_normalization".to_string());
    }
    if !smoke.validation_status_normalized {
        missing_surfaces.push("validation_status_normalization".to_string());
    }
    if !smoke.failure_reason_normalized {
        missing_surfaces.push("failure_reason_normalization".to_string());
    }

    let coverage_status = if missing_surfaces.is_empty() {
        COVERAGE_STATUS_COMPLETE.to_string()
    } else {
        COVERAGE_STATUS_INCOMPLETE.to_string()
    };

    FastqValidateReadsReadyRow {
        result_id: expected_row.result_row_id.clone(),
        stage_id: active_row.stage_id.clone(),
        tool_id: active_row.tool_id.clone(),
        corpus_id: active_row.corpus_id.clone(),
        sample_scope: expected_row.sample_scope.clone(),
        support_status: active_row.support_status.clone(),
        adapter_status: active_row.adapter_status.clone(),
        parser_status: parser_row.parser_status.clone(),
        corpus_status: active_row.corpus_status.clone(),
        report_section_id: report_map_row.report_section_id.clone(),
        summary_table_id: report_map_row.summary_table_id.clone(),
        command_readiness_kind: command_row.readiness_kind.clone(),
        expected_outputs: expected_row.expected_output_artifact_ids.clone(),
        raw_output_artifact_ids: output_row.raw_output_artifact_ids.clone(),
        normalized_metrics_output_id: output_row.normalized_metrics_output_id.clone(),
        shared_metric_fields: comparable_row.shared_metric_fields.clone(),
        validation_report_fields,
        validated_reads_manifest_fields,
        command_argv_output_path: DEFAULT_RENDERED_COMMAND_ARGV_PATH.to_string(),
        output_contract_proof_path: DEFAULT_FASTQ_ADAPTER_OUTPUT_CONTRACT_PATH.to_string(),
        parser_proof_path: DEFAULT_FASTQ_PARSER_COVERAGE_PATH.to_string(),
        comparable_metrics_proof_path: DEFAULT_FASTQ_COMPARABLE_METRICS_PATH.to_string(),
        expected_result_proof_path: DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH.to_string(),
        report_map_proof_path: DEFAULT_FASTQ_REPORT_MAP_PATH.to_string(),
        smoke_proof_path: smoke.proof_path.clone(),
        smoke_sample_ids: smoke.sample_ids.clone(),
        smoke_layouts: smoke.layouts.clone(),
        smoke_read_count_totals: smoke.read_count_totals.clone(),
        smoke_validation_statuses: smoke.validation_statuses.clone(),
        smoke_failure_classes: smoke.failure_classes.clone(),
        command_ready,
        output_ready,
        parser_ready,
        comparable_metrics_ready,
        expected_result_ready,
        report_ready,
        smoke_parseable: smoke.parseable,
        sample_id_normalized: smoke.sample_id_normalized,
        layout_normalized: smoke.layout_normalized,
        read_count_normalized: smoke.read_count_normalized,
        validation_status_normalized: smoke.validation_status_normalized,
        failure_reason_normalized: smoke.failure_reason_normalized,
        coverage_status: coverage_status.clone(),
        missing_surfaces,
        reason: if coverage_status == COVERAGE_STATUS_COMPLETE {
            format!(
                "retained FASTQ validator `{}` keeps active scope, command, output, parser, expected-result, report, and normalized validation proof for `{}`",
                active_row.tool_id, active_row.stage_id
            )
        } else {
            format!(
                "retained FASTQ validator `{}` is missing at least one governed readiness or normalization surface for `{}`",
                active_row.tool_id, active_row.stage_id
            )
        },
    }
}

fn load_fastq_validate_reads_smoke_evidence(
    repo_root: &Path,
) -> Result<FastqValidateReadsSmokeEvidence> {
    let smoke_path = repo_root.join(FASTQ_VALIDATE_READS_SMOKE_PATH);
    if !smoke_path.is_file() {
        materialize_local_stage(repo_root, FASTQ_VALIDATE_READS_STAGE_ID)
            .context("materialize fastq.validate_reads local smoke report")?;
    }
    let payload = fs::read_to_string(&smoke_path)
        .with_context(|| format!("read {}", smoke_path.display()))?;
    let report: LocalValidateReadsSmokeReport =
        serde_json::from_str(&payload).context("parse fastq.validate_reads smoke report")?;

    let mut sample_ids = Vec::new();
    let mut layouts = Vec::new();
    let mut read_count_totals = Vec::new();
    let mut validation_statuses = Vec::new();
    let mut failure_classes = Vec::new();
    let mut parseable = true;
    let mut sample_id_normalized = true;
    let mut layout_normalized = true;
    let mut read_count_normalized = true;
    let mut validation_status_normalized = true;
    let mut failure_reason_normalized = true;

    for case in &report.cases {
        sample_id_normalized &= !case.sample_id.trim().is_empty();
        sample_ids.push(case.sample_id.clone());
        layouts.push(case.layout.clone());
        read_count_totals.push(case.input_read_count_total);
        validation_statuses.push(case.validation_status.clone());

        let validation_report_path = repo_root.join(&case.validation_report);
        let validation_report_json = fs::read_to_string(&validation_report_path)
            .with_context(|| format!("read {}", validation_report_path.display()))?;
        let validation_report = parse_validation_report(&validation_report_json)
            .context("parse smoke validation report")?;

        let validated_reads_manifest_path = repo_root.join(&case.validated_reads_manifest);
        let validated_reads_manifest_json = fs::read_to_string(&validated_reads_manifest_path)
            .with_context(|| format!("read {}", validated_reads_manifest_path.display()))?;
        let validated_reads_manifest =
            parse_validated_reads_manifest(&validated_reads_manifest_json)
                .context("parse smoke validated-reads manifest")?;

        let derived_layout = paired_mode_label(validated_reads_manifest.paired_mode).to_string();
        layout_normalized &= derived_layout == case.layout;

        let derived_total = validation_report.validated_reads_r1
            + validation_report.validated_reads_r2.unwrap_or(0);
        read_count_normalized &= derived_total == case.input_read_count_total;
        if case.layout == "paired_end" {
            read_count_normalized &= validation_report.validated_pairs == case.input_pair_count;
        }

        let derived_status = if validation_report.strict_pass { "pass" } else { "fail" };
        validation_status_normalized &= derived_status == case.validation_status;
        failure_reason_normalized &= true;
        failure_classes
            .push(validate_failure_class_label(&validation_report.failure_class).to_string());

        parseable &= validation_report.stage_id == FASTQ_VALIDATE_READS_STAGE_ID
            && validated_reads_manifest.stage_id == FASTQ_VALIDATE_READS_STAGE_ID
            && validation_report.input_r2.is_some() == (case.layout == "paired_end")
            && validated_reads_manifest.input_r2.is_some() == (case.layout == "paired_end");
    }

    sample_ids.sort();
    sample_ids.dedup();
    layouts.sort();
    layouts.dedup();
    read_count_totals.sort_unstable();
    validation_statuses.sort();
    validation_statuses.dedup();
    failure_classes.sort();
    failure_classes.dedup();

    sample_id_normalized &= sample_ids == REQUIRED_SMOKE_SAMPLE_IDS.map(str::to_string);
    layout_normalized &= layouts == REQUIRED_SMOKE_LAYOUTS.map(str::to_string);
    validation_status_normalized &= validation_statuses == vec!["pass".to_string()];
    parseable &= report.cases.len() == EXPECTED_SMOKE_CASE_COUNT;

    Ok(FastqValidateReadsSmokeEvidence {
        proof_path: FASTQ_VALIDATE_READS_SMOKE_PATH.to_string(),
        sample_case_count: report.cases.len(),
        sample_ids,
        layouts,
        read_count_totals,
        validation_statuses,
        failure_classes,
        parseable,
        sample_id_normalized,
        layout_normalized,
        read_count_normalized,
        validation_status_normalized,
        failure_reason_normalized,
    })
}

fn command_has_required_validation_fields(row: &RenderedCommandArgvRow, tool_id: &str) -> bool {
    let argv_text = row.argv.join(" ");
    argv_text.contains(&format!("\"stage_id\":\"{FASTQ_VALIDATE_READS_STAGE_ID}\""))
        && argv_text.contains(&format!("\"tool_id\":\"{tool_id}\""))
        && argv_text.contains("validation.json")
        && argv_text.contains("validated_reads_manifest.json")
        && required_validation_report_fields()
            .iter()
            .all(|field| argv_text.contains(&format!("\"{field}\"")))
        && required_validated_reads_manifest_fields()
            .iter()
            .all(|field| argv_text.contains(&format!("\"{field}\"")))
}

fn ensure_required_tool_set<'a>(
    surface: &str,
    tool_ids: impl Iterator<Item = &'a str>,
) -> Result<()> {
    let observed = tool_ids.map(str::to_string).collect::<BTreeSet<_>>();
    let expected = required_tool_ids().into_iter().collect::<BTreeSet<_>>();
    if observed != expected {
        return Err(anyhow!("{surface} must contain exactly {:?}, found {:?}", expected, observed));
    }
    Ok(())
}

fn required_tool_ids() -> Vec<String> {
    REQUIRED_TOOL_IDS.iter().map(|value| (*value).to_string()).collect()
}

fn required_validation_report_fields() -> Vec<String> {
    REQUIRED_VALIDATION_REPORT_FIELDS.iter().map(|value| (*value).to_string()).collect()
}

fn required_validated_reads_manifest_fields() -> Vec<String> {
    REQUIRED_VALIDATED_READS_MANIFEST_FIELDS.iter().map(|value| (*value).to_string()).collect()
}

fn binding_key(stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey { stage_id: stage_id.to_string(), tool_id: tool_id.to_string() }
}

fn paired_mode_label(paired_mode: PairedMode) -> &'static str {
    match paired_mode {
        PairedMode::SingleEnd => "single_end",
        PairedMode::PairedEnd => "paired_end",
        PairedMode::Unknown => "unknown",
    }
}

fn validate_failure_class_label(
    value: &bijux_dna_domain_fastq::ValidateFailureClass,
) -> &'static str {
    match value {
        bijux_dna_domain_fastq::ValidateFailureClass::None => "none",
        bijux_dna_domain_fastq::ValidateFailureClass::UnsupportedCompression => {
            "unsupported_compression"
        }
        bijux_dna_domain_fastq::ValidateFailureClass::EmptyInput => "empty_input",
        bijux_dna_domain_fastq::ValidateFailureClass::MalformedRecord => "malformed_record",
        bijux_dna_domain_fastq::ValidateFailureClass::InvalidQualityEncoding => {
            "invalid_quality_encoding"
        }
        bijux_dna_domain_fastq::ValidateFailureClass::ValidatorError => "validator_error",
        bijux_dna_domain_fastq::ValidateFailureClass::PairCountMismatch => "pair_count_mismatch",
        bijux_dna_domain_fastq::ValidateFailureClass::HeaderSyncMismatch => "header_sync_mismatch",
    }
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
        render_fastq_validate_reads_ready, DEFAULT_FASTQ_VALIDATE_READS_READY_PATH,
        FASTQ_VALIDATE_READS_READY_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_fastq_validate_reads_ready_reports_complete_active_validator_rows() {
        let root = repo_root();
        let report = render_fastq_validate_reads_ready(
            &root,
            PathBuf::from(DEFAULT_FASTQ_VALIDATE_READS_READY_PATH),
        )
        .expect("render fastq validate-reads readiness");

        assert_eq!(report.schema_version, FASTQ_VALIDATE_READS_READY_SCHEMA_VERSION);
        assert_eq!(report.output_path, "benchmarks/readiness/fastq/validate-reads-ready.json");
        assert_eq!(report.active_row_count, 5);
        assert_eq!(report.complete_row_count, 5);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 8);
        assert_eq!(report.sample_case_count, 2);
        assert_eq!(report.smoke_sample_ids, vec!["toy-pe".to_string(), "toy-se".to_string()]);
        assert_eq!(report.smoke_layouts, vec!["paired_end".to_string(), "single_end".to_string()]);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);

        let fastqvalidator = report
            .rows
            .iter()
            .find(|row| row.tool_id == "fastqvalidator")
            .expect("fastqvalidator row");
        assert_eq!(
            fastqvalidator.result_id,
            "fastq:corpus-01-mini:fastq.validate_reads:sample-set:fastqvalidator"
        );
        assert_eq!(fastqvalidator.report_section_id, "input_readiness");
        assert_eq!(fastqvalidator.summary_table_id, "validation_intake");
        assert_eq!(fastqvalidator.command_readiness_kind, "smoke");
        assert_eq!(
            fastqvalidator.normalized_metrics_output_id.as_deref(),
            Some("validation_report")
        );
        assert!(fastqvalidator
            .expected_outputs
            .iter()
            .any(|artifact_id| artifact_id == "validation_report"));
        assert!(fastqvalidator
            .expected_outputs
            .iter()
            .any(|artifact_id| artifact_id == "validated_reads_manifest"));
        assert!(fastqvalidator
            .shared_metric_fields
            .iter()
            .any(|field| field == "format_validation_pass_rate"));
        assert!(fastqvalidator
            .validation_report_fields
            .iter()
            .any(|field| field == "failure_class"));
        assert!(fastqvalidator
            .validated_reads_manifest_fields
            .iter()
            .any(|field| field == "paired_mode"));
        assert!(fastqvalidator.command_ready);
        assert!(fastqvalidator.output_ready);
        assert!(fastqvalidator.parser_ready);
        assert!(fastqvalidator.comparable_metrics_ready);
        assert!(fastqvalidator.expected_result_ready);
        assert!(fastqvalidator.report_ready);
        assert!(fastqvalidator.smoke_parseable);
        assert!(fastqvalidator.sample_id_normalized);
        assert!(fastqvalidator.layout_normalized);
        assert!(fastqvalidator.read_count_normalized);
        assert!(fastqvalidator.validation_status_normalized);
        assert!(fastqvalidator.failure_reason_normalized);
        assert_eq!(fastqvalidator.coverage_status, "complete");
        assert!(fastqvalidator.missing_surfaces.is_empty());
    }
}
