use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

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
use super::fastq_command_adapter_coverage::{
    render_fastq_command_adapter_coverage, FastqAdapterCoverageKind, FastqBenchmarkStatus,
    FastqCommandAdapterCoverageRow, FastqReadinessGapKind,
    DEFAULT_FASTQ_COMMAND_ADAPTER_COVERAGE_PATH,
};
use super::fastq_normalized_metrics_schema::render_fastq_normalized_metrics_schema;
use super::fastq_parser_coverage::{
    render_fastq_parser_coverage, FastqParserCoverageKind, FastqParserCoverageRow,
    DEFAULT_FASTQ_PARSER_COVERAGE_PATH,
};
use super::fastq_report_map::{
    render_fastq_report_map, FastqReportMapRow, DEFAULT_FASTQ_REPORT_MAP_PATH,
};
use crate::commands::benchmark::schema_paths::DEFAULT_FASTQ_NORMALIZED_METRICS_SCHEMA_PATH;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FASTQ_TRIM_STAGES_READY_PATH: &str =
    "benchmarks/readiness/fastq/trim-stages-ready.json";
const FASTQ_TRIM_STAGES_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_trim_stages_ready.v1";
const COVERAGE_STATUS_COMPLETE: &str = "complete";
const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
const CHECKED_SURFACE_COUNT: usize = 7;
const REQUIRED_REPORT_SECTION_ID: &str = "read_cleanup";
const REQUIRED_SUMMARY_TABLE_ID: &str = "cleanup_retention";

const TRIM_READS_TOOL_IDS: [&str; 13] = [
    "adapterremoval",
    "alientrimmer",
    "atropos",
    "bbduk",
    "cutadapt",
    "fastp",
    "fastx_clipper",
    "leehom",
    "prinseq",
    "seqkit",
    "skewer",
    "trim_galore",
    "trimmomatic",
];
const TRIM_TERMINAL_DAMAGE_TOOL_IDS: [&str; 3] = ["adapterremoval", "cutadapt", "seqkit"];
const TRIM_POLYG_TAILS_TOOL_IDS: [&str; 2] = ["bbduk", "fastp"];

const TRIM_READS_REQUIRED_METRIC_FIELDS: [&str; 3] =
    ["reads_retained", "reads_dropped", "bases_removed"];
const TRIM_TERMINAL_DAMAGE_REQUIRED_METRIC_FIELDS: [&str; 4] =
    ["reads_retained", "bases_removed", "trim_5p_bases", "trim_3p_bases"];
const TRIM_POLYG_TAILS_REQUIRED_METRIC_FIELDS: [&str; 5] = [
    "reads_retained",
    "reads_dropped",
    "bases_removed",
    "trimmed_tail_count",
    "bases_trimmed_polyg",
];

#[derive(Debug, Clone, Copy)]
struct TrimStageSpec {
    stage_id: &'static str,
    expected_tool_ids: &'static [&'static str],
    required_metric_fields: &'static [&'static str],
}

const TRIM_STAGE_SPECS: [TrimStageSpec; 3] = [
    TrimStageSpec {
        stage_id: "fastq.trim_reads",
        expected_tool_ids: &TRIM_READS_TOOL_IDS,
        required_metric_fields: &TRIM_READS_REQUIRED_METRIC_FIELDS,
    },
    TrimStageSpec {
        stage_id: "fastq.trim_terminal_damage",
        expected_tool_ids: &TRIM_TERMINAL_DAMAGE_TOOL_IDS,
        required_metric_fields: &TRIM_TERMINAL_DAMAGE_REQUIRED_METRIC_FIELDS,
    },
    TrimStageSpec {
        stage_id: "fastq.trim_polyg_tails",
        expected_tool_ids: &TRIM_POLYG_TAILS_TOOL_IDS,
        required_metric_fields: &TRIM_POLYG_TAILS_REQUIRED_METRIC_FIELDS,
    },
];

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqTrimStagesReadyRow {
    pub(crate) result_id: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) sample_scope: String,
    pub(crate) benchmark_status: String,
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
    pub(crate) schema_extension_id: String,
    pub(crate) required_metric_fields: Vec<String>,
    pub(crate) schema_required_fields: Vec<String>,
    pub(crate) active_scope_proof_path: String,
    pub(crate) command_proof_path: String,
    pub(crate) output_contract_proof_path: String,
    pub(crate) parser_proof_path: String,
    pub(crate) expected_result_proof_path: String,
    pub(crate) report_map_proof_path: String,
    pub(crate) schema_proof_path: String,
    pub(crate) active_scope_ready: bool,
    pub(crate) command_ready: bool,
    pub(crate) output_ready: bool,
    pub(crate) parser_ready: bool,
    pub(crate) expected_result_ready: bool,
    pub(crate) report_ready: bool,
    pub(crate) schema_ready: bool,
    pub(crate) coverage_status: String,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqTrimStagesReadyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) active_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) incomplete_row_count: usize,
    pub(crate) checked_surface_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) expected_tool_ids_by_stage: BTreeMap<String, Vec<String>>,
    pub(crate) required_metric_fields_by_stage: BTreeMap<String, Vec<String>>,
    pub(crate) coverage_status_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<FastqTrimStagesReadyRow>,
    pub(crate) violations: Vec<FastqTrimStagesReadyRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone)]
struct TrimStageSchemaContract {
    extension_id: String,
    required_fields: Vec<String>,
}

pub(crate) fn run_render_fastq_trim_stages_ready(
    args: &parse::BenchReadinessRenderFastqTrimStagesReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_trim_stages_ready(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_TRIM_STAGES_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_fastq_trim_stages_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqTrimStagesReadyReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_fastq_trim_stages_ready_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "fastq trim stages must keep active scope, command, output, parser, expected-result, report, and schema proof"
        ));
    }
    Ok(report)
}

fn build_fastq_trim_stages_ready_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<FastqTrimStagesReadyReport> {
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
    let expected_results = render_expected_benchmark_results(
        repo_root,
        PathBuf::from(DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH),
    )?;
    let report_map =
        render_fastq_report_map(repo_root, PathBuf::from(DEFAULT_FASTQ_REPORT_MAP_PATH))?;
    let command_coverage = render_fastq_command_adapter_coverage(
        repo_root,
        PathBuf::from(DEFAULT_FASTQ_COMMAND_ADAPTER_COVERAGE_PATH),
    )?;
    let schema_report = render_fastq_normalized_metrics_schema(
        repo_root,
        PathBuf::from(DEFAULT_FASTQ_NORMALIZED_METRICS_SCHEMA_PATH),
    )?;
    let schema_contracts = collect_trim_stage_schema_contracts()?;

    let active_rows = active_matrix
        .rows
        .into_iter()
        .filter(|row| trim_stage_spec(&row.stage_id).is_some())
        .collect::<Vec<_>>();
    ensure_trim_stage_tool_sets(
        "FASTQ trim-stage active rows",
        active_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let output_rows = output_contract
        .rows
        .into_iter()
        .filter(|row| trim_stage_binding_admitted(&row.stage_id, &row.tool_id))
        .collect::<Vec<_>>();
    ensure_trim_stage_tool_sets(
        "FASTQ trim-stage output-contract rows",
        output_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let parser_rows = parser_coverage
        .rows
        .into_iter()
        .filter(|row| trim_stage_binding_admitted(&row.stage_id, &row.tool_id))
        .collect::<Vec<_>>();
    ensure_trim_stage_tool_sets(
        "FASTQ trim-stage parser rows",
        parser_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let expected_rows = expected_results
        .rows
        .into_iter()
        .filter(|row| {
            row.domain == "fastq" && trim_stage_binding_admitted(&row.stage_id, &row.tool_id)
        })
        .collect::<Vec<_>>();
    ensure_trim_stage_tool_sets(
        "FASTQ trim-stage expected-result rows",
        expected_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let command_rows = command_coverage
        .rows
        .into_iter()
        .filter(|row| trim_stage_binding_admitted(&row.stage_id, &row.tool_id))
        .collect::<Vec<_>>();
    ensure_trim_stage_tool_sets(
        "FASTQ trim-stage rendered-command rows",
        command_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let report_map_rows = report_map
        .rows
        .into_iter()
        .filter(|row| trim_stage_spec(&row.stage_id).is_some())
        .collect::<Vec<_>>();
    ensure_trim_stage_tool_sets(
        "FASTQ trim-stage report-map rows",
        report_map_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;
    ensure_required_stage_rows(
        "FASTQ trim-stage schema rows",
        schema_contracts.keys().map(String::as_str),
    )?;

    let output_by_binding = collect_rows_by_binding(output_rows)?;
    let parser_by_binding = collect_rows_by_binding(parser_rows)?;
    let expected_by_binding = collect_rows_by_binding(expected_rows)?;
    let command_by_binding = collect_rows_by_binding(command_rows)?;
    let report_map_by_binding = report_map_rows
        .into_iter()
        .map(|row| (binding_key(&row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::with_capacity(active_rows.len());
    for active_row in active_rows {
        let key = binding_key(&active_row.stage_id, &active_row.tool_id);
        let spec = trim_stage_spec(&active_row.stage_id).expect("trim stage spec");
        let output_row = output_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing trim-stage output-contract row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        let parser_row = parser_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing trim-stage parser row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        let expected_row = expected_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing trim-stage expected-result row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        let command_row = command_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing trim-stage rendered-command row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        let report_map_row = report_map_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing trim-stage report-map row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        let schema_contract = schema_contracts.get(&active_row.stage_id).ok_or_else(|| {
            anyhow!("missing trim-stage schema contract for `{}`", active_row.stage_id)
        })?;

        rows.push(build_fastq_trim_stages_ready_row(
            active_row,
            output_row,
            parser_row,
            expected_row,
            command_row,
            report_map_row,
            schema_contract,
            spec,
            &schema_report.output_path,
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

    Ok(FastqTrimStagesReadyReport {
        schema_version: FASTQ_TRIM_STAGES_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: rows.len(),
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: CHECKED_SURFACE_COUNT,
        stage_count: TRIM_STAGE_SPECS.len(),
        expected_tool_ids_by_stage: TRIM_STAGE_SPECS
            .iter()
            .map(|spec| {
                (
                    spec.stage_id.to_string(),
                    spec.expected_tool_ids.iter().map(|tool_id| (*tool_id).to_string()).collect(),
                )
            })
            .collect(),
        required_metric_fields_by_stage: TRIM_STAGE_SPECS
            .iter()
            .map(|spec| {
                (
                    spec.stage_id.to_string(),
                    spec.required_metric_fields.iter().map(|field| (*field).to_string()).collect(),
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
fn build_fastq_trim_stages_ready_row(
    active_row: FastqActiveStageToolMatrixRow,
    output_row: &FastqAdapterOutputContractRow,
    parser_row: &FastqParserCoverageRow,
    expected_row: &ExpectedBenchmarkResultRow,
    command_row: &FastqCommandAdapterCoverageRow,
    report_map_row: &FastqReportMapRow,
    schema_contract: &TrimStageSchemaContract,
    spec: TrimStageSpec,
    schema_proof_path: &str,
) -> FastqTrimStagesReadyRow {
    let active_scope_ready = active_row.benchmark_status == "benchmark_ready"
        && has_governed_support(&active_row.support_status)
        && has_runnable_adapter(&active_row.adapter_status)
        && has_fixture_corpus(&active_row.corpus_status);
    let command_ready = command_row.benchmark_status == FastqBenchmarkStatus::BenchmarkReady
        && command_row.adapter_coverage == FastqAdapterCoverageKind::Covered
        && command_row.readiness_gap == FastqReadinessGapKind::None;
    let output_ready = output_row.output_contract_status
        == FastqAdapterOutputContractStatus::Complete
        && output_row.missing_declarations.is_empty()
        && output_row.normalized_metrics_output_id.as_deref() == Some("report_json");
    let parser_ready = parser_row.parser_coverage == FastqParserCoverageKind::Covered;
    let expected_result_ready = expected_row.sample_scope == "sample-set"
        && expected_row.normalized_metrics_output_id.as_deref() == Some("report_json")
        && expected_row.expected_output_artifact_ids.iter().any(|value| value == "report_json")
        && !expected_row.stage_result_manifest_path.is_empty()
        && !expected_row.stdout_path.is_empty()
        && !expected_row.stderr_path.is_empty();
    let report_ready = report_map_row.report_section_id == REQUIRED_REPORT_SECTION_ID
        && report_map_row.summary_table_id == REQUIRED_SUMMARY_TABLE_ID;
    let schema_ready = spec
        .required_metric_fields
        .iter()
        .all(|field| schema_contract.required_fields.iter().any(|candidate| candidate == field));

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

    let coverage_status = if missing_surfaces.is_empty() {
        COVERAGE_STATUS_COMPLETE.to_string()
    } else {
        COVERAGE_STATUS_INCOMPLETE.to_string()
    };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "trim binding `{}` / `{}` keeps active scope, command, output, parser, expected-result, report, and schema proof for `{}`",
            active_row.stage_id, active_row.tool_id, active_row.stage_id
        )
    } else {
        format!(
            "trim binding `{}` / `{}` is missing readiness proof for {}",
            active_row.stage_id,
            active_row.tool_id,
            missing_surfaces.join(", ")
        )
    };

    FastqTrimStagesReadyRow {
        result_id: expected_row.result_row_id.clone(),
        stage_id: active_row.stage_id.clone(),
        tool_id: active_row.tool_id.clone(),
        corpus_id: active_row.corpus_id,
        sample_scope: expected_row.sample_scope.clone(),
        benchmark_status: active_row.benchmark_status,
        support_status: active_row.support_status,
        adapter_status: active_row.adapter_status,
        parser_status: active_row.parser_status,
        corpus_status: active_row.corpus_status,
        report_section_id: report_map_row.report_section_id.clone(),
        summary_table_id: report_map_row.summary_table_id.clone(),
        command_readiness_kind: expected_row.readiness_kind.clone(),
        expected_outputs: expected_row.expected_output_artifact_ids.clone(),
        raw_output_artifact_ids: output_row.raw_output_artifact_ids.clone(),
        normalized_metrics_output_id: output_row.normalized_metrics_output_id.clone(),
        schema_extension_id: schema_contract.extension_id.clone(),
        required_metric_fields: spec
            .required_metric_fields
            .iter()
            .map(|field| (*field).to_string())
            .collect(),
        schema_required_fields: schema_contract.required_fields.clone(),
        active_scope_proof_path: active_row.scope_proof_path,
        command_proof_path: DEFAULT_FASTQ_COMMAND_ADAPTER_COVERAGE_PATH.to_string(),
        output_contract_proof_path: DEFAULT_FASTQ_ADAPTER_OUTPUT_CONTRACT_PATH.to_string(),
        parser_proof_path: DEFAULT_FASTQ_PARSER_COVERAGE_PATH.to_string(),
        expected_result_proof_path: DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH.to_string(),
        report_map_proof_path: DEFAULT_FASTQ_REPORT_MAP_PATH.to_string(),
        schema_proof_path: schema_proof_path.to_string(),
        active_scope_ready,
        command_ready,
        output_ready,
        parser_ready,
        expected_result_ready,
        report_ready,
        schema_ready,
        coverage_status,
        missing_surfaces,
        reason,
    }
}

fn collect_trim_stage_schema_contracts() -> Result<BTreeMap<String, TrimStageSchemaContract>> {
    let schema = bijux_dna_api::v1::api::bench::render_fastq_normalized_metrics_schema();
    let stage_defs = schema
        .get("$defs")
        .and_then(|value| value.get("stages"))
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            anyhow!("FASTQ normalized metrics schema is missing object `$defs.stages`")
        })?;

    let mut contracts = BTreeMap::new();
    for spec in TRIM_STAGE_SPECS {
        let stage_contract = stage_defs
            .get(spec.stage_id)
            .ok_or_else(|| {
                anyhow!("FASTQ normalized metrics schema is missing `{}`", spec.stage_id)
            })?
            .get("allOf")
            .and_then(serde_json::Value::as_array)
            .and_then(|items| items.get(1))
            .ok_or_else(|| {
                anyhow!(
                    "FASTQ normalized metrics stage `{}` is missing stage extension",
                    spec.stage_id
                )
            })?;
        let extension_id = stage_contract
            .get("x-bijux-extension-id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                anyhow!(
                    "FASTQ normalized metrics stage `{}` is missing string `x-bijux-extension-id`",
                    spec.stage_id
                )
            })?;
        let required_fields = stage_contract
            .get("required")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| {
                anyhow!(
                    "FASTQ normalized metrics stage `{}` is missing `required` keys",
                    spec.stage_id
                )
            })?
            .iter()
            .map(|value| {
                value.as_str().map(str::to_string).ok_or_else(|| {
                    anyhow!(
                        "FASTQ normalized metrics stage `{}` has non-string required key",
                        spec.stage_id
                    )
                })
            })
            .collect::<Result<Vec<_>>>()?;
        contracts.insert(
            spec.stage_id.to_string(),
            TrimStageSchemaContract { extension_id: extension_id.to_string(), required_fields },
        );
    }

    Ok(contracts)
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
                "duplicate trim-stage binding `{}` / `{}`",
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

impl BindingRow for FastqAdapterOutputContractRow {
    fn stage_id(&self) -> &str {
        &self.stage_id
    }

    fn tool_id(&self) -> &str {
        &self.tool_id
    }
}

impl BindingRow for FastqParserCoverageRow {
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

impl BindingRow for FastqCommandAdapterCoverageRow {
    fn stage_id(&self) -> &str {
        &self.stage_id
    }

    fn tool_id(&self) -> &str {
        &self.tool_id
    }
}

fn ensure_trim_stage_tool_sets<'a>(
    label: &str,
    pairs: impl Iterator<Item = (&'a str, &'a str)>,
) -> Result<()> {
    let mut observed = BTreeMap::<String, BTreeSet<String>>::new();
    for (stage_id, tool_id) in pairs {
        observed.entry(stage_id.to_string()).or_default().insert(tool_id.to_string());
    }

    for spec in TRIM_STAGE_SPECS {
        let actual = observed.get(spec.stage_id).cloned().unwrap_or_default();
        let expected = spec
            .expected_tool_ids
            .iter()
            .map(|tool_id| (*tool_id).to_string())
            .collect::<BTreeSet<_>>();
        if actual != expected {
            return Err(anyhow!(
                "{} for `{}` must match the governed trim tool set: expected [{}], found [{}]",
                label,
                spec.stage_id,
                expected.iter().cloned().collect::<Vec<_>>().join(", "),
                actual.iter().cloned().collect::<Vec<_>>().join(", ")
            ));
        }
    }
    Ok(())
}

fn ensure_required_stage_rows<'a>(
    label: &str,
    stage_ids: impl Iterator<Item = &'a str>,
) -> Result<()> {
    let observed = stage_ids.map(str::to_string).collect::<BTreeSet<_>>();
    let expected =
        TRIM_STAGE_SPECS.iter().map(|spec| spec.stage_id.to_string()).collect::<BTreeSet<_>>();
    if observed != expected {
        return Err(anyhow!(
            "{} must cover [{}], found [{}]",
            label,
            expected.iter().cloned().collect::<Vec<_>>().join(", "),
            observed.iter().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    Ok(())
}

fn trim_stage_spec(stage_id: &str) -> Option<TrimStageSpec> {
    TRIM_STAGE_SPECS.iter().copied().find(|spec| spec.stage_id == stage_id)
}

fn trim_stage_binding_admitted(stage_id: &str, tool_id: &str) -> bool {
    trim_stage_spec(stage_id).is_some_and(|spec| spec.expected_tool_ids.contains(&tool_id))
}

fn binding_key(stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey { stage_id: stage_id.to_string(), tool_id: tool_id.to_string() }
}

fn has_governed_support(value: &str) -> bool {
    matches!(
        value,
        "governed_execution" | "governed_benchmark_cohort" | "observer_specialized_benchmark"
    )
}

fn has_runnable_adapter(value: &str) -> bool {
    matches!(value, "runnable" | "plannable")
}

fn has_fixture_corpus(value: &str) -> bool {
    value.starts_with("fixture:")
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
        render_fastq_trim_stages_ready, DEFAULT_FASTQ_TRIM_STAGES_READY_PATH,
        FASTQ_TRIM_STAGES_READY_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_fastq_trim_stages_ready_reports_complete_trim_bindings() {
        let root = repo_root();
        let report = render_fastq_trim_stages_ready(
            &root,
            PathBuf::from(DEFAULT_FASTQ_TRIM_STAGES_READY_PATH),
        )
        .expect("render FASTQ trim stages readiness");

        assert_eq!(report.schema_version, FASTQ_TRIM_STAGES_READY_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_FASTQ_TRIM_STAGES_READY_PATH);
        assert_eq!(report.active_row_count, 18);
        assert_eq!(report.complete_row_count, 18);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 7);
        assert_eq!(report.stage_count, 3);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);
        assert_eq!(report.coverage_status_counts.get("complete"), Some(&18));

        let row = report
            .rows
            .iter()
            .find(|row| row.stage_id == "fastq.trim_polyg_tails" && row.tool_id == "fastp")
            .expect("fastp trim_polyg_tails row");
        assert!(row.active_scope_ready);
        assert!(row.command_ready);
        assert!(row.output_ready);
        assert!(row.parser_ready);
        assert!(row.expected_result_ready);
        assert!(row.report_ready);
        assert!(row.schema_ready);
        assert_eq!(row.report_section_id, "read_cleanup");
        assert_eq!(row.summary_table_id, "cleanup_retention");
        assert_eq!(row.normalized_metrics_output_id.as_deref(), Some("report_json"));
        assert!(row.schema_required_fields.iter().any(|field| field == "trimmed_tail_count"));
        assert!(row.schema_required_fields.iter().any(|field| field == "bases_trimmed_polyg"));
    }
}
