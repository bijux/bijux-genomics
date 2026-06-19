use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::Value;

use super::local_adna_micro_pipeline::{
    AdnaMicroPipelineReport, AdnaMicroPipelineRow,
};
use super::local_amplicon_micro_pipeline::{AmpliconMicroPipelineReport, AmpliconMicroPipelineRow};
use super::local_bam_micro_smoke_subset::{BamMicroSmokeFamilyRow, BamMicroSmokeSubsetReport};
use super::local_core_germline_micro_pipeline::{
    CoreGermlineMicroPipelineReport, CoreGermlineMicroPipelineRow,
};
use super::local_edna_micro_pipeline::{EdnaMicroPipelineReport, EdnaMicroPipelineRow};
use super::local_fastq_micro_smoke_subset::{
    FastqMicroSmokeFamilyRow, FastqMicroSmokeSubsetReport,
};
use super::local_micro_benchmark_run::{
    MicroBenchmarkExecutionStatus, MicroBenchmarkLogRow, MicroBenchmarkOutputRow,
    MicroBenchmarkResultRow, MicroBenchmarkRunManifest,
};
use super::local_real_smoke_core_subset::{RealSmokeCoreSubsetReport, RealSmokeCoreSubsetRow};
use super::local_stage_result_manifest::{
    load_validated_stage_result_manifest_path, path_relative_to_repo,
};
use super::local_vcf_micro_smoke_subset::{VcfMicroSmokeFamilyRow, VcfMicroSmokeSubsetReport};
use super::path_resolution::{ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver};
use super::readiness::scientific_acceptance_thresholds::{
    scientific_acceptance_direction_label, scientific_acceptance_insufficiency_behavior_label,
    scientific_acceptance_pass_rule_label, scientific_acceptance_tolerance_kind_label,
    ScientificAcceptanceThresholdRow, ScientificAcceptanceThresholdsConfig,
};
use super::readiness::stage_tool_resources::{StageToolResourceRow, StageToolResourcesConfig};

pub(crate) const DEFAULT_MICRO_BENCHMARK_REPORT_MARKDOWN_PATH: &str =
    "runs/bench/micro/MICRO_BENCHMARK_REPORT.md";
pub(crate) const DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH: &str =
    "runs/bench/micro/MICRO_BENCHMARK_REPORT.json";

const MICRO_BENCHMARK_REPORT_SCHEMA_VERSION: &str = "bijux.bench.local_micro_benchmark_report.v1";
const SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH: &str =
    "benchmarks/configs/local/scientific-acceptance-thresholds.toml";
const STAGE_TOOL_RESOURCES_PATH: &str = "benchmarks/configs/local/stage-tool-resources.toml";

pub(crate) struct MicroBenchmarkSourceReports<'a> {
    pub(crate) real_smoke: &'a RealSmokeCoreSubsetReport,
    pub(crate) fastq: &'a FastqMicroSmokeSubsetReport,
    pub(crate) bam: &'a BamMicroSmokeSubsetReport,
    pub(crate) vcf: &'a VcfMicroSmokeSubsetReport,
    pub(crate) amplicon: &'a AmpliconMicroPipelineReport,
    pub(crate) adna: &'a AdnaMicroPipelineReport,
    pub(crate) edna: &'a EdnaMicroPipelineReport,
    pub(crate) core_germline: &'a CoreGermlineMicroPipelineReport,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MicroBenchmarkCompleteRow {
    pub(crate) execution_id: String,
    pub(crate) component_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) source_report_path: String,
    pub(crate) evidence_path: Option<String>,
    pub(crate) command: Option<String>,
    pub(crate) reason: String,
    pub(crate) normalized_metric_count: usize,
    pub(crate) output_count: usize,
    pub(crate) log_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MicroBenchmarkFailedRow {
    pub(crate) row_id: String,
    pub(crate) source_surface: String,
    pub(crate) component_id: Option<String>,
    pub(crate) execution_id: Option<String>,
    pub(crate) domain: Option<String>,
    pub(crate) stage_id: Option<String>,
    pub(crate) tool_id: Option<String>,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MicroBenchmarkMissingRow {
    pub(crate) component_id: String,
    pub(crate) execution_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) source_report_path: String,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MicroBenchmarkUnavailableRow {
    pub(crate) execution_id: String,
    pub(crate) component_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) execution_status: MicroBenchmarkExecutionStatus,
    pub(crate) source_report_path: String,
    pub(crate) command: Option<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MicroBenchmarkInsufficientDataRow {
    pub(crate) execution_id: String,
    pub(crate) component_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) source_report_path: String,
    pub(crate) evidence_path: Option<String>,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MicroBenchmarkRuntimeRow {
    pub(crate) execution_id: String,
    pub(crate) component_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) execution_status: MicroBenchmarkExecutionStatus,
    pub(crate) elapsed_seconds: Option<f64>,
    pub(crate) runtime_source: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MicroBenchmarkMemorySourceRow {
    pub(crate) execution_id: String,
    pub(crate) component_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) execution_status: MicroBenchmarkExecutionStatus,
    pub(crate) declared_memory_mb: Option<f64>,
    pub(crate) declared_cpu_threads: Option<u32>,
    pub(crate) observed_memory_mb: Option<f64>,
    pub(crate) observed_cpu_threads: Option<u32>,
    pub(crate) memory_source: String,
    pub(crate) resource_origin: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MicroBenchmarkScienceThresholdRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) metric_id: String,
    pub(crate) metric_name: String,
    pub(crate) unit: Option<String>,
    pub(crate) direction: String,
    pub(crate) tolerance_kind: String,
    pub(crate) tolerance_value: f64,
    pub(crate) pass_rule: String,
    pub(crate) insufficiency_behavior: String,
    pub(crate) required: bool,
    pub(crate) declaration_origin: String,
    pub(crate) covered_tool_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MicroBenchmarkReport {
    pub(crate) schema_version: &'static str,
    pub(crate) markdown_output_path: String,
    pub(crate) json_output_path: String,
    pub(crate) micro_run_manifest_path: String,
    pub(crate) result_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) failed_row_count: usize,
    pub(crate) missing_row_count: usize,
    pub(crate) unavailable_row_count: usize,
    pub(crate) insufficient_data_row_count: usize,
    pub(crate) runtime_row_count: usize,
    pub(crate) memory_source_row_count: usize,
    pub(crate) science_threshold_row_count: usize,
    pub(crate) passes_behavior_test: bool,
    pub(crate) execution_status_counts: BTreeMap<String, usize>,
    pub(crate) runtime_source_counts: BTreeMap<String, usize>,
    pub(crate) memory_source_counts: BTreeMap<String, usize>,
    pub(crate) complete_rows: Vec<MicroBenchmarkCompleteRow>,
    pub(crate) failed_rows: Vec<MicroBenchmarkFailedRow>,
    pub(crate) missing_rows: Vec<MicroBenchmarkMissingRow>,
    pub(crate) unavailable_rows: Vec<MicroBenchmarkUnavailableRow>,
    pub(crate) insufficient_data_rows: Vec<MicroBenchmarkInsufficientDataRow>,
    pub(crate) runtime_rows: Vec<MicroBenchmarkRuntimeRow>,
    pub(crate) memory_source_rows: Vec<MicroBenchmarkMemorySourceRow>,
    pub(crate) science_threshold_rows: Vec<MicroBenchmarkScienceThresholdRow>,
}

#[derive(Debug, Clone)]
struct ExpectedMicroCoverageRow {
    component_id: String,
    execution_id: String,
    domain: String,
    stage_id: String,
    tool_id: String,
    source_report_path: String,
}

#[derive(Debug, Clone)]
struct RuntimeEvidence {
    elapsed_seconds: Option<f64>,
    runtime_source: &'static str,
}

#[derive(Debug, Clone)]
struct MemoryEvidence {
    declared_memory_mb: Option<f64>,
    declared_cpu_threads: Option<u32>,
    observed_memory_mb: Option<f64>,
    observed_cpu_threads: Option<u32>,
    memory_source: &'static str,
    resource_origin: Option<String>,
}

pub(crate) fn render_micro_benchmark_report(
    repo_root: &Path,
    manifest: &MicroBenchmarkRunManifest,
    result_rows: &[MicroBenchmarkResultRow],
    output_rows: &[MicroBenchmarkOutputRow],
    log_rows: &[MicroBenchmarkLogRow],
    markdown_output_path: PathBuf,
    json_output_path: PathBuf,
    source_reports: MicroBenchmarkSourceReports<'_>,
) -> Result<MicroBenchmarkReport> {
    let benchmark_paths = BenchmarkPathResolver::new(repo_root, None);
    let markdown_output_path = benchmark_paths.resolve_repo_relative(&markdown_output_path);
    let json_output_path = benchmark_paths.resolve_repo_relative(&json_output_path);
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &markdown_output_path,
        "micro benchmark markdown report",
    )?;
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &json_output_path,
        "micro benchmark json report",
    )?;
    for path in [&markdown_output_path, &json_output_path] {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
    }

    let expected_rows = collect_expected_micro_rows(&source_reports);
    let actual_key_counts =
        result_rows.iter().fold(BTreeMap::<(&str, &str), usize>::new(), |mut counts, row| {
            *counts.entry((row.component_id.as_str(), row.execution_id.as_str())).or_default() += 1;
            counts
        });
    let expected_keys = expected_rows
        .iter()
        .map(|row| (row.component_id.as_str(), row.execution_id.as_str()))
        .collect::<BTreeSet<_>>();

    let complete_rows = result_rows
        .iter()
        .filter(|row| row.status == MicroBenchmarkExecutionStatus::Succeeded)
        .map(|row| MicroBenchmarkCompleteRow {
            execution_id: row.execution_id.clone(),
            component_id: row.component_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            source_report_path: row.source_report_path.clone(),
            evidence_path: row.evidence_path.clone(),
            command: row.command.clone(),
            reason: row.reason.clone(),
            normalized_metric_count: row.normalized_metric_count,
            output_count: row.output_count,
            log_count: row.log_count,
        })
        .collect::<Vec<_>>();

    let unavailable_rows = result_rows
        .iter()
        .filter(|row| row.status != MicroBenchmarkExecutionStatus::Succeeded)
        .map(|row| MicroBenchmarkUnavailableRow {
            execution_id: row.execution_id.clone(),
            component_id: row.component_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            execution_status: row.status,
            source_report_path: row.source_report_path.clone(),
            command: row.command.clone(),
            reason: row.reason.clone(),
        })
        .collect::<Vec<_>>();

    let mut failed_rows = Vec::new();
    for ((component_id, execution_id), count) in &actual_key_counts {
        if *count > 1 {
            failed_rows.push(MicroBenchmarkFailedRow {
                row_id: format!("duplicate:{component_id}:{execution_id}"),
                source_surface: "micro_result_rows".to_string(),
                component_id: Some((*component_id).to_string()),
                execution_id: Some((*execution_id).to_string()),
                domain: None,
                stage_id: None,
                tool_id: None,
                detail: format!(
                    "micro benchmark result rows duplicated `{component_id}` / `{execution_id}` {count} times"
                ),
            });
        }
    }
    for row in result_rows {
        if !expected_keys.contains(&(row.component_id.as_str(), row.execution_id.as_str())) {
            failed_rows.push(MicroBenchmarkFailedRow {
                row_id: format!("unexpected:{}:{}", row.component_id, row.execution_id),
                source_surface: "micro_result_rows".to_string(),
                component_id: Some(row.component_id.clone()),
                execution_id: Some(row.execution_id.clone()),
                domain: Some(row.domain.clone()),
                stage_id: Some(row.stage_id.clone()),
                tool_id: Some(row.tool_id.clone()),
                detail: "micro benchmark result row has no matching source-report row".to_string(),
            });
        }
        if row.status == MicroBenchmarkExecutionStatus::Succeeded {
            match &row.evidence_path {
                Some(evidence_path) if repo_root.join(evidence_path).is_file() => {}
                Some(_) => failed_rows.push(MicroBenchmarkFailedRow {
                    row_id: format!("missing-evidence:{}:{}", row.component_id, row.execution_id),
                    source_surface: "micro_result_rows".to_string(),
                    component_id: Some(row.component_id.clone()),
                    execution_id: Some(row.execution_id.clone()),
                    domain: Some(row.domain.clone()),
                    stage_id: Some(row.stage_id.clone()),
                    tool_id: Some(row.tool_id.clone()),
                    detail: "successful micro benchmark row references a missing evidence file"
                        .to_string(),
                }),
                None => failed_rows.push(MicroBenchmarkFailedRow {
                    row_id: format!("blank-evidence:{}:{}", row.component_id, row.execution_id),
                    source_surface: "micro_result_rows".to_string(),
                    component_id: Some(row.component_id.clone()),
                    execution_id: Some(row.execution_id.clone()),
                    domain: Some(row.domain.clone()),
                    stage_id: Some(row.stage_id.clone()),
                    tool_id: Some(row.tool_id.clone()),
                    detail: "successful micro benchmark row is missing an evidence path"
                        .to_string(),
                }),
            }
        }
    }
    for output_row in output_rows.iter().filter(|row| !row.exists) {
        failed_rows.push(MicroBenchmarkFailedRow {
            row_id: format!(
                "missing-output:{}:{}",
                output_row.component_id, output_row.execution_id
            ),
            source_surface: "micro_output_rows".to_string(),
            component_id: Some(output_row.component_id.clone()),
            execution_id: Some(output_row.execution_id.clone()),
            domain: None,
            stage_id: None,
            tool_id: None,
            detail: format!("output row references a missing path `{}`", output_row.path),
        });
    }
    for log_row in log_rows.iter().filter(|row| !row.exists) {
        failed_rows.push(MicroBenchmarkFailedRow {
            row_id: format!("missing-log:{}:{}", log_row.component_id, log_row.execution_id),
            source_surface: "micro_log_rows".to_string(),
            component_id: Some(log_row.component_id.clone()),
            execution_id: Some(log_row.execution_id.clone()),
            domain: None,
            stage_id: None,
            tool_id: None,
            detail: format!("log row references a missing path `{}`", log_row.path),
        });
    }

    let missing_rows = expected_rows
        .iter()
        .filter(|row| {
            !actual_key_counts.contains_key(&(row.component_id.as_str(), row.execution_id.as_str()))
        })
        .map(|row| MicroBenchmarkMissingRow {
            component_id: row.component_id.clone(),
            execution_id: row.execution_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            source_report_path: row.source_report_path.clone(),
            detail: "source-report row was not collected into the micro benchmark result set"
                .to_string(),
        })
        .collect::<Vec<_>>();

    let insufficient_data_rows = result_rows
        .iter()
        .filter_map(|row| detect_insufficient_data_row(repo_root, row).transpose())
        .collect::<Result<Vec<_>>>()?;

    let stage_tool_resources =
        load_stage_tool_resources(repo_root.join(STAGE_TOOL_RESOURCES_PATH))?;
    let science_threshold_rows = load_science_threshold_rows(
        repo_root.join(SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH),
        result_rows,
    )?;

    let mut runtime_rows = Vec::with_capacity(result_rows.len());
    let mut memory_source_rows = Vec::with_capacity(result_rows.len());
    let mut runtime_source_counts = BTreeMap::<String, usize>::new();
    let mut memory_source_counts = BTreeMap::<String, usize>::new();
    for row in result_rows {
        let runtime = load_runtime_evidence(repo_root, row)?;
        let memory = load_memory_evidence(repo_root, row, &stage_tool_resources)?;
        runtime_rows.push(MicroBenchmarkRuntimeRow {
            execution_id: row.execution_id.clone(),
            component_id: row.component_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            execution_status: row.status,
            elapsed_seconds: runtime.elapsed_seconds,
            runtime_source: runtime.runtime_source.to_string(),
        });
        memory_source_rows.push(MicroBenchmarkMemorySourceRow {
            execution_id: row.execution_id.clone(),
            component_id: row.component_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            execution_status: row.status,
            declared_memory_mb: memory.declared_memory_mb,
            declared_cpu_threads: memory.declared_cpu_threads,
            observed_memory_mb: memory.observed_memory_mb,
            observed_cpu_threads: memory.observed_cpu_threads,
            memory_source: memory.memory_source.to_string(),
            resource_origin: memory.resource_origin,
        });
        *runtime_source_counts.entry(runtime.runtime_source.to_string()).or_default() += 1;
        *memory_source_counts.entry(memory.memory_source.to_string()).or_default() += 1;
    }

    let mut execution_status_counts = BTreeMap::<String, usize>::new();
    for row in result_rows {
        *execution_status_counts
            .entry(micro_execution_status_label(row.status).to_string())
            .or_default() += 1;
    }

    let mut report = MicroBenchmarkReport {
        schema_version: MICRO_BENCHMARK_REPORT_SCHEMA_VERSION,
        markdown_output_path: path_relative_to_repo(repo_root, &markdown_output_path),
        json_output_path: path_relative_to_repo(repo_root, &json_output_path),
        micro_run_manifest_path: manifest.manifest_path.clone(),
        result_row_count: result_rows.len(),
        complete_row_count: complete_rows.len(),
        failed_row_count: failed_rows.len(),
        missing_row_count: missing_rows.len(),
        unavailable_row_count: unavailable_rows.len(),
        insufficient_data_row_count: insufficient_data_rows.len(),
        runtime_row_count: runtime_rows.len(),
        memory_source_row_count: memory_source_rows.len(),
        science_threshold_row_count: science_threshold_rows.len(),
        passes_behavior_test: false,
        execution_status_counts,
        runtime_source_counts,
        memory_source_counts,
        complete_rows,
        failed_rows,
        missing_rows,
        unavailable_rows,
        insufficient_data_rows,
        runtime_rows,
        memory_source_rows,
        science_threshold_rows,
    };
    ensure_micro_benchmark_report_contract(manifest, result_rows, &mut report)?;

    fs::write(&markdown_output_path, render_micro_benchmark_report_markdown(&report))
        .with_context(|| format!("write {}", markdown_output_path.display()))?;
    bijux_dna_infra::atomic_write_json(&json_output_path, &report)?;
    Ok(report)
}

fn collect_expected_micro_rows(
    reports: &MicroBenchmarkSourceReports<'_>,
) -> Vec<ExpectedMicroCoverageRow> {
    let mut rows = Vec::new();
    rows.extend(
        reports
            .real_smoke
            .rows
            .iter()
            .map(|row| expected_real_smoke_row(reports.real_smoke.output_path.clone(), row)),
    );
    rows.extend(reports.fastq.rows.iter().map(|row| {
        expected_family_row(
            "fastq_micro_smoke_subset",
            "fastq",
            reports.fastq.output_path.clone(),
            row,
        )
    }));
    rows.extend(
        reports
            .bam
            .rows
            .iter()
            .map(|row| expected_bam_family_row(reports.bam.output_path.clone(), row)),
    );
    rows.extend(
        reports
            .vcf
            .rows
            .iter()
            .map(|row| expected_vcf_family_row(reports.vcf.output_path.clone(), row)),
    );
    rows.extend(reports.amplicon.rows.iter().map(|row| {
        expected_pipeline_row("amplicon_micro_pipeline", reports.amplicon.output_path.clone(), row)
    }));
    rows.extend(
        reports
            .adna
            .rows
            .iter()
            .map(|row| expected_adna_pipeline_row(reports.adna.output_path.clone(), row)),
    );
    rows.extend(
        reports
            .edna
            .rows
            .iter()
            .map(|row| expected_edna_pipeline_row(reports.edna.output_path.clone(), row)),
    );
    rows.extend(reports.core_germline.rows.iter().map(|row| {
        expected_core_germline_pipeline_row(reports.core_germline.output_path.clone(), row)
    }));
    rows.sort_by(|left, right| {
        left.component_id
            .cmp(&right.component_id)
            .then_with(|| left.execution_id.cmp(&right.execution_id))
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    rows
}

fn expected_real_smoke_row(
    source_report_path: String,
    row: &RealSmokeCoreSubsetRow,
) -> ExpectedMicroCoverageRow {
    ExpectedMicroCoverageRow {
        component_id: "real_smoke_core_subset".to_string(),
        execution_id: row.execution_id.clone(),
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        source_report_path,
    }
}

fn expected_family_row(
    component_id: &str,
    domain: &str,
    source_report_path: String,
    row: &FastqMicroSmokeFamilyRow,
) -> ExpectedMicroCoverageRow {
    ExpectedMicroCoverageRow {
        component_id: component_id.to_string(),
        execution_id: row.family_id.clone(),
        domain: domain.to_string(),
        stage_id: row.representative_stage_id.clone(),
        tool_id: row.representative_tool_id.clone(),
        source_report_path,
    }
}

fn expected_bam_family_row(
    source_report_path: String,
    row: &BamMicroSmokeFamilyRow,
) -> ExpectedMicroCoverageRow {
    ExpectedMicroCoverageRow {
        component_id: "bam_micro_smoke_subset".to_string(),
        execution_id: row.family_id.clone(),
        domain: "bam".to_string(),
        stage_id: row.representative_stage_id.clone(),
        tool_id: row.representative_tool_id.clone(),
        source_report_path,
    }
}

fn expected_vcf_family_row(
    source_report_path: String,
    row: &VcfMicroSmokeFamilyRow,
) -> ExpectedMicroCoverageRow {
    ExpectedMicroCoverageRow {
        component_id: "vcf_micro_smoke_subset".to_string(),
        execution_id: row.family_id.clone(),
        domain: "vcf".to_string(),
        stage_id: row.representative_stage_id.clone(),
        tool_id: row.representative_tool_id.clone(),
        source_report_path,
    }
}

fn expected_pipeline_row(
    component_id: &str,
    source_report_path: String,
    row: &AmpliconMicroPipelineRow,
) -> ExpectedMicroCoverageRow {
    ExpectedMicroCoverageRow {
        component_id: component_id.to_string(),
        execution_id: row.stage_id.clone(),
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        source_report_path,
    }
}

fn expected_adna_pipeline_row(
    source_report_path: String,
    row: &AdnaMicroPipelineRow,
) -> ExpectedMicroCoverageRow {
    ExpectedMicroCoverageRow {
        component_id: "adna_micro_pipeline".to_string(),
        execution_id: row.stage_id.clone(),
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        source_report_path,
    }
}

fn expected_edna_pipeline_row(
    source_report_path: String,
    row: &EdnaMicroPipelineRow,
) -> ExpectedMicroCoverageRow {
    ExpectedMicroCoverageRow {
        component_id: "edna_micro_pipeline".to_string(),
        execution_id: row.stage_id.clone(),
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        source_report_path,
    }
}

fn expected_core_germline_pipeline_row(
    source_report_path: String,
    row: &CoreGermlineMicroPipelineRow,
) -> ExpectedMicroCoverageRow {
    ExpectedMicroCoverageRow {
        component_id: "core_germline_micro_pipeline".to_string(),
        execution_id: row.stage_id.clone(),
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        source_report_path,
    }
}

fn detect_insufficient_data_row(
    repo_root: &Path,
    row: &MicroBenchmarkResultRow,
) -> Result<Option<MicroBenchmarkInsufficientDataRow>> {
    let reason_lower = row.reason.to_ascii_lowercase();
    if reason_lower.contains("insufficient") {
        return Ok(Some(MicroBenchmarkInsufficientDataRow {
            execution_id: row.execution_id.clone(),
            component_id: row.component_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            source_report_path: row.source_report_path.clone(),
            evidence_path: row.evidence_path.clone(),
            detail: row.reason.clone(),
        }));
    }
    let Some(evidence_path) = &row.evidence_path else {
        return Ok(None);
    };
    if !evidence_path.ends_with(".json") {
        return Ok(None);
    }
    let payload = read_json_value(repo_root.join(evidence_path))?;
    if json_contains_insufficient_marker(&payload) {
        return Ok(Some(MicroBenchmarkInsufficientDataRow {
            execution_id: row.execution_id.clone(),
            component_id: row.component_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            source_report_path: row.source_report_path.clone(),
            evidence_path: row.evidence_path.clone(),
            detail: "evidence report retains an insufficient-data marker".to_string(),
        }));
    }
    Ok(None)
}

fn json_contains_insufficient_marker(value: &Value) -> bool {
    match value {
        Value::String(text) => {
            let text = text.to_ascii_lowercase();
            text.contains("insufficient_data")
                || text.contains("insufficient-data")
                || text.contains("insufficient_overlap")
                || text.contains("insufficient-overlap")
        }
        Value::Array(values) => values.iter().any(json_contains_insufficient_marker),
        Value::Object(map) => map.values().any(json_contains_insufficient_marker),
        _ => false,
    }
}

fn load_runtime_evidence(
    repo_root: &Path,
    row: &MicroBenchmarkResultRow,
) -> Result<RuntimeEvidence> {
    if row.status != MicroBenchmarkExecutionStatus::Succeeded {
        return Ok(RuntimeEvidence { elapsed_seconds: None, runtime_source: "not_applicable" });
    }
    if let Some(manifest_path) = &row.stage_result_manifest_path {
        let manifest = load_validated_stage_result_manifest_path(&repo_root.join(manifest_path))
            .with_context(|| format!("load {}", repo_root.join(manifest_path).display()))?;
        return Ok(RuntimeEvidence {
            elapsed_seconds: Some(manifest.runtime.elapsed_seconds),
            runtime_source: "stage_result_manifest",
        });
    }
    if let Some(evidence_path) = &row.evidence_path {
        if evidence_path.ends_with(".json") {
            let payload = read_json_value(repo_root.join(evidence_path))?;
            if let Some(elapsed_seconds) = json_optional_f64(&payload, "elapsed_seconds")
                .or_else(|| json_optional_f64(&payload, "runtime_s"))
            {
                return Ok(RuntimeEvidence {
                    elapsed_seconds: Some(elapsed_seconds),
                    runtime_source: "evidence_report",
                });
            }
        }
    }
    Ok(RuntimeEvidence { elapsed_seconds: None, runtime_source: "not_available" })
}

fn load_memory_evidence(
    repo_root: &Path,
    row: &MicroBenchmarkResultRow,
    stage_tool_resources: &BTreeMap<(String, String, String), StageToolResourceRow>,
) -> Result<MemoryEvidence> {
    if row.status != MicroBenchmarkExecutionStatus::Succeeded {
        return Ok(MemoryEvidence {
            declared_memory_mb: None,
            declared_cpu_threads: None,
            observed_memory_mb: None,
            observed_cpu_threads: None,
            memory_source: "not_applicable",
            resource_origin: None,
        });
    }
    if let Some(manifest_path) = &row.stage_result_manifest_path {
        let manifest = load_validated_stage_result_manifest_path(&repo_root.join(manifest_path))
            .with_context(|| format!("load {}", repo_root.join(manifest_path).display()))?;
        if manifest.resource_metrics.memory_mb.is_some()
            || manifest.resource_metrics.cpu_threads.is_some()
        {
            return Ok(MemoryEvidence {
                declared_memory_mb: None,
                declared_cpu_threads: None,
                observed_memory_mb: manifest.resource_metrics.memory_mb,
                observed_cpu_threads: manifest.resource_metrics.cpu_threads,
                memory_source: "stage_result_manifest",
                resource_origin: None,
            });
        }
    }
    if let Some(evidence_path) = &row.evidence_path {
        if evidence_path.ends_with(".json") {
            let payload = read_json_value(repo_root.join(evidence_path))?;
            let observed_memory_mb = json_optional_f64(&payload, "memory_mb")
                .or_else(|| json_optional_f64(&payload, "peak_memory_mb"));
            let observed_cpu_threads = json_optional_u32(&payload, "cpu_threads")
                .or_else(|| json_optional_u32(&payload, "threads"));
            if observed_memory_mb.is_some() || observed_cpu_threads.is_some() {
                return Ok(MemoryEvidence {
                    declared_memory_mb: None,
                    declared_cpu_threads: None,
                    observed_memory_mb,
                    observed_cpu_threads,
                    memory_source: "evidence_report",
                    resource_origin: None,
                });
            }
        }
    }
    if let Some(resource) =
        stage_tool_resources.get(&(row.domain.clone(), row.stage_id.clone(), row.tool_id.clone()))
    {
        return Ok(MemoryEvidence {
            declared_memory_mb: Some(f64::from(resource.memory_gb) * 1024.0),
            declared_cpu_threads: Some(resource.threads),
            observed_memory_mb: None,
            observed_cpu_threads: None,
            memory_source: "declared_stage_tool_resource",
            resource_origin: Some(resource.resource_origin.clone()),
        });
    }
    Ok(MemoryEvidence {
        declared_memory_mb: None,
        declared_cpu_threads: None,
        observed_memory_mb: None,
        observed_cpu_threads: None,
        memory_source: "not_available",
        resource_origin: None,
    })
}

fn load_stage_tool_resources(
    config_path: PathBuf,
) -> Result<BTreeMap<(String, String, String), StageToolResourceRow>> {
    let payload = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: StageToolResourcesConfig =
        toml::from_str(&payload).with_context(|| format!("parse {}", config_path.display()))?;
    Ok(config
        .rows
        .into_iter()
        .map(|row| ((row.domain.clone(), row.stage_id.clone(), row.tool_id.clone()), row))
        .collect())
}

fn load_science_threshold_rows(
    config_path: PathBuf,
    result_rows: &[MicroBenchmarkResultRow],
) -> Result<Vec<MicroBenchmarkScienceThresholdRow>> {
    let payload = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: ScientificAcceptanceThresholdsConfig =
        toml::from_str(&payload).with_context(|| format!("parse {}", config_path.display()))?;
    let stage_tools = result_rows.iter().fold(
        BTreeMap::<(String, String), BTreeSet<String>>::new(),
        |mut acc, row| {
            acc.entry((row.domain.clone(), row.stage_id.clone()))
                .or_default()
                .insert(row.tool_id.clone());
            acc
        },
    );
    let mut rows = config
        .rows
        .into_iter()
        .filter_map(|row| {
            stage_tools
                .get(&(row.domain.clone(), row.stage_id.clone()))
                .map(|tool_ids| science_threshold_row(row, tool_ids))
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.metric_id.cmp(&right.metric_id))
    });
    Ok(rows)
}

fn science_threshold_row(
    row: ScientificAcceptanceThresholdRow,
    tool_ids: &BTreeSet<String>,
) -> MicroBenchmarkScienceThresholdRow {
    MicroBenchmarkScienceThresholdRow {
        domain: row.domain,
        stage_id: row.stage_id,
        metric_id: row.metric_id,
        metric_name: row.metric_name,
        unit: row.unit,
        direction: scientific_acceptance_direction_label(row.direction).to_string(),
        tolerance_kind: scientific_acceptance_tolerance_kind_label(row.tolerance_kind).to_string(),
        tolerance_value: row.tolerance_value,
        pass_rule: scientific_acceptance_pass_rule_label(row.pass_rule).to_string(),
        insufficiency_behavior: scientific_acceptance_insufficiency_behavior_label(
            row.insufficiency_behavior,
        )
        .to_string(),
        required: row.required,
        declaration_origin: row.declaration_origin,
        covered_tool_ids: tool_ids.iter().cloned().collect(),
    }
}

fn ensure_micro_benchmark_report_contract(
    manifest: &MicroBenchmarkRunManifest,
    result_rows: &[MicroBenchmarkResultRow],
    report: &mut MicroBenchmarkReport,
) -> Result<()> {
    if report.micro_run_manifest_path != manifest.manifest_path {
        bail!("micro benchmark report manifest path drifted from the governed run manifest");
    }
    if report.result_row_count != result_rows.len() {
        bail!("micro benchmark report result_row_count drifted from the governed micro run");
    }
    if report.complete_row_count + report.unavailable_row_count != report.result_row_count {
        bail!(
            "micro benchmark report must account for every result row as complete or unavailable"
        );
    }
    if report.runtime_row_count != report.result_row_count {
        bail!("micro benchmark report runtime rows must cover every result row");
    }
    if report.memory_source_row_count != report.result_row_count {
        bail!("micro benchmark report memory-source rows must cover every result row");
    }
    if report.science_threshold_row_count == 0 {
        bail!("micro benchmark report must retain at least one science-threshold row");
    }
    if report.unavailable_row_count
        != result_rows
            .iter()
            .filter(|row| row.status != MicroBenchmarkExecutionStatus::Succeeded)
            .count()
    {
        bail!("micro benchmark report unavailable rows drifted from the governed run status slice");
    }
    let execution_status_total = report.execution_status_counts.values().sum::<usize>();
    if execution_status_total != report.result_row_count {
        bail!("micro benchmark report execution-status counts must sum to result_row_count");
    }
    if report.failed_rows.iter().any(|row| row.detail.trim().is_empty())
        || report.missing_rows.iter().any(|row| row.detail.trim().is_empty())
        || report.unavailable_rows.iter().any(|row| row.reason.trim().is_empty())
        || report.insufficient_data_rows.iter().any(|row| row.detail.trim().is_empty())
    {
        bail!("micro benchmark report rows must keep stable detail and reason fields");
    }
    report.passes_behavior_test = true;
    Ok(())
}

fn render_micro_benchmark_report_markdown(report: &MicroBenchmarkReport) -> String {
    let mut rendered = String::from("# Micro Benchmark Report\n\n");
    rendered.push_str(&format!(
        "- Result rows: {}\n- Complete rows: {}\n- Failed rows: {}\n- Missing rows: {}\n- Unavailable rows: {}\n- Insufficient-data rows: {}\n- Runtime rows: {}\n- Memory-source rows: {}\n- Science-threshold rows: {}\n\n",
        report.result_row_count,
        report.complete_row_count,
        report.failed_row_count,
        report.missing_row_count,
        report.unavailable_row_count,
        report.insufficient_data_row_count,
        report.runtime_row_count,
        report.memory_source_row_count,
        report.science_threshold_row_count
    ));

    rendered.push_str("## Complete\n\n");
    rendered.push_str("| Execution ID | Component | Domain | Stage | Tool | Metrics | Outputs | Logs | Reason |\n");
    rendered.push_str("| --- | --- | --- | --- | --- | ---: | ---: | ---: | --- |\n");
    for row in &report.complete_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.execution_id),
            sanitize_markdown_cell(&row.component_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            row.normalized_metric_count,
            row.output_count,
            row.log_count,
            sanitize_markdown_cell(&row.reason)
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Failed\n\n");
    rendered.push_str(
        "| Row ID | Source Surface | Component | Execution ID | Domain | Stage | Tool | Detail |\n",
    );
    rendered.push_str("| --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for row in &report.failed_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.row_id),
            sanitize_markdown_cell(&row.source_surface),
            sanitize_markdown_cell(row.component_id.as_deref().unwrap_or("")),
            sanitize_markdown_cell(row.execution_id.as_deref().unwrap_or("")),
            sanitize_markdown_cell(row.domain.as_deref().unwrap_or("")),
            sanitize_markdown_cell(row.stage_id.as_deref().unwrap_or("")),
            sanitize_markdown_cell(row.tool_id.as_deref().unwrap_or("")),
            sanitize_markdown_cell(&row.detail)
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Missing\n\n");
    rendered.push_str(
        "| Component | Execution ID | Domain | Stage | Tool | Source Report | Detail |\n",
    );
    rendered.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");
    for row in &report.missing_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.component_id),
            sanitize_markdown_cell(&row.execution_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            sanitize_markdown_cell(&row.source_report_path),
            sanitize_markdown_cell(&row.detail)
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Unavailable\n\n");
    rendered.push_str("| Execution ID | Component | Domain | Stage | Tool | Status | Reason |\n");
    rendered.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");
    for row in &report.unavailable_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.execution_id),
            sanitize_markdown_cell(&row.component_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            micro_execution_status_label(row.execution_status),
            sanitize_markdown_cell(&row.reason)
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Insufficient Data\n\n");
    rendered.push_str("| Execution ID | Component | Domain | Stage | Tool | Detail |\n");
    rendered.push_str("| --- | --- | --- | --- | --- | --- |\n");
    for row in &report.insufficient_data_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.execution_id),
            sanitize_markdown_cell(&row.component_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            sanitize_markdown_cell(&row.detail)
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Runtime\n\n");
    rendered.push_str("| Execution ID | Component | Domain | Stage | Tool | Status | Elapsed Seconds | Source |\n");
    rendered.push_str("| --- | --- | --- | --- | --- | --- | ---: | --- |\n");
    for row in &report.runtime_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.execution_id),
            sanitize_markdown_cell(&row.component_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            micro_execution_status_label(row.execution_status),
            format_optional_f64(row.elapsed_seconds),
            sanitize_markdown_cell(&row.runtime_source)
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Memory Sources\n\n");
    rendered.push_str("| Execution ID | Component | Domain | Stage | Tool | Status | Declared Memory MB | Declared CPU Threads | Observed Memory MB | Observed CPU Threads | Source |\n");
    rendered.push_str("| --- | --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | --- |\n");
    for row in &report.memory_source_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.execution_id),
            sanitize_markdown_cell(&row.component_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            micro_execution_status_label(row.execution_status),
            format_optional_f64(row.declared_memory_mb),
            format_optional_u32(row.declared_cpu_threads),
            format_optional_f64(row.observed_memory_mb),
            format_optional_u32(row.observed_cpu_threads),
            sanitize_markdown_cell(&row.memory_source)
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Science Thresholds\n\n");
    rendered.push_str("| Domain | Stage | Metric ID | Metric Name | Unit | Direction | Tolerance Kind | Tolerance | Pass Rule | Insufficiency Behavior | Required | Covered Tools |\n");
    rendered
        .push_str("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for row in &report.science_threshold_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.metric_id),
            sanitize_markdown_cell(&row.metric_name),
            sanitize_markdown_cell(row.unit.as_deref().unwrap_or("")),
            sanitize_markdown_cell(&row.direction),
            sanitize_markdown_cell(&row.tolerance_kind),
            format_threshold_value(row.tolerance_value),
            sanitize_markdown_cell(&row.pass_rule),
            sanitize_markdown_cell(&row.insufficiency_behavior),
            row.required.to_string(),
            sanitize_markdown_cell(&row.covered_tool_ids.join(", "))
        ));
    }

    rendered
}

fn micro_execution_status_label(status: MicroBenchmarkExecutionStatus) -> &'static str {
    match status {
        MicroBenchmarkExecutionStatus::Succeeded => "succeeded",
        MicroBenchmarkExecutionStatus::ContainerNeeded => "container_needed",
        MicroBenchmarkExecutionStatus::Unavailable => "unavailable",
    }
}

fn read_json_value(path: PathBuf) -> Result<Value> {
    let payload = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&payload).with_context(|| format!("parse {}", path.display()))
}

fn json_optional_f64(value: &Value, field: &str) -> Option<f64> {
    value.get(field).and_then(Value::as_f64)
}

fn json_optional_u32(value: &Value, field: &str) -> Option<u32> {
    value.get(field).and_then(Value::as_u64).and_then(|value| u32::try_from(value).ok())
}

fn format_optional_f64(value: Option<f64>) -> String {
    value.map_or_else(String::new, |value| format!("{value:.3}"))
}

fn format_optional_u32(value: Option<u32>) -> String {
    value.map_or_else(String::new, |value| value.to_string())
}

fn format_threshold_value(value: f64) -> String {
    format!("{value:.6}")
}

fn sanitize_markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', "<br>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn micro_benchmark_report_renders_governed_sections() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();

        fs::create_dir_all(repo_root.join("benchmarks/configs/local")).expect("create configs");
        fs::write(
            repo_root.join(SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH),
            r#"
schema_version = "bijux.bench.local_scientific_acceptance_thresholds.v1"

[[rows]]
domain = "fastq"
stage_id = "fastq.validate_reads"
metric_id = "validated_reads"
metric_name = "validated_reads"
direction = "range"
tolerance_kind = "relative_fraction"
tolerance_value = 0.02
pass_rule = "must_remain_within_reference_range"
insufficiency_behavior = "refuse_stage_comparison"
required = true
declaration_origin = "test_contract"
"#,
        )
        .expect("write thresholds");
        fs::write(
            repo_root.join(STAGE_TOOL_RESOURCES_PATH),
            r#"
schema_version = "bijux.bench.local_stage_tool_resources.v1"
classification_scope = "benchmark_ready_command_resources"

[[rows]]
domain = "fastq"
stage_id = "fastq.validate_reads"
tool_id = "fastqvalidator"
threads = 1
memory_gb = 1
walltime_minutes = 1
scratch_gb = 1
resource_origin = "test_resource"
"#,
        )
        .expect("write resources");

        let fastq_report_path = repo_root.join("runs/bench/micro/fastq/MICRO_FASTQ_SUMMARY.json");
        let adna_report_path =
            repo_root.join("runs/bench/micro/pipelines/adna/MICRO_ADNA_SUMMARY.json");
        let manifest_path = repo_root.join("runs/bench/micro/MICRO_BENCHMARK_RUN.json");
        let evidence_path =
            repo_root.join("runs/bench/local-smoke/fastq.validate_reads/report.json");
        let log_path = repo_root.join("runs/bench/micro/logs/MICRO_RUN.log");
        for path in
            [&fastq_report_path, &adna_report_path, &manifest_path, &evidence_path, &log_path]
        {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create parent");
            }
            fs::write(path, "{}").expect("write file");
        }

        let fastq_report = FastqMicroSmokeSubsetReport {
            schema_version: "bijux.bench.local_fastq_micro_smoke_subset.v1",
            output_path: "runs/bench/micro/fastq/MICRO_FASTQ_SUMMARY.json".to_string(),
            family_count: 1,
            local_smoke_count: 1,
            container_needed_count: 0,
            unavailable_count: 0,
            passes_behavior_test: true,
            rows: vec![FastqMicroSmokeFamilyRow {
                family_id: "fastq.validate_reads".to_string(),
                surface_label: "Validate Reads".to_string(),
                stage_ids: vec!["fastq.validate_reads".to_string()],
                representative_stage_id: "fastq.validate_reads".to_string(),
                representative_tool_id: "fastqvalidator".to_string(),
                registered_binary: "fastqvalidator".to_string(),
                smoke_tool_id: "fastqvalidator".to_string(),
                smoke_path_kind: "host_stage_smoke".to_string(),
                smoke_runtime: "host".to_string(),
                smoke_command: "bijux-dna bench local materialize-stage --stage-id fastq.validate_reads".to_string(),
                smoke_support_path: None,
                execution_status: super::super::local_fastq_micro_smoke_subset::FastqMicroSmokeExecutionStatus::LocalSmoke,
                reason: "governed local smoke".to_string(),
                evidence_path: Some("runs/bench/local-smoke/fastq.validate_reads/report.json".to_string()),
                evidence_format: Some("json".to_string()),
                parsed_schema_version: Some("bijux.fastq.validate.local_smoke.report.v1".to_string()),
            }],
        };
        let adna_report = AdnaMicroPipelineReport {
            schema_version: "bijux.bench.local_adna_micro_pipeline.v1",
            command: "bijux-dna bench local run-adna-micro-pipeline",
            output_path: "runs/bench/micro/pipelines/adna/MICRO_ADNA_SUMMARY.json".to_string(),
            pipeline_id: "adna-pseudohaploid-fastq-bam-vcf",
            sample_id: "adna-micro".to_string(),
            reference_fasta_path: "reference.fasta".to_string(),
            started_at: "2026-01-01T00:00:00Z".to_string(),
            finished_at: "2026-01-01T00:00:01Z".to_string(),
            elapsed_seconds: 1.0,
            stage_count: 1,
            handoff_count: 0,
            skipped_count: 1,
            passes_behavior_test: true,
            rows: vec![AdnaMicroPipelineRow {
                stage_id: "bam.contamination".to_string(),
                domain: "bam".to_string(),
                tool_id: "verifybamid2".to_string(),
                execution_mode: "skipped".to_string(),
                status: super::super::local_adna_micro_pipeline::AdnaMicroPipelineRowStatus::Skipped,
                reason: "synthetic aDNA micro execution does not claim panel-backed contamination evidence".to_string(),
                evidence_path: None,
                parsed_schema_version: None,
                consumed_inputs: BTreeMap::new(),
                outputs: BTreeMap::new(),
                metrics: BTreeMap::new(),
            }],
            handoffs: Vec::new(),
        };

        let empty_bam = BamMicroSmokeSubsetReport {
            schema_version: "bijux.bench.local_bam_micro_smoke_subset.v2",
            output_path: "runs/bench/micro/bam/MICRO_BAM_SUMMARY.json".to_string(),
            family_count: 0,
            local_smoke_count: 0,
            container_needed_count: 0,
            unavailable_count: 0,
            passes_behavior_test: true,
            rows: Vec::new(),
        };
        let empty_vcf = VcfMicroSmokeSubsetReport {
            schema_version: "bijux.bench.local_vcf_micro_smoke_subset.v1",
            output_path: "runs/bench/micro/vcf/MICRO_VCF_SUMMARY.json".to_string(),
            family_count: 0,
            local_smoke_count: 0,
            container_needed_count: 0,
            unavailable_count: 0,
            passes_behavior_test: true,
            rows: Vec::new(),
        };
        let empty_amplicon = AmpliconMicroPipelineReport {
            schema_version: "bijux.bench.local_amplicon_micro_pipeline.v1",
            command: "bijux-dna bench local run-amplicon-micro-pipeline",
            output_path: "runs/bench/micro/pipelines/amplicon/MICRO_AMPLICON_SUMMARY.json"
                .to_string(),
            pipeline_id: "amplicon-asv-otu-no-vcf",
            corpus_manifest_path: "corpus.toml".to_string(),
            truth_manifest_path: "truth.toml".to_string(),
            sample_count: 0,
            started_at: "2026-01-01T00:00:00Z".to_string(),
            finished_at: "2026-01-01T00:00:01Z".to_string(),
            elapsed_seconds: 1.0,
            stage_count: 0,
            handoff_count: 0,
            passes_behavior_test: true,
            rows: Vec::new(),
            handoffs: Vec::new(),
        };
        let empty_edna = EdnaMicroPipelineReport {
            schema_version: "bijux.bench.local_edna_micro_pipeline.v1",
            command: "bijux-dna bench local run-edna-micro-pipeline",
            output_path: "runs/bench/micro/pipelines/edna/MICRO_EDNA_SUMMARY.json".to_string(),
            pipeline_id: "edna-taxonomy-fastq",
            corpus_manifest_path: "corpus.toml".to_string(),
            taxonomy_database_manifest_path: "taxonomy.toml".to_string(),
            sample_count: 0,
            started_at: "2026-01-01T00:00:00Z".to_string(),
            finished_at: "2026-01-01T00:00:01Z".to_string(),
            elapsed_seconds: 1.0,
            stage_count: 0,
            handoff_count: 0,
            passes_behavior_test: true,
            rows: Vec::new(),
            handoffs: Vec::new(),
        };
        let empty_core = CoreGermlineMicroPipelineReport {
            schema_version: "bijux.bench.local_core_germline_micro_pipeline.v1",
            command: "bijux-dna bench local run-core-germline-micro-pipeline",
            output_path: "runs/bench/micro/pipelines/core-germline/MICRO_PIPELINE_SUMMARY.json"
                .to_string(),
            pipeline_id: "core-germline-fastq-bam-vcf",
            sample_id: "sample".to_string(),
            reference_fasta_path: "reference.fasta".to_string(),
            started_at: "2026-01-01T00:00:00Z".to_string(),
            finished_at: "2026-01-01T00:00:01Z".to_string(),
            elapsed_seconds: 1.0,
            stage_count: 0,
            handoff_count: 0,
            passes_behavior_test: true,
            rows: Vec::new(),
            handoffs: Vec::new(),
        };
        let empty_real = RealSmokeCoreSubsetReport {
            schema_version: "bijux.bench.local_real_smoke_core_subset.v1",
            output_path: "runs/bench/micro/core/REAL_SMOKE_CORE_SUMMARY.json".to_string(),
            execution_count: 0,
            stage_execution_count: 0,
            pipeline_bridge_count: 0,
            domain_counts: BTreeMap::new(),
            passes_behavior_test: true,
            rows: Vec::new(),
        };

        let manifest = MicroBenchmarkRunManifest {
            schema_version: "bijux.bench.local_micro_benchmark_run.v1",
            manifest_path: "runs/bench/micro/MICRO_BENCHMARK_RUN.json".to_string(),
            run_root: "runs/bench/micro".to_string(),
            run_id: "micro-benchmark-test".to_string(),
            repo_revision: "0123456789abcdef0123456789abcdef01234567".to_string(),
            worktree_dirty: false,
            created_at_unix: 1,
            command: "bijux-dna bench run-micro",
            component_reports: Vec::new(),
            result_rows_path: "runs/bench/micro/results/MICRO_RESULT_ROWS.json".to_string(),
            output_rows_path: "runs/bench/micro/outputs/MICRO_OUTPUT_ROWS.json".to_string(),
            log_rows_path: "runs/bench/micro/logs/MICRO_LOG_ROWS.json".to_string(),
            normalized_metrics_path:
                "runs/bench/micro/normalized-metrics/MICRO_NORMALIZED_METRICS.json".to_string(),
            result_row_count: 2,
            output_row_count: 1,
            log_row_count: 1,
            normalized_metric_row_count: 1,
            passes_behavior_test: true,
        };
        let result_rows = vec![
            MicroBenchmarkResultRow {
                execution_id: "fastq.validate_reads".to_string(),
                component_id: "fastq_micro_smoke_subset".to_string(),
                result_kind: super::super::local_micro_benchmark_run::MicroBenchmarkResultKind::FamilyRepresentative,
                domain: "fastq".to_string(),
                bridge_source_domain: None,
                bridge_target_domain: None,
                stage_id: "fastq.validate_reads".to_string(),
                tool_id: "fastqvalidator".to_string(),
                status: MicroBenchmarkExecutionStatus::Succeeded,
                reason: "governed local smoke".to_string(),
                command: Some("bijux-dna bench local materialize-stage --stage-id fastq.validate_reads".to_string()),
                source_report_path: fastq_report.output_path.clone(),
                evidence_path: Some("runs/bench/local-smoke/fastq.validate_reads/report.json".to_string()),
                stage_result_manifest_path: None,
                normalized_metric_count: 1,
                output_count: 1,
                log_count: 0,
            },
            MicroBenchmarkResultRow {
                execution_id: "bam.contamination".to_string(),
                component_id: "adna_micro_pipeline".to_string(),
                result_kind: super::super::local_micro_benchmark_run::MicroBenchmarkResultKind::Stage,
                domain: "bam".to_string(),
                bridge_source_domain: None,
                bridge_target_domain: None,
                stage_id: "bam.contamination".to_string(),
                tool_id: "verifybamid2".to_string(),
                status: MicroBenchmarkExecutionStatus::Unavailable,
                reason: "synthetic aDNA micro execution does not claim panel-backed contamination evidence".to_string(),
                command: Some("bijux-dna bench local run-adna-micro-pipeline".to_string()),
                source_report_path: adna_report.output_path.clone(),
                evidence_path: None,
                stage_result_manifest_path: None,
                normalized_metric_count: 0,
                output_count: 0,
                log_count: 0,
            },
        ];
        let output_rows = vec![MicroBenchmarkOutputRow {
            execution_id: "fastq.validate_reads".to_string(),
            component_id: "fastq_micro_smoke_subset".to_string(),
            artifact_id: "evidence".to_string(),
            role: "family_evidence".to_string(),
            path: "runs/bench/local-smoke/fastq.validate_reads/report.json".to_string(),
            exists: true,
            source: "fastq_micro_smoke_subset".to_string(),
        }];
        let log_rows = vec![MicroBenchmarkLogRow {
            execution_id: "micro.run".to_string(),
            component_id: "micro_benchmark_run".to_string(),
            role: "run_log".to_string(),
            path: "runs/bench/micro/logs/MICRO_RUN.log".to_string(),
            exists: true,
            source: "micro_benchmark_run".to_string(),
        }];

        let report = render_micro_benchmark_report(
            repo_root,
            &manifest,
            &result_rows,
            &output_rows,
            &log_rows,
            PathBuf::from(DEFAULT_MICRO_BENCHMARK_REPORT_MARKDOWN_PATH),
            PathBuf::from(DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH),
            MicroBenchmarkSourceReports {
                real_smoke: &empty_real,
                fastq: &fastq_report,
                bam: &empty_bam,
                vcf: &empty_vcf,
                amplicon: &empty_amplicon,
                adna: &adna_report,
                edna: &empty_edna,
                core_germline: &empty_core,
            },
        )
        .expect("render report");

        assert_eq!(report.complete_row_count, 1);
        assert_eq!(report.unavailable_row_count, 1);
        assert_eq!(report.failed_row_count, 0);
        assert_eq!(report.missing_row_count, 0);
        assert_eq!(report.runtime_row_count, 2);
        assert_eq!(report.memory_source_row_count, 2);
        assert_eq!(report.science_threshold_row_count, 1);
        assert!(report.passes_behavior_test);

        let markdown =
            fs::read_to_string(repo_root.join(DEFAULT_MICRO_BENCHMARK_REPORT_MARKDOWN_PATH))
                .expect("read markdown");
        assert!(markdown.contains("# Micro Benchmark Report"));
        assert!(markdown.contains("## Complete"));
        assert!(markdown.contains("## Failed"));
        assert!(markdown.contains("## Missing"));
        assert!(markdown.contains("## Unavailable"));
        assert!(markdown.contains("## Insufficient Data"));
        assert!(markdown.contains("## Runtime"));
        assert!(markdown.contains("## Memory Sources"));
        assert!(markdown.contains("## Science Thresholds"));
    }

    #[test]
    fn insufficient_marker_detector_matches_nested_json() {
        let payload = json!({
            "summary": {
                "status": "insufficient_data"
            }
        });
        assert!(json_contains_insufficient_marker(&payload));
    }
}
