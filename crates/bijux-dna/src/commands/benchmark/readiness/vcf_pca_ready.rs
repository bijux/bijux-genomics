use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_stage_tool_matrix::{
    collect_all_domain_active_stage_tool_matrix_rows, AllDomainActiveStageToolMatrixRow,
};
use super::benchmark_command_rows::render_shell_command;
use super::vcf_active_stage_tool_matrix::{
    collect_vcf_active_stage_tool_matrix_rows, VcfActiveStageToolMatrixRow,
};
use super::vcf_eigensoft_adapter::{
    collect_vcf_eigensoft_adapter_rows_for_tool, VcfEigensoftAdapterArtifact,
    VcfEigensoftAdapterRow, DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH,
};
use super::vcf_expected_benchmark_results::DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH;
use super::vcf_parser_fixture_coverage::{
    collect_vcf_parser_fixture_coverage_rows, VcfParserFixtureCoverageRow,
    VcfParserFixtureCoverageStatus, DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH,
};
use super::vcf_plink_family_adapter::{
    collect_vcf_plink_family_adapter_rows_for_tool, VcfPlinkFamilyAdapterArtifact,
    VcfPlinkFamilyAdapterRow, DEFAULT_VCF_PLINK2_ADAPTER_PATH,
};
use super::vcf_rendered_command_rows::VcfRenderedCommandRow;
use super::vcf_rendered_commands::VcfRenderedCommandsReport;
use super::vcf_report_map::DEFAULT_VCF_REPORT_MAP_PATH;
use crate::commands::benchmark::local_slurm_run_paths::LOCAL_SLURM_DRY_RUN_RUN_ID;
use crate::commands::benchmark::local_vcf_pca_smoke::{
    run_local_vcf_pca_smoke, LocalVcfPcaSmokeReport, LocalVcfPcaSmokeRow,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_PCA_READY_PATH: &str = "benchmarks/readiness/vcf/pca-ready.json";
const VCF_PCA_READY_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_pca_ready.v1";
const VCF_PCA_STAGE_ID: &str = "vcf.pca";
const REQUIRED_METRIC_NAMES: [&str; 5] =
    ["sample_count", "variant_count", "excluded_samples", "unexpected_samples", "eigenvalues"];
const REQUIRED_VARIANT_COUNT: u64 = 2;
const REQUIRED_SAMPLE_COUNT: u64 = 4;
const REQUIRED_SAMPLE_IDS: [&str; 4] = ["sample_a", "sample_b", "sample_c", "sample_d"];
const COVERAGE_STATUS_COMPLETE: &str = "complete";
const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
const NO_VALUE: &str = "none";
const VCF_PCA_RENDERED_COMMANDS_PATH: &str =
    "artifacts/bench-readiness/vcf-pca-ready/vcf-pca-rendered-commands.sh";
const VCF_PCA_READY_LOCK_PATH: &str = "artifacts/bench-readiness/vcf-pca-ready/render.lock";

#[derive(Debug, Clone)]
struct PcaAdapterOutputProof {
    output_proof_path: String,
    benchmark_status: String,
    raw_outputs: Vec<String>,
    normalized_metrics_outputs: Vec<String>,
    manifest_output: String,
    index_outputs: Vec<String>,
}

#[derive(Debug, Clone)]
struct VcfPcaExpectedResultProof {
    result_id: String,
    tool_id: String,
    corpus_id: String,
    asset_profile_id: String,
    expected_outputs: Vec<String>,
    expected_metrics: Vec<String>,
    report_section: String,
}

#[derive(Debug, Clone)]
struct VcfPcaReportMapProof {
    tool_id: String,
    section_id: String,
    summary_table: String,
    metric_columns: Vec<String>,
}

#[derive(Debug, Clone)]
struct VcfPcaBinding {
    retained_row: VcfActiveStageToolMatrixRow,
    active_row: Option<AllDomainActiveStageToolMatrixRow>,
    command_row: Option<VcfRenderedCommandRow>,
    parser_row: Option<VcfParserFixtureCoverageRow>,
    expected_row: Option<VcfPcaExpectedResultProof>,
    report_row: Option<VcfPcaReportMapProof>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfPcaReadyRow {
    pub(crate) result_id: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) tool_status: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) schema_id: String,
    pub(crate) retained_scope_state: String,
    pub(crate) retained_scope_detail: String,
    pub(crate) retained_scope_proof_path: String,
    pub(crate) all_domain_active_row_present: bool,
    pub(crate) all_domain_active_row_proof_path: String,
    pub(crate) command_ready: bool,
    pub(crate) command_source: String,
    pub(crate) command_step_count: usize,
    pub(crate) command_step_ids: Vec<String>,
    pub(crate) primary_executables: Vec<String>,
    pub(crate) command_output_path: String,
    pub(crate) command_argv_output_path: String,
    pub(crate) output_ready: bool,
    pub(crate) output_proof_path: String,
    pub(crate) raw_outputs: Vec<String>,
    pub(crate) normalized_metrics_outputs: Vec<String>,
    pub(crate) manifest_output: String,
    pub(crate) index_outputs: Vec<String>,
    pub(crate) parser_ready: bool,
    pub(crate) parser_proof_path: String,
    pub(crate) parser_fixture_parser_id: String,
    pub(crate) parser_fixture_schema_id: String,
    pub(crate) parser_fixture_path: String,
    pub(crate) expected_result_ready: bool,
    pub(crate) expected_result_proof_path: String,
    pub(crate) expected_outputs: Vec<String>,
    pub(crate) expected_metrics: Vec<String>,
    pub(crate) report_section_id: String,
    pub(crate) report_ready: bool,
    pub(crate) report_map_proof_path: String,
    pub(crate) summary_table_id: String,
    pub(crate) report_metric_columns: Vec<String>,
    pub(crate) smoke_ready: bool,
    pub(crate) smoke_command: String,
    pub(crate) smoke_output_root: String,
    pub(crate) smoke_input_vcf_path: String,
    pub(crate) smoke_sample_metadata_path: String,
    pub(crate) smoke_population_metadata_path: String,
    pub(crate) smoke_population_labels_manifest_path: String,
    pub(crate) smoke_pca_tsv_path: String,
    pub(crate) smoke_pca_json_path: String,
    pub(crate) smoke_source_eigenvec_path: String,
    pub(crate) smoke_source_eigenval_path: String,
    pub(crate) smoke_source_pca_manifest_path: String,
    pub(crate) smoke_source_logs_path: String,
    pub(crate) smoke_stage_result_manifest_path: String,
    pub(crate) smoke_execution_mode: String,
    pub(crate) smoke_tool_ok: bool,
    pub(crate) smoke_variant_count: u64,
    pub(crate) smoke_sample_count: u64,
    pub(crate) smoke_excluded_samples: Vec<String>,
    pub(crate) smoke_unexpected_samples: Vec<String>,
    pub(crate) smoke_eigenvalues: Vec<f64>,
    pub(crate) smoke_rows: Vec<LocalVcfPcaSmokeRow>,
    pub(crate) required_metric_names: Vec<String>,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) coverage_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfPcaReadyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) retained_row_count: usize,
    pub(crate) active_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) incomplete_row_count: usize,
    pub(crate) checked_surface_count: usize,
    pub(crate) required_metric_names: Vec<String>,
    pub(crate) tool_status_counts: BTreeMap<String, usize>,
    pub(crate) coverage_status_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<VcfPcaReadyRow>,
    pub(crate) violations: Vec<VcfPcaReadyRow>,
}

pub(crate) fn run_render_vcf_pca_ready(
    args: &parse::BenchReadinessRenderVcfPcaReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_pca_ready(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_PCA_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_pca_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfPcaReadyReport> {
    let _lock = bijux_dna_infra::FileLock::acquire(
        &repo_root.join(VCF_PCA_READY_LOCK_PATH),
        Duration::from_secs(300),
    )
    .with_context(|| {
        format!(
            "acquire VCF PCA readiness lock {}",
            repo_root.join(VCF_PCA_READY_LOCK_PATH).display()
        )
    })?;
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_vcf_pca_ready_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "vcf.pca active retained callers must stay complete across active-scope, command, output, parser, report, and sample-complete PCA smoke proof"
        ));
    }
    Ok(report)
}

fn build_vcf_pca_ready_report(repo_root: &Path, output_path: &Path) -> Result<VcfPcaReadyReport> {
    let (command_report, bindings) = collect_vcf_pca_bindings(repo_root)?;
    let active_bindings = bindings
        .into_iter()
        .filter(|binding| binding.retained_row.scope_state == "active")
        .collect::<Vec<_>>();

    let mut rows = Vec::with_capacity(active_bindings.len());
    for binding in active_bindings {
        let smoke_report = run_local_vcf_pca_smoke(repo_root, &binding.retained_row.tool_id).ok();
        rows.push(build_vcf_pca_ready_row(
            repo_root,
            &command_report,
            binding,
            smoke_report.as_ref(),
        )?);
    }
    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });

    let complete_row_count =
        rows.iter().filter(|row| row.coverage_status == COVERAGE_STATUS_COMPLETE).count();
    let incomplete_row_count = rows.len().saturating_sub(complete_row_count);
    let active_row_count = rows.iter().filter(|row| row.all_domain_active_row_present).count();
    let mut tool_status_counts = BTreeMap::<String, usize>::new();
    let mut coverage_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *tool_status_counts.entry(row.tool_status.clone()).or_default() += 1;
        *coverage_status_counts.entry(row.coverage_status.clone()).or_default() += 1;
    }
    let violations = rows
        .iter()
        .filter(|row| row.coverage_status != COVERAGE_STATUS_COMPLETE)
        .cloned()
        .collect::<Vec<_>>();

    let report = VcfPcaReadyReport {
        schema_version: VCF_PCA_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        retained_row_count: rows.len(),
        active_row_count,
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: 8,
        required_metric_names: required_metric_names(),
        tool_status_counts,
        coverage_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    };
    ensure_vcf_pca_ready_contract(&report)?;
    Ok(report)
}

fn build_vcf_pca_ready_row(
    repo_root: &Path,
    command_report: &VcfRenderedCommandsReport,
    binding: VcfPcaBinding,
    smoke_report: Option<&LocalVcfPcaSmokeReport>,
) -> Result<VcfPcaReadyRow> {
    let output_proof = load_pca_output_proof(repo_root, &binding.retained_row.tool_id)?;
    let result_id = binding
        .expected_row
        .as_ref()
        .map_or_else(|| retained_result_id(&binding.retained_row), |row| row.result_id.clone());
    let required_metric_names = required_metric_names();
    let mut missing_surfaces = Vec::new();

    let retained_scope_active = binding.retained_row.scope_state == "active";
    if !retained_scope_active {
        missing_surfaces.push("retained_vcf_active_scope".to_string());
    }

    let all_domain_active_row_present = binding.active_row.is_some();
    if !all_domain_active_row_present {
        missing_surfaces.push("all_domain_active_row".to_string());
    }

    let command_ready = binding.command_row.as_ref().is_some_and(|row| {
        row.benchmark_status == "benchmark_ready"
            && !row.command_steps.is_empty()
            && !row.script_commands.is_empty()
            && row.command_steps.iter().all(|step| {
                step.argv.first().is_some_and(|value| !value.trim().is_empty())
                    && !step.command.trim().is_empty()
            })
    });
    if !command_ready {
        missing_surfaces.push("vcf_rendered_commands".to_string());
    }

    let output_ready = output_proof.benchmark_status == "benchmark_ready"
        && contains_artifact_id(&output_proof.normalized_metrics_outputs, "pca_report")
        && !output_proof.manifest_output.trim().is_empty()
        && match binding.retained_row.tool_id.as_str() {
            "eigensoft" => {
                contains_artifact_id(&output_proof.raw_outputs, "eigensoft_geno")
                    && contains_artifact_id(&output_proof.raw_outputs, "eigensoft_snp")
                    && contains_artifact_id(&output_proof.raw_outputs, "eigensoft_ind")
                    && contains_artifact_id(&output_proof.raw_outputs, "smartpca_eigenvec")
                    && contains_artifact_id(&output_proof.raw_outputs, "smartpca_eigenval")
                    && contains_artifact_id(&output_proof.raw_outputs, "smartpca_log")
            }
            "plink2" => {
                contains_artifact_id(&output_proof.raw_outputs, "pca_eigenvec")
                    && contains_artifact_id(&output_proof.raw_outputs, "pca_eigenval")
                    && contains_artifact_id(&output_proof.raw_outputs, "plink2_log")
            }
            _ => false,
        };
    if !output_ready {
        missing_surfaces.push("vcf_adapter_output_coverage".to_string());
    }

    let parser_ready = binding.parser_row.as_ref().is_some_and(|row| {
        row.coverage_status == VcfParserFixtureCoverageStatus::Covered
            && !row.parser_id.trim().is_empty()
            && !row.parser_fixture_root_path.trim().is_empty()
            && !row.expected_normalized_path.trim().is_empty()
            && row.raw_fixture_count > 0
            && !row.schema_id.trim().is_empty()
    });
    if !parser_ready {
        missing_surfaces.push("vcf_parser_fixture_coverage".to_string());
    }

    let expected_result_ready = binding.expected_row.as_ref().is_some_and(|row| {
        row.report_section == "population_structure"
            && row.expected_outputs.iter().any(|value| value == "pca_report")
            && required_metric_names
                .iter()
                .all(|metric| row.expected_metrics.iter().any(|value| value == metric))
    });
    if !expected_result_ready {
        missing_surfaces.push("vcf_expected_benchmark_results".to_string());
    }

    let report_ready = binding.report_row.as_ref().is_some_and(|row| {
        row.section_id == "population_structure"
            && row.summary_table == "population_structure_metrics"
            && required_metric_names
                .iter()
                .all(|metric| row.metric_columns.iter().any(|value| value == metric))
    });
    if !report_ready {
        missing_surfaces.push("vcf_report_map".to_string());
    }

    let smoke_ready = smoke_report.is_some_and(pca_smoke_matches_governed_contract);
    if !smoke_ready {
        missing_surfaces.push("local_vcf_pca_smoke".to_string());
    }

    let coverage_status = if missing_surfaces.is_empty() {
        COVERAGE_STATUS_COMPLETE.to_string()
    } else {
        COVERAGE_STATUS_INCOMPLETE.to_string()
    };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "active retained VCF PCA caller `{}` keeps active scope, command, output, parser, expected-result, report, and sample-complete PCA smoke proof for `vcf.pca`",
            binding.retained_row.tool_id
        )
    } else {
        format!(
            "active retained VCF PCA caller `{}` is missing: {}",
            binding.retained_row.tool_id,
            missing_surfaces.join(", ")
        )
    };

    Ok(VcfPcaReadyRow {
        result_id,
        stage_id: binding.retained_row.stage_id,
        tool_id: binding.retained_row.tool_id,
        tool_status: binding.retained_row.tool_status,
        corpus_id: binding.retained_row.corpus_id,
        asset_profile_id: binding.retained_row.asset_profile_id,
        adapter_id: binding.retained_row.adapter_id,
        parser_id: binding.retained_row.parser_id,
        schema_id: binding.retained_row.schema_id,
        retained_scope_state: binding.retained_row.scope_state,
        retained_scope_detail: binding.retained_row.scope_detail,
        retained_scope_proof_path: binding.retained_row.scope_proof_path,
        all_domain_active_row_present,
        all_domain_active_row_proof_path:
            "benchmarks/readiness/all-domains/active-stage-tool-matrix.tsv".to_string(),
        command_ready,
        command_source: binding
            .command_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.command_source.clone()),
        command_step_count: binding.command_row.as_ref().map_or(0, |row| row.command_steps.len()),
        command_step_ids: binding
            .command_row
            .as_ref()
            .map(|row| row.command_steps.iter().map(|step| step.step_id.clone()).collect())
            .unwrap_or_default(),
        primary_executables: binding
            .command_row
            .as_ref()
            .map(|row| {
                row.command_steps
                    .iter()
                    .map(|step| step.argv.first().cloned().unwrap_or_else(no_value_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        command_output_path: command_report.output_path.clone(),
        command_argv_output_path: command_report.argv_output_path.clone(),
        output_ready,
        output_proof_path: output_proof.output_proof_path,
        raw_outputs: output_proof.raw_outputs,
        normalized_metrics_outputs: output_proof.normalized_metrics_outputs,
        manifest_output: output_proof.manifest_output,
        index_outputs: output_proof.index_outputs,
        parser_ready,
        parser_proof_path: DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH.to_string(),
        parser_fixture_parser_id: binding
            .parser_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.parser_fixture_parser_id.clone()),
        parser_fixture_schema_id: binding
            .parser_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.parser_fixture_schema_id.clone()),
        parser_fixture_path: binding
            .parser_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.parser_fixture_root_path.clone()),
        expected_result_ready,
        expected_result_proof_path: DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH.to_string(),
        expected_outputs: binding
            .expected_row
            .as_ref()
            .map(|row| row.expected_outputs.clone())
            .unwrap_or_default(),
        expected_metrics: binding
            .expected_row
            .as_ref()
            .map(|row| row.expected_metrics.clone())
            .unwrap_or_default(),
        report_section_id: binding
            .expected_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.report_section.clone()),
        report_ready,
        report_map_proof_path: DEFAULT_VCF_REPORT_MAP_PATH.to_string(),
        summary_table_id: binding
            .report_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.summary_table.clone()),
        report_metric_columns: binding
            .report_row
            .as_ref()
            .map(|row| row.metric_columns.clone())
            .unwrap_or_default(),
        smoke_ready,
        smoke_command: smoke_report.map_or_else(no_value_string, |report| report.command.clone()),
        smoke_output_root: smoke_report
            .map_or_else(no_value_string, |report| report.output_root.clone()),
        smoke_input_vcf_path: smoke_report
            .map_or_else(no_value_string, |report| report.input_vcf_path.clone()),
        smoke_sample_metadata_path: smoke_report
            .map_or_else(no_value_string, |report| report.sample_metadata_path.clone()),
        smoke_population_metadata_path: smoke_report
            .map_or_else(no_value_string, |report| report.population_metadata_path.clone()),
        smoke_population_labels_manifest_path: smoke_report
            .map_or_else(no_value_string, |report| report.population_labels_manifest_path.clone()),
        smoke_pca_tsv_path: smoke_report
            .map_or_else(no_value_string, |report| report.pca_tsv_path.clone()),
        smoke_pca_json_path: smoke_report
            .map_or_else(no_value_string, |report| report.pca_json_path.clone()),
        smoke_source_eigenvec_path: smoke_report
            .map_or_else(no_value_string, |report| report.source_eigenvec_path.clone()),
        smoke_source_eigenval_path: smoke_report
            .map_or_else(no_value_string, |report| report.source_eigenval_path.clone()),
        smoke_source_pca_manifest_path: smoke_report
            .map_or_else(no_value_string, |report| report.source_pca_manifest_path.clone()),
        smoke_source_logs_path: smoke_report
            .map_or_else(no_value_string, |report| report.source_logs_path.clone()),
        smoke_stage_result_manifest_path: smoke_report
            .map_or_else(no_value_string, |report| report.stage_result_manifest_path.clone()),
        smoke_execution_mode: smoke_report
            .map_or_else(no_value_string, |report| report.execution_mode.clone()),
        smoke_tool_ok: smoke_report.is_some_and(|report| report.tool_ok),
        smoke_variant_count: smoke_report.map_or(0, |report| report.variant_count),
        smoke_sample_count: smoke_report.map_or(0, |report| report.sample_count),
        smoke_excluded_samples: smoke_report
            .map(|report| report.excluded_samples.clone())
            .unwrap_or_default(),
        smoke_unexpected_samples: smoke_report
            .map(|report| report.unexpected_samples.clone())
            .unwrap_or_default(),
        smoke_eigenvalues: smoke_report
            .map(|report| report.eigenvalues.clone())
            .unwrap_or_default(),
        smoke_rows: smoke_report.map(|report| report.rows.clone()).unwrap_or_default(),
        required_metric_names,
        missing_surfaces,
        coverage_status,
        reason,
    })
}

fn collect_vcf_pca_bindings(
    repo_root: &Path,
) -> Result<(VcfRenderedCommandsReport, Vec<VcfPcaBinding>)> {
    let retained_rows = collect_vcf_active_stage_tool_matrix_rows(repo_root)?
        .into_iter()
        .filter(|row| row.stage_id == VCF_PCA_STAGE_ID)
        .collect::<Vec<_>>();
    if retained_rows.is_empty() {
        return Err(anyhow!("VCF PCA readiness is missing retained `vcf.pca` bindings"));
    }

    let active_by_key = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?
        .into_iter()
        .filter(|row| row.domain == "vcf" && row.stage_id == VCF_PCA_STAGE_ID)
        .map(|row| {
            (binding_key(&row.stage_id, &row.tool_id, &row.corpus_id, &row.asset_profile_id), row)
        })
        .collect::<BTreeMap<_, _>>();

    let command_report = render_vcf_pca_command_report(repo_root)?;
    let command_by_tool = command_report
        .rows
        .iter()
        .filter(|row| row.stage_id == VCF_PCA_STAGE_ID)
        .cloned()
        .map(|row| (row.tool_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let parser_by_tool = collect_vcf_parser_fixture_coverage_rows(repo_root)?
        .2
        .into_iter()
        .filter(|row| row.stage_id == VCF_PCA_STAGE_ID)
        .map(|row| (row.tool_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let expected_by_key = collect_vcf_pca_expected_result_proofs(repo_root)?
        .into_iter()
        .map(|row| {
            (
                binding_key(VCF_PCA_STAGE_ID, &row.tool_id, &row.corpus_id, &row.asset_profile_id),
                row,
            )
        })
        .collect::<BTreeMap<_, _>>();

    let report_by_tool = collect_vcf_pca_report_map_proofs(repo_root)?
        .into_iter()
        .map(|row| (row.tool_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::with_capacity(retained_rows.len());
    for retained_row in retained_rows {
        let key = binding_key(
            &retained_row.stage_id,
            &retained_row.tool_id,
            &retained_row.corpus_id,
            &retained_row.asset_profile_id,
        );
        rows.push(VcfPcaBinding {
            retained_row: retained_row.clone(),
            active_row: active_by_key.get(&key).cloned(),
            command_row: command_by_tool.get(&retained_row.tool_id).cloned(),
            parser_row: parser_by_tool.get(&retained_row.tool_id).cloned(),
            expected_row: expected_by_key.get(&key).cloned(),
            report_row: report_by_tool.get(&retained_row.tool_id).cloned(),
        });
    }

    Ok((command_report, rows))
}

fn render_vcf_pca_command_report(repo_root: &Path) -> Result<VcfRenderedCommandsReport> {
    let output_path = repo_root.join(VCF_PCA_RENDERED_COMMANDS_PATH);
    let argv_output_path = output_path.with_file_name("vcf-pca-rendered-commands.argv.jsonl");
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut rows = Vec::new();
    rows.extend(
        collect_vcf_plink_family_adapter_rows_for_tool(repo_root, "plink2")?
            .into_iter()
            .filter(|row| row.stage_id == VCF_PCA_STAGE_ID)
            .map(vcf_pca_plink_command_row),
    );
    rows.extend(
        collect_vcf_eigensoft_adapter_rows_for_tool(repo_root)?
            .into_iter()
            .filter(|row| row.stage_id == VCF_PCA_STAGE_ID)
            .map(vcf_pca_eigensoft_command_row),
    );
    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });

    let rendered_script = render_vcf_pca_commands_shell_script(
        &rows,
        &repo_root_relative_to_output(repo_root, &output_path),
    );
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered_script.as_bytes())?;
    let argv_jsonl = render_vcf_pca_command_argv_jsonl(&rows)?;
    bijux_dna_infra::atomic_write_bytes(&argv_output_path, argv_jsonl.as_bytes())?;

    Ok(VcfRenderedCommandsReport {
        schema_version: "bijux.bench.readiness.vcf_rendered_commands.v1",
        output_path: path_relative_to_repo(repo_root, &output_path),
        argv_output_path: path_relative_to_repo(repo_root, &argv_output_path),
        row_count: rows.len(),
        rows,
    })
}

fn vcf_pca_plink_command_row(row: VcfPlinkFamilyAdapterRow) -> VcfRenderedCommandRow {
    let command_steps = row
        .command_steps
        .into_iter()
        .map(|step| super::vcf_rendered_command_rows::VcfRenderedCommandStep {
            step_id: step.step_id,
            step_kind: step.step_kind,
            consumes_previous_stdout: false,
            command: render_shell_command(&step.argv),
            argv: step.argv,
        })
        .collect::<Vec<_>>();
    VcfRenderedCommandRow {
        stage_id: row.stage_id,
        tool_id: row.tool_id,
        readiness_kind: "benchmark_ready".to_string(),
        benchmark_status: row.benchmark_status,
        command_source: "vcf_plink_family_adapter".to_string(),
        script_commands: command_steps.iter().map(|step| step.command.clone()).collect(),
        command_steps,
        reason: row.reason,
    }
}

fn vcf_pca_eigensoft_command_row(row: VcfEigensoftAdapterRow) -> VcfRenderedCommandRow {
    let command_steps = row
        .command_steps
        .into_iter()
        .map(|step| super::vcf_rendered_command_rows::VcfRenderedCommandStep {
            step_id: step.step_id,
            step_kind: step.step_kind,
            consumes_previous_stdout: false,
            command: render_shell_command(&step.argv),
            argv: step.argv,
        })
        .collect::<Vec<_>>();
    VcfRenderedCommandRow {
        stage_id: row.stage_id,
        tool_id: row.tool_id,
        readiness_kind: "benchmark_ready".to_string(),
        benchmark_status: row.benchmark_status,
        command_source: "vcf_eigensoft_adapter".to_string(),
        script_commands: command_steps.iter().map(|step| step.command.clone()).collect(),
        command_steps,
        reason: row.reason,
    }
}

fn render_vcf_pca_commands_shell_script(
    rows: &[VcfRenderedCommandRow],
    repo_root_relative_to_output: &str,
) -> String {
    let mut rendered = String::from("#!/usr/bin/env bash\nset -euo pipefail\n");
    rendered.push_str(&format!(
        "repo_root=\"$(cd \"$(dirname \"${{BASH_SOURCE[0]}}\")/{repo_root_relative_to_output}\" && pwd)\"\n"
    ));
    rendered.push_str("cd \"$repo_root\"\n\n");
    for (index, row) in rows.iter().enumerate() {
        rendered.push_str(&format!("# {} / {}\n", row.stage_id, row.tool_id));
        for step in &row.command_steps {
            rendered.push_str(&step.command);
            rendered.push('\n');
        }
        if index + 1 < rows.len() {
            rendered.push('\n');
        }
    }
    rendered
}

fn render_vcf_pca_command_argv_jsonl(rows: &[VcfRenderedCommandRow]) -> Result<String> {
    let mut rendered = String::new();
    for row in rows {
        let payload = serde_json::json!({
            "stage_id": row.stage_id,
            "tool_id": row.tool_id,
            "readiness_kind": row.readiness_kind,
            "benchmark_status": row.benchmark_status,
            "command_source": row.command_source,
            "command_steps": row.command_steps.iter().map(|step| serde_json::json!({
                "step_id": step.step_id,
                "step_kind": step.step_kind,
                "consumes_previous_stdout": step.consumes_previous_stdout,
                "argv": step.argv,
            })).collect::<Vec<_>>(),
        });
        rendered.push_str(
            &serde_json::to_string(&payload).context("serialize VCF PCA command argv row")?,
        );
        rendered.push('\n');
    }
    Ok(rendered)
}

fn collect_vcf_pca_expected_result_proofs(
    repo_root: &Path,
) -> Result<Vec<VcfPcaExpectedResultProof>> {
    let path = repo_root.join(DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut rows = Vec::new();
    for (index, line) in raw.lines().enumerate() {
        if index == 0 || line.trim().is_empty() {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 8 || columns[0] != "vcf" || columns[1] != VCF_PCA_STAGE_ID {
            continue;
        }
        rows.push(VcfPcaExpectedResultProof {
            result_id: format!("vcf:{}:{}:{}:{}", columns[3], columns[1], columns[4], columns[2]),
            tool_id: columns[2].to_string(),
            corpus_id: columns[3].to_string(),
            asset_profile_id: columns[4].to_string(),
            expected_outputs: split_csv(columns[5]),
            expected_metrics: split_csv(columns[6]),
            report_section: columns[7].to_string(),
        });
    }
    Ok(rows)
}

fn collect_vcf_pca_report_map_proofs(repo_root: &Path) -> Result<Vec<VcfPcaReportMapProof>> {
    let path = repo_root.join(DEFAULT_VCF_REPORT_MAP_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut rows = Vec::new();
    for (index, line) in raw.lines().enumerate() {
        if index == 0 || line.trim().is_empty() {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 6 || columns[0] != VCF_PCA_STAGE_ID {
            continue;
        }
        rows.push(VcfPcaReportMapProof {
            tool_id: columns[1].to_string(),
            section_id: columns[2].to_string(),
            summary_table: columns[3].to_string(),
            metric_columns: split_csv(columns[4]),
        });
    }
    Ok(rows)
}

fn pca_smoke_matches_governed_contract(report: &LocalVcfPcaSmokeReport) -> bool {
    let expected_samples = REQUIRED_SAMPLE_IDS.iter().copied().collect::<BTreeSet<_>>();
    let observed_samples =
        report.rows.iter().map(|row| row.sample_id.as_str()).collect::<BTreeSet<_>>();

    !report.input_vcf_path.trim().is_empty()
        && !report.sample_metadata_path.trim().is_empty()
        && !report.population_metadata_path.trim().is_empty()
        && !report.population_labels_manifest_path.trim().is_empty()
        && !report.output_root.trim().is_empty()
        && !report.pca_tsv_path.trim().is_empty()
        && !report.pca_json_path.trim().is_empty()
        && !report.source_eigenvec_path.trim().is_empty()
        && !report.source_eigenval_path.trim().is_empty()
        && !report.source_pca_manifest_path.trim().is_empty()
        && !report.source_logs_path.trim().is_empty()
        && !report.stage_result_manifest_path.trim().is_empty()
        && report.variant_count == REQUIRED_VARIANT_COUNT
        && report.sample_count == REQUIRED_SAMPLE_COUNT
        && report.excluded_samples.is_empty()
        && report.unexpected_samples.is_empty()
        && report.eigenvalues.len() >= 2
        && report.rows.len() == REQUIRED_SAMPLE_IDS.len()
        && observed_samples == expected_samples
        && report.rows.iter().all(|row| {
            !row.population_id.trim().is_empty()
                && !row.population_label.trim().is_empty()
                && !row.sex.trim().is_empty()
                && row.pc1.is_finite()
                && row.pc2.is_finite()
        })
}

fn load_pca_output_proof(repo_root: &Path, tool_id: &str) -> Result<PcaAdapterOutputProof> {
    match tool_id {
        "plink2" => {
            let row = collect_vcf_plink_family_adapter_rows_for_tool(repo_root, "plink2")?
                .into_iter()
                .find(|row| row.stage_id == VCF_PCA_STAGE_ID)
                .ok_or_else(|| {
                    anyhow!("VCF PCA readiness is missing the retained `plink2` adapter row")
                })?;
            Ok(plink_output_proof(&row))
        }
        "eigensoft" => {
            let row = collect_vcf_eigensoft_adapter_rows_for_tool(repo_root)?
                .into_iter()
                .find(|row| row.stage_id == VCF_PCA_STAGE_ID)
                .ok_or_else(|| {
                    anyhow!("VCF PCA readiness is missing the retained `eigensoft` adapter row")
                })?;
            Ok(eigensoft_output_proof(&row))
        }
        other => Err(anyhow!("VCF PCA readiness does not own output proof for `{other}`")),
    }
}

fn plink_output_proof(row: &VcfPlinkFamilyAdapterRow) -> PcaAdapterOutputProof {
    let declared_by_id = row
        .declared_outputs
        .iter()
        .cloned()
        .map(|artifact| (artifact.artifact_id.clone(), artifact))
        .collect::<BTreeMap<_, _>>();
    PcaAdapterOutputProof {
        output_proof_path: DEFAULT_VCF_PLINK2_ADAPTER_PATH.to_string(),
        benchmark_status: row.benchmark_status.clone(),
        raw_outputs: row
            .raw_output_ids
            .iter()
            .filter_map(|artifact_id| declared_by_id.get(artifact_id))
            .filter(|artifact| !is_index_artifact(&artifact.path))
            .map(render_plink_artifact_entry)
            .collect(),
        normalized_metrics_outputs: row
            .stage_output_ids
            .iter()
            .filter_map(|artifact_id| declared_by_id.get(artifact_id))
            .map(render_plink_artifact_entry)
            .collect(),
        manifest_output: render_stage_result_manifest_path(&row.stage_id, &row.tool_id),
        index_outputs: row
            .declared_outputs
            .iter()
            .filter(|artifact| is_index_artifact(&artifact.path))
            .map(render_plink_artifact_entry)
            .collect(),
    }
}

fn eigensoft_output_proof(row: &VcfEigensoftAdapterRow) -> PcaAdapterOutputProof {
    let declared_by_id = row
        .declared_outputs
        .iter()
        .cloned()
        .map(|artifact| (artifact.artifact_id.clone(), artifact))
        .collect::<BTreeMap<_, _>>();
    PcaAdapterOutputProof {
        output_proof_path: DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH.to_string(),
        benchmark_status: row.benchmark_status.clone(),
        raw_outputs: row
            .raw_output_ids
            .iter()
            .filter_map(|artifact_id| declared_by_id.get(artifact_id))
            .filter(|artifact| !is_index_artifact(&artifact.path))
            .map(render_eigensoft_artifact_entry)
            .collect(),
        normalized_metrics_outputs: row
            .stage_output_ids
            .iter()
            .filter_map(|artifact_id| declared_by_id.get(artifact_id))
            .map(render_eigensoft_artifact_entry)
            .collect(),
        manifest_output: render_stage_result_manifest_path(&row.stage_id, &row.tool_id),
        index_outputs: row
            .declared_outputs
            .iter()
            .filter(|artifact| is_index_artifact(&artifact.path))
            .map(render_eigensoft_artifact_entry)
            .collect(),
    }
}

fn ensure_vcf_pca_ready_contract(report: &VcfPcaReadyReport) -> Result<()> {
    if report.retained_row_count != report.rows.len() {
        return Err(anyhow!(
            "VCF PCA readiness must keep exactly one row per active retained `vcf.pca` binding"
        ));
    }
    if report.rows.is_empty() {
        return Err(anyhow!("VCF PCA readiness must keep at least one active retained caller row"));
    }
    if report.checked_surface_count != 8 {
        return Err(anyhow!("VCF PCA readiness must record exactly 8 checked surfaces"));
    }
    let unique_results =
        report.rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>().len();
    if unique_results != report.rows.len() {
        return Err(anyhow!(
            "VCF PCA readiness must keep one unique result_id per active retained caller row"
        ));
    }
    let observed_tools =
        report.rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>();
    let expected_tools = BTreeSet::from(["eigensoft", "plink2"]);
    if observed_tools != expected_tools {
        return Err(anyhow!(
            "VCF PCA readiness must retain the governed PCA tools `eigensoft` and `plink2`, found {observed_tools:?}"
        ));
    }
    for row in &report.rows {
        if row.stage_id != VCF_PCA_STAGE_ID {
            return Err(anyhow!(
                "VCF PCA readiness row `{}` drifted away from the `vcf.pca` stage",
                row.stage_id
            ));
        }
        if row.coverage_status == COVERAGE_STATUS_COMPLETE && !row.missing_surfaces.is_empty() {
            return Err(anyhow!(
                "VCF PCA readiness row `{}` / `{}` cannot be complete while listing missing surfaces",
                row.stage_id,
                row.tool_id
            ));
        }
    }
    Ok(())
}

fn binding_key(stage_id: &str, tool_id: &str, corpus_id: &str, asset_profile_id: &str) -> String {
    format!("{stage_id}:{tool_id}:{corpus_id}:{asset_profile_id}")
}

fn retained_result_id(row: &VcfActiveStageToolMatrixRow) -> String {
    format!("vcf:{}:{}:{}:{}", row.corpus_id, row.stage_id, row.asset_profile_id, row.tool_id)
}

fn contains_artifact_id(entries: &[String], expected_id: &str) -> bool {
    entries.iter().any(|entry| artifact_id(entry) == expected_id)
}

fn render_plink_artifact_entry(artifact: &VcfPlinkFamilyAdapterArtifact) -> String {
    format!("{}={}", artifact.artifact_id, artifact.path)
}

fn render_eigensoft_artifact_entry(artifact: &VcfEigensoftAdapterArtifact) -> String {
    format!("{}={}", artifact.artifact_id, artifact.path)
}

fn render_stage_result_manifest_path(stage_id: &str, tool_id: &str) -> String {
    format!(
        "runs/bench/slurm-dry-run/runs/{}/{}/{}/{}/{}/stage-result.json",
        LOCAL_SLURM_DRY_RUN_RUN_ID, "{fixture_scope}", stage_id, "{sample_scope}", tool_id
    )
}

fn artifact_id(entry: &str) -> &str {
    entry.split_once('=').map_or(entry, |(id, _)| id)
}

fn is_index_artifact(path: &str) -> bool {
    path.ends_with(".tbi") || path.ends_with(".csi") || path.ends_with(".fai")
}

fn split_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn required_metric_names() -> Vec<String> {
    REQUIRED_METRIC_NAMES.iter().map(|value| (*value).to_string()).collect()
}

fn no_value_string() -> String {
    NO_VALUE.to_string()
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn repo_root_relative_to_output(repo_root: &Path, output_path: &Path) -> String {
    let relative_output_path = output_path.strip_prefix(repo_root).unwrap_or(output_path);
    let depth = relative_output_path.parent().map_or(0, |parent| parent.components().count());
    if depth == 0 {
        ".".to_string()
    } else {
        std::iter::repeat_n("..", depth).collect::<Vec<_>>().join("/")
    }
}
