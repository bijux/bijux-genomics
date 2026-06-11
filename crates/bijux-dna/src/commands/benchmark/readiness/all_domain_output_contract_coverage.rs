use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_stage_tool_matrix::{
    collect_all_domain_active_stage_tool_matrix_rows, AllDomainActiveStageToolMatrixRow,
};
use super::all_domain_output_declarations::{
    collect_all_domain_output_declaration_rows, AllDomainOutputDeclarationRow,
    AllDomainOutputDeclarationStatus,
};
use super::bam_adapter_output_contract::{
    collect_bam_adapter_output_contract_rows, BamAdapterOutputContractRow,
    BamAdapterOutputContractStatus,
};
use super::fastq_adapter_output_contract::{
    collect_fastq_adapter_output_contract_rows, FastqAdapterOutputContractRow,
    FastqAdapterOutputContractStatus,
};
use super::vcf_adapter_output_coverage::{
    collect_vcf_adapter_output_coverage_rows, VcfAdapterOutputCoverageRow,
    VcfAdapterOutputCoverageStatus,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_PATH: &str =
    "benchmarks/readiness/all-domains/output-contract-coverage.tsv";
const ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_output_contract_coverage.v1";
const COVERAGE_STATUS_COVERED: &str = "covered";
const COVERAGE_STATUS_MISSING_OUTPUT_DECLARATION: &str = "missing_output_declaration";
const COVERAGE_STATUS_MISSING_SOURCE_CONTRACT: &str = "missing_source_contract";
const COVERAGE_STATUS_INCOMPLETE_SOURCE_CONTRACT: &str = "incomplete_source_contract";
const COVERAGE_STATUS_OUTPUT_CONTRACT_MISMATCH: &str = "output_contract_mismatch";
const INDEX_STATUS_COVERED: &str = "covered";
const INDEX_STATUS_NOT_APPLICABLE: &str = "not_applicable";
const INDEX_STATUS_MISMATCH: &str = "mismatch";
const OUTPUT_STATUS_COMPLETE: &str = "complete";
const SOURCE_STATUS_COMPLETE: &str = "complete";
const PROOF_SOURCE_FASTQ: &str = "fastq_output_contract";
const PROOF_SOURCE_BAM: &str = "bam_output_contract";
const PROOF_SOURCE_VCF: &str = "vcf_output_contract";
const NO_VALUE: &str = "none";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CoverageKey {
    domain: String,
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    asset_profile_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SourceKey {
    domain: String,
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone)]
struct SourceProofRow {
    proof_source: String,
    source_contract_status: String,
    raw_output_ids: Vec<String>,
    normalized_metric_ids: Vec<String>,
    stdout: String,
    stderr: String,
    manifest: String,
    index_output_ids: Vec<String>,
    requires_exact_logs_and_manifest: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainOutputContractCoverageRow {
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) proof_source: String,
    pub(crate) source_contract_status: String,
    pub(crate) output_declaration_status: String,
    pub(crate) raw_output_ids: Vec<String>,
    pub(crate) normalized_metric_ids: Vec<String>,
    pub(crate) log_declarations: Vec<String>,
    pub(crate) manifest: String,
    pub(crate) index_output_ids: Vec<String>,
    pub(crate) raw_outputs_declared: bool,
    pub(crate) normalized_metrics_declared: bool,
    pub(crate) logs_declared: bool,
    pub(crate) manifest_declared: bool,
    pub(crate) index_coverage_status: String,
    pub(crate) coverage_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainOutputContractCoverageReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) result_id_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) output_declaration_binding_count: usize,
    pub(crate) source_proof_binding_count: usize,
    pub(crate) covered_row_count: usize,
    pub(crate) missing_row_count: usize,
    pub(crate) coverage_percent: f64,
    pub(crate) raw_output_declared_row_count: usize,
    pub(crate) normalized_metrics_declared_row_count: usize,
    pub(crate) logs_declared_row_count: usize,
    pub(crate) manifest_declared_row_count: usize,
    pub(crate) index_required_row_count: usize,
    pub(crate) index_declared_row_count: usize,
    pub(crate) index_not_applicable_row_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) proof_source_counts: BTreeMap<String, usize>,
    pub(crate) index_coverage_counts: BTreeMap<String, usize>,
    pub(crate) coverage_status_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<AllDomainOutputContractCoverageRow>,
    pub(crate) violations: Vec<AllDomainOutputContractCoverageRow>,
}

pub(crate) fn run_render_all_domain_output_contract_coverage(
    args: &parse::BenchReadinessRenderAllDomainOutputContractCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_output_contract_coverage(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_output_contract_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainOutputContractCoverageReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_all_domain_output_contract_coverage_report(repo_root, &output_path)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_all_domain_output_contract_coverage_tsv(&report.rows))
        .with_context(|| format!("write {}", output_path.display()))?;
    if !report.ok {
        return Err(anyhow!("all-domain active rows must keep complete governed output contracts"));
    }
    Ok(report)
}

fn build_all_domain_output_contract_coverage_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<AllDomainOutputContractCoverageReport> {
    let active_rows = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?;
    let output_rows = collect_all_domain_output_declaration_rows(repo_root)?;
    let source_rows = collect_source_proof_rows(repo_root)?;

    let active_keys = active_rows.iter().map(coverage_key_from_active_row).collect::<BTreeSet<_>>();
    let output_by_key = output_rows
        .into_iter()
        .map(|row| (coverage_key_from_output_row(&row), row))
        .collect::<BTreeMap<_, _>>();
    let output_keys = output_by_key.keys().cloned().collect::<BTreeSet<_>>();

    let mut source_proof_binding_keys = BTreeSet::<CoverageKey>::new();
    let mut rows = Vec::with_capacity(active_rows.len());
    for active_row in &active_rows {
        let output_row = output_by_key.get(&coverage_key_from_active_row(active_row));
        let source_row = source_rows.get(&source_key_from_active_row(active_row));
        if source_row.is_some() {
            source_proof_binding_keys.insert(coverage_key_from_active_row(active_row));
        }
        rows.push(render_row(active_row, output_row, source_row));
    }
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });

    let row_count = rows.len();
    let covered_row_count =
        rows.iter().filter(|row| row.coverage_status == COVERAGE_STATUS_COVERED).count();
    let missing_row_count = row_count.saturating_sub(covered_row_count);
    let coverage_percent =
        if row_count == 0 { 0.0 } else { covered_row_count as f64 * 100.0 / row_count as f64 };
    let result_id_count = rows
        .iter()
        .map(|row| row.result_id.as_str())
        .filter(|result_id| *result_id != NO_VALUE)
        .collect::<BTreeSet<_>>()
        .len();
    let stage_count = rows
        .iter()
        .map(|row| (row.domain.as_str(), row.stage_id.as_str()))
        .collect::<BTreeSet<_>>()
        .len();
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let raw_output_declared_row_count = rows.iter().filter(|row| row.raw_outputs_declared).count();
    let normalized_metrics_declared_row_count =
        rows.iter().filter(|row| row.normalized_metrics_declared).count();
    let logs_declared_row_count = rows.iter().filter(|row| row.logs_declared).count();
    let manifest_declared_row_count = rows.iter().filter(|row| row.manifest_declared).count();
    let index_required_row_count =
        rows.iter().filter(|row| row.index_coverage_status != INDEX_STATUS_NOT_APPLICABLE).count();
    let index_declared_row_count =
        rows.iter().filter(|row| row.index_coverage_status == INDEX_STATUS_COVERED).count();
    let index_not_applicable_row_count =
        rows.iter().filter(|row| row.index_coverage_status == INDEX_STATUS_NOT_APPLICABLE).count();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut proof_source_counts = BTreeMap::<String, usize>::new();
    let mut index_coverage_counts = BTreeMap::<String, usize>::new();
    let mut coverage_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *proof_source_counts.entry(row.proof_source.clone()).or_default() += 1;
        *index_coverage_counts.entry(row.index_coverage_status.clone()).or_default() += 1;
        *coverage_status_counts.entry(row.coverage_status.clone()).or_default() += 1;
    }
    let violations = rows
        .iter()
        .filter(|row| row.coverage_status != COVERAGE_STATUS_COVERED)
        .cloned()
        .collect::<Vec<_>>();

    let report = AllDomainOutputContractCoverageReport {
        schema_version: ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        row_count,
        result_id_count,
        stage_count,
        tool_count,
        output_declaration_binding_count: output_keys.len(),
        source_proof_binding_count: source_proof_binding_keys.len(),
        covered_row_count,
        missing_row_count,
        coverage_percent,
        raw_output_declared_row_count,
        normalized_metrics_declared_row_count,
        logs_declared_row_count,
        manifest_declared_row_count,
        index_required_row_count,
        index_declared_row_count,
        index_not_applicable_row_count,
        domain_counts,
        proof_source_counts,
        index_coverage_counts,
        coverage_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    };
    ensure_all_domain_output_contract_coverage_contract(
        &active_rows,
        &active_keys,
        &output_keys,
        &source_proof_binding_keys,
        &report,
    )?;
    Ok(report)
}

fn render_row(
    active_row: &AllDomainActiveStageToolMatrixRow,
    output_row: Option<&AllDomainOutputDeclarationRow>,
    source_row: Option<&SourceProofRow>,
) -> AllDomainOutputContractCoverageRow {
    match (output_row, source_row) {
        (None, None) => missing_row(
            active_row,
            NO_VALUE,
            NO_VALUE,
            NO_VALUE,
            COVERAGE_STATUS_MISSING_OUTPUT_DECLARATION,
            format!(
                "active row `{}` / `{}` / `{}` is missing both all-domain output declarations and a domain-owned source contract",
                active_row.domain, active_row.stage_id, active_row.tool_id
            ),
        ),
        (None, Some(source_row)) => missing_row(
            active_row,
            NO_VALUE,
            &source_row.proof_source,
            &source_row.source_contract_status,
            COVERAGE_STATUS_MISSING_OUTPUT_DECLARATION,
            format!(
                "active row `{}` / `{}` / `{}` is missing an all-domain output declaration row in the governed `{}` surface",
                active_row.domain, active_row.stage_id, active_row.tool_id, source_row.proof_source
            ),
        ),
        (Some(output_row), None) => missing_row(
            active_row,
            &output_row.result_id,
            NO_VALUE,
            NO_VALUE,
            COVERAGE_STATUS_MISSING_SOURCE_CONTRACT,
            format!(
                "active row `{}` / `{}` / `{}` keeps an all-domain output declaration but is missing a domain-owned source output contract",
                active_row.domain, active_row.stage_id, active_row.tool_id
            ),
        ),
        (Some(output_row), Some(source_row)) => {
            let raw_outputs_declared =
                string_set(&output_row.raw_outputs) == string_set(&source_row.raw_output_ids)
                    && !source_row.raw_output_ids.is_empty();
            let normalized_metrics_declared =
                string_set(&output_row.normalized_metrics)
                    == string_set(&source_row.normalized_metric_ids)
                    && !source_row.normalized_metric_ids.is_empty();
            let logs_declared = if source_row.requires_exact_logs_and_manifest {
                output_row.logs.iter().any(|entry| entry == &format!("stdout={}", source_row.stdout))
                    && output_row
                        .logs
                        .iter()
                        .any(|entry| entry == &format!("stderr={}", source_row.stderr))
                    && output_row.logs.len() == 2
            } else {
                output_row.logs.len() == 2
                    && output_row.logs.iter().any(|entry| entry.starts_with("stdout="))
                    && output_row.logs.iter().any(|entry| entry.starts_with("stderr="))
                    && !source_row.stdout.is_empty()
                    && !source_row.stderr.is_empty()
            };
            let manifest_declared = if source_row.requires_exact_logs_and_manifest {
                output_row.manifest == source_row.manifest && !source_row.manifest.is_empty()
            } else {
                !output_row.manifest.trim().is_empty() && !source_row.manifest.is_empty()
            };
            let index_coverage_status = if source_row.index_output_ids.is_empty() {
                if output_row.index_outputs.is_empty() {
                    INDEX_STATUS_NOT_APPLICABLE.to_string()
                } else {
                    INDEX_STATUS_MISMATCH.to_string()
                }
            } else if string_set(&output_row.index_outputs) == string_set(&source_row.index_output_ids) {
                INDEX_STATUS_COVERED.to_string()
            } else {
                INDEX_STATUS_MISMATCH.to_string()
            };
            let output_declaration_status = output_status_label(output_row.status).to_string();

            let coverage_status =
                if source_row.source_contract_status != SOURCE_STATUS_COMPLETE {
                    COVERAGE_STATUS_INCOMPLETE_SOURCE_CONTRACT.to_string()
                } else if output_row.status != AllDomainOutputDeclarationStatus::Complete {
                    COVERAGE_STATUS_OUTPUT_CONTRACT_MISMATCH.to_string()
                } else if raw_outputs_declared
                    && normalized_metrics_declared
                    && logs_declared
                    && manifest_declared
                    && index_coverage_status != INDEX_STATUS_MISMATCH
                {
                    COVERAGE_STATUS_COVERED.to_string()
                } else {
                    COVERAGE_STATUS_OUTPUT_CONTRACT_MISMATCH.to_string()
                };

            let reason = if coverage_status == COVERAGE_STATUS_COVERED {
                match index_coverage_status.as_str() {
                    INDEX_STATUS_COVERED => format!(
                        "active row `{}` / `{}` / `{}` keeps governed raw outputs, normalized metrics, logs, manifest, and required index declarations through `{}`",
                        active_row.domain, active_row.stage_id, active_row.tool_id, source_row.proof_source
                    ),
                    _ => format!(
                        "active row `{}` / `{}` / `{}` keeps governed raw outputs, normalized metrics, logs, and manifest declarations through `{}` with no index requirement",
                        active_row.domain, active_row.stage_id, active_row.tool_id, source_row.proof_source
                    ),
                }
            } else {
                let mut missing = Vec::<&str>::new();
                if source_row.source_contract_status != SOURCE_STATUS_COMPLETE {
                    missing.push("source_contract_status");
                }
                if output_row.status != AllDomainOutputDeclarationStatus::Complete {
                    missing.push("output_declaration_status");
                }
                if !raw_outputs_declared {
                    missing.push("raw_outputs");
                }
                if !normalized_metrics_declared {
                    missing.push("normalized_metrics");
                }
                if !logs_declared {
                    missing.push("logs");
                }
                if !manifest_declared {
                    missing.push("manifest");
                }
                if index_coverage_status == INDEX_STATUS_MISMATCH {
                    missing.push("index_outputs");
                }
                format!(
                    "active row `{}` / `{}` / `{}` is missing governed output-contract coverage for {}",
                    active_row.domain,
                    active_row.stage_id,
                    active_row.tool_id,
                    missing.join(", ")
                )
            };

            AllDomainOutputContractCoverageRow {
                result_id: output_row.result_id.clone(),
                domain: active_row.domain.clone(),
                stage_id: active_row.stage_id.clone(),
                tool_id: active_row.tool_id.clone(),
                corpus_id: active_row.corpus_id.clone(),
                asset_profile_id: active_row.asset_profile_id.clone(),
                adapter_id: active_row.adapter_id.clone(),
                proof_source: source_row.proof_source.clone(),
                source_contract_status: source_row.source_contract_status.clone(),
                output_declaration_status,
                raw_output_ids: output_row.raw_outputs.clone(),
                normalized_metric_ids: output_row.normalized_metrics.clone(),
                log_declarations: output_row.logs.clone(),
                manifest: output_row.manifest.clone(),
                index_output_ids: output_row.index_outputs.clone(),
                raw_outputs_declared,
                normalized_metrics_declared,
                logs_declared,
                manifest_declared,
                index_coverage_status,
                coverage_status,
                reason,
            }
        }
    }
}

fn missing_row(
    active_row: &AllDomainActiveStageToolMatrixRow,
    result_id: &str,
    proof_source: &str,
    source_contract_status: &str,
    coverage_status: &str,
    reason: String,
) -> AllDomainOutputContractCoverageRow {
    AllDomainOutputContractCoverageRow {
        result_id: result_id.to_string(),
        domain: active_row.domain.clone(),
        stage_id: active_row.stage_id.clone(),
        tool_id: active_row.tool_id.clone(),
        corpus_id: active_row.corpus_id.clone(),
        asset_profile_id: active_row.asset_profile_id.clone(),
        adapter_id: active_row.adapter_id.clone(),
        proof_source: proof_source.to_string(),
        source_contract_status: source_contract_status.to_string(),
        output_declaration_status: NO_VALUE.to_string(),
        raw_output_ids: Vec::new(),
        normalized_metric_ids: Vec::new(),
        log_declarations: Vec::new(),
        manifest: NO_VALUE.to_string(),
        index_output_ids: Vec::new(),
        raw_outputs_declared: false,
        normalized_metrics_declared: false,
        logs_declared: false,
        manifest_declared: false,
        index_coverage_status: INDEX_STATUS_MISMATCH.to_string(),
        coverage_status: coverage_status.to_string(),
        reason,
    }
}

fn collect_source_proof_rows(repo_root: &Path) -> Result<BTreeMap<SourceKey, SourceProofRow>> {
    let mut rows = BTreeMap::<SourceKey, SourceProofRow>::new();

    for row in collect_fastq_adapter_output_contract_rows(repo_root)? {
        insert_source_proof_row(
            &mut rows,
            SourceKey {
                domain: "fastq".to_string(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
            },
            render_fastq_source_proof_row(row),
        )?;
    }
    for row in collect_bam_adapter_output_contract_rows(repo_root)? {
        insert_source_proof_row(
            &mut rows,
            SourceKey {
                domain: "bam".to_string(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
            },
            render_bam_source_proof_row(row),
        )?;
    }
    for row in collect_vcf_adapter_output_coverage_rows(repo_root)? {
        if row.benchmark_status != "benchmark_ready" {
            continue;
        }
        insert_source_proof_row(
            &mut rows,
            SourceKey {
                domain: "vcf".to_string(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
            },
            render_vcf_source_proof_row(row),
        )?;
    }

    Ok(rows)
}

fn insert_source_proof_row(
    rows: &mut BTreeMap<SourceKey, SourceProofRow>,
    key: SourceKey,
    row: SourceProofRow,
) -> Result<()> {
    if rows.insert(key.clone(), row).is_some() {
        return Err(anyhow!(
            "all-domain output-contract coverage encountered duplicate source proof rows for `{}` / `{}` / `{}`",
            key.domain,
            key.stage_id,
            key.tool_id
        ));
    }
    Ok(())
}

fn render_fastq_source_proof_row(row: FastqAdapterOutputContractRow) -> SourceProofRow {
    SourceProofRow {
        proof_source: PROOF_SOURCE_FASTQ.to_string(),
        source_contract_status: fastq_source_status_label(row.output_contract_status).to_string(),
        raw_output_ids: row.raw_output_artifact_ids,
        normalized_metric_ids: row.normalized_metrics_output_id.into_iter().collect(),
        stdout: row.stdout_path_template.unwrap_or_default(),
        stderr: row.stderr_path_template.unwrap_or_default(),
        manifest: row.stage_result_manifest_path_template.unwrap_or_default(),
        index_output_ids: Vec::new(),
        requires_exact_logs_and_manifest: false,
    }
}

fn render_bam_source_proof_row(row: BamAdapterOutputContractRow) -> SourceProofRow {
    SourceProofRow {
        proof_source: PROOF_SOURCE_BAM.to_string(),
        source_contract_status: bam_source_status_label(row.output_contract_status).to_string(),
        raw_output_ids: row.raw_output_artifact_ids,
        normalized_metric_ids: row.normalized_metrics_output_id.into_iter().collect(),
        stdout: row.stdout_path_template.unwrap_or_default(),
        stderr: row.stderr_path_template.unwrap_or_default(),
        manifest: row.stage_result_manifest_path_template.unwrap_or_default(),
        index_output_ids: Vec::new(),
        requires_exact_logs_and_manifest: false,
    }
}

fn render_vcf_source_proof_row(row: VcfAdapterOutputCoverageRow) -> SourceProofRow {
    SourceProofRow {
        proof_source: PROOF_SOURCE_VCF.to_string(),
        source_contract_status: vcf_source_status_label(row.status).to_string(),
        raw_output_ids: row.raw_outputs.into_iter().map(|entry| artifact_id(&entry)).collect(),
        normalized_metric_ids: row
            .normalized_metrics
            .into_iter()
            .map(|entry| artifact_id(&entry))
            .collect(),
        stdout: row.stdout,
        stderr: row.stderr,
        manifest: row.manifest,
        index_output_ids: row.index_outputs.into_iter().map(|entry| artifact_id(&entry)).collect(),
        requires_exact_logs_and_manifest: false,
    }
}

fn ensure_all_domain_output_contract_coverage_contract(
    active_rows: &[AllDomainActiveStageToolMatrixRow],
    active_keys: &BTreeSet<CoverageKey>,
    output_keys: &BTreeSet<CoverageKey>,
    source_proof_binding_keys: &BTreeSet<CoverageKey>,
    report: &AllDomainOutputContractCoverageReport,
) -> Result<()> {
    if report.row_count != active_rows.len() || report.row_count != report.rows.len() {
        return Err(anyhow!(
            "all-domain output-contract coverage must keep exactly one row per active binding"
        ));
    }
    if report.covered_row_count + report.missing_row_count != report.row_count {
        return Err(anyhow!(
            "all-domain output-contract coverage drifted from its covered/missing counts"
        ));
    }
    if *active_keys != *output_keys {
        let missing_keys =
            active_keys.difference(output_keys).map(format_coverage_key).collect::<Vec<_>>();
        let extra_keys =
            output_keys.difference(active_keys).map(format_coverage_key).collect::<Vec<_>>();
        return Err(anyhow!(
            "all-domain output-contract coverage drifted from active scope, missing output keys: [{}], extra output keys: [{}]",
            missing_keys.join(", "),
            extra_keys.join(", ")
        ));
    }
    if *active_keys != *source_proof_binding_keys {
        let missing_keys = active_keys
            .difference(source_proof_binding_keys)
            .map(format_coverage_key)
            .collect::<Vec<_>>();
        return Err(anyhow!(
            "all-domain output-contract coverage is missing source proof for active bindings: [{}]",
            missing_keys.join(", ")
        ));
    }

    let reported_keys =
        report.rows.iter().map(coverage_key_from_report_row).collect::<BTreeSet<_>>();
    if reported_keys != *active_keys {
        return Err(anyhow!(
            "all-domain output-contract coverage TSV must match the governed active coverage-key set"
        ));
    }
    if report.result_id_count != report.row_count {
        return Err(anyhow!(
            "all-domain output-contract coverage must keep exactly one unique result_id per active binding"
        ));
    }
    if report.raw_output_declared_row_count != report.row_count
        || report.normalized_metrics_declared_row_count != report.row_count
        || report.logs_declared_row_count != report.row_count
        || report.manifest_declared_row_count != report.row_count
    {
        return Err(anyhow!(
            "all-domain output-contract coverage must keep raw outputs, normalized metrics, logs, and manifest declarations for every active binding"
        ));
    }
    if report.index_declared_row_count + report.index_not_applicable_row_count != report.row_count {
        return Err(anyhow!(
            "all-domain output-contract coverage drifted from its index applicability counts"
        ));
    }
    if report.violation_count != report.violations.len() {
        return Err(anyhow!("all-domain output-contract coverage drifted from its violation set"));
    }
    if report.ok && report.violation_count != 0 {
        return Err(anyhow!(
            "all-domain output-contract coverage cannot be ok while violations remain"
        ));
    }
    if !report.ok && report.violation_count == 0 {
        return Err(anyhow!(
            "all-domain output-contract coverage must keep explicit violations when failing"
        ));
    }
    if report.ok
        && (report.covered_row_count != report.row_count || report.coverage_percent != 100.0)
    {
        return Err(anyhow!(
            "all-domain output-contract coverage must reach 100% when no violations remain"
        ));
    }

    for row in &report.rows {
        if row.result_id.trim().is_empty()
            || row.domain.trim().is_empty()
            || row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.corpus_id.trim().is_empty()
            || row.asset_profile_id.trim().is_empty()
            || row.adapter_id.trim().is_empty()
            || row.proof_source.trim().is_empty()
            || row.source_contract_status.trim().is_empty()
            || row.output_declaration_status.trim().is_empty()
            || row.coverage_status.trim().is_empty()
            || row.index_coverage_status.trim().is_empty()
        {
            return Err(anyhow!(
                "all-domain output-contract coverage row `{}` / `{}` / `{}` is missing a required field",
                row.domain,
                row.stage_id,
                row.tool_id
            ));
        }
        if row.coverage_status == COVERAGE_STATUS_COVERED {
            if !row.raw_outputs_declared
                || !row.normalized_metrics_declared
                || !row.logs_declared
                || !row.manifest_declared
                || row.index_coverage_status == INDEX_STATUS_MISMATCH
                || row.source_contract_status != SOURCE_STATUS_COMPLETE
                || row.output_declaration_status != OUTPUT_STATUS_COMPLETE
            {
                return Err(anyhow!(
                    "all-domain output-contract coverage row `{}` / `{}` / `{}` is missing covered proof detail",
                    row.domain,
                    row.stage_id,
                    row.tool_id
                ));
            }
        }
    }

    Ok(())
}

fn render_all_domain_output_contract_coverage_tsv(
    rows: &[AllDomainOutputContractCoverageRow],
) -> String {
    let mut rendered = String::from(
        "result_id\tdomain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tproof_source\tsource_contract_status\toutput_declaration_status\traw_output_ids\tnormalized_metric_ids\tlog_declarations\tmanifest\tindex_output_ids\traw_outputs_declared\tnormalized_metrics_declared\tlogs_declared\tmanifest_declared\tindex_coverage_status\tcoverage_status\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.result_id),
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.corpus_id),
            sanitize_tsv(&row.asset_profile_id),
            sanitize_tsv(&row.adapter_id),
            sanitize_tsv(&row.proof_source),
            sanitize_tsv(&row.source_contract_status),
            sanitize_tsv(&row.output_declaration_status),
            sanitize_tsv(&row.raw_output_ids.join(",")),
            sanitize_tsv(&row.normalized_metric_ids.join(",")),
            sanitize_tsv(&row.log_declarations.join(",")),
            sanitize_tsv(&row.manifest),
            sanitize_tsv(&row.index_output_ids.join(",")),
            row.raw_outputs_declared,
            row.normalized_metrics_declared,
            row.logs_declared,
            row.manifest_declared,
            sanitize_tsv(&row.index_coverage_status),
            sanitize_tsv(&row.coverage_status),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn coverage_key_from_active_row(row: &AllDomainActiveStageToolMatrixRow) -> CoverageKey {
    CoverageKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn coverage_key_from_output_row(row: &AllDomainOutputDeclarationRow) -> CoverageKey {
    CoverageKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn coverage_key_from_report_row(row: &AllDomainOutputContractCoverageRow) -> CoverageKey {
    CoverageKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn source_key_from_active_row(row: &AllDomainActiveStageToolMatrixRow) -> SourceKey {
    SourceKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
    }
}

fn fastq_source_status_label(status: FastqAdapterOutputContractStatus) -> &'static str {
    match status {
        FastqAdapterOutputContractStatus::Complete => "complete",
        FastqAdapterOutputContractStatus::Incomplete => "incomplete",
        FastqAdapterOutputContractStatus::MissingAdapter => "missing_adapter",
    }
}

fn bam_source_status_label(status: BamAdapterOutputContractStatus) -> &'static str {
    match status {
        BamAdapterOutputContractStatus::Complete => "complete",
        BamAdapterOutputContractStatus::Incomplete => "incomplete",
        BamAdapterOutputContractStatus::MissingAdapter => "missing_adapter",
    }
}

fn vcf_source_status_label(status: VcfAdapterOutputCoverageStatus) -> &'static str {
    match status {
        VcfAdapterOutputCoverageStatus::Complete => "complete",
        VcfAdapterOutputCoverageStatus::Incomplete => "incomplete",
    }
}

fn output_status_label(status: AllDomainOutputDeclarationStatus) -> &'static str {
    match status {
        AllDomainOutputDeclarationStatus::Complete => "complete",
        AllDomainOutputDeclarationStatus::Incomplete => "incomplete",
        AllDomainOutputDeclarationStatus::MissingAdapter => "missing_adapter",
    }
}

fn format_coverage_key(key: &CoverageKey) -> String {
    format!(
        "{}/{}/{}/{}/{}",
        key.domain, key.stage_id, key.tool_id, key.corpus_id, key.asset_profile_id
    )
}

fn string_set(values: &[String]) -> BTreeSet<&str> {
    values.iter().map(String::as_str).collect()
}

fn artifact_id(entry: &str) -> String {
    entry
        .split_once('=')
        .map(|(artifact_id, _)| artifact_id.to_string())
        .unwrap_or_else(|| entry.to_string())
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

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_all_domain_output_contract_coverage,
        ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_SCHEMA_VERSION, COVERAGE_STATUS_COVERED,
        DEFAULT_ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_PATH, INDEX_STATUS_COVERED,
        INDEX_STATUS_NOT_APPLICABLE, PROOF_SOURCE_BAM, PROOF_SOURCE_FASTQ, PROOF_SOURCE_VCF,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_all_domain_output_contract_coverage_reports_complete_active_rows() {
        let root = repo_root();
        let report = render_all_domain_output_contract_coverage(
            &root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_PATH),
        )
        .expect("render all-domain output-contract coverage");

        assert_eq!(report.schema_version, ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_PATH);
        assert_eq!(report.result_id_count, report.row_count);
        assert_eq!(report.stage_count, 61);
        assert_eq!(report.tool_count, 68);
        assert_eq!(report.output_declaration_binding_count, report.row_count);
        assert_eq!(report.source_proof_binding_count, report.row_count);
        assert_eq!(report.covered_row_count, report.row_count);
        assert_eq!(report.missing_row_count, 0);
        assert_eq!(report.coverage_percent, 100.0);
        assert_eq!(report.raw_output_declared_row_count, report.row_count);
        assert_eq!(report.normalized_metrics_declared_row_count, report.row_count);
        assert_eq!(report.logs_declared_row_count, report.row_count);
        assert_eq!(report.manifest_declared_row_count, report.row_count);
        assert_eq!(report.domain_counts.get("fastq"), Some(&63));
        assert_eq!(report.domain_counts.get("bam"), Some(&49));
        assert_eq!(report.domain_counts.get("vcf"), Some(&16));
        assert_eq!(report.proof_source_counts.get(PROOF_SOURCE_FASTQ), Some(&63));
        assert_eq!(report.proof_source_counts.get(PROOF_SOURCE_BAM), Some(&49));
        assert_eq!(report.proof_source_counts.values().copied().sum::<usize>(), report.row_count);
        assert_eq!(report.index_required_row_count, report.index_declared_row_count);
        assert!(report.index_declared_row_count > 0);
        assert!(report.index_declared_row_count + report.index_not_applicable_row_count <= report.row_count);
        assert_eq!(
            report.index_coverage_counts.get(INDEX_STATUS_COVERED),
            Some(&report.index_declared_row_count)
        );
        assert_eq!(
            report.index_coverage_counts.get(INDEX_STATUS_NOT_APPLICABLE),
            Some(&report.index_not_applicable_row_count)
        );
        assert_eq!(report.coverage_status_counts.get(COVERAGE_STATUS_COVERED), Some(&report.row_count));
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);
        assert!(report.violations.is_empty());

        assert!(report.rows.iter().any(|row| {
            row.result_id == "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2"
                && row.proof_source == PROOF_SOURCE_FASTQ
                && row.source_contract_status == "complete"
                && row.output_declaration_status == "complete"
                && row.raw_outputs_declared
                && row.normalized_metrics_declared
                && row.logs_declared
                && row.manifest_declared
                && row.index_coverage_status == INDEX_STATUS_NOT_APPLICABLE
        }));
        assert!(report.rows.iter().any(|row| {
            row.result_id == "bam:corpus-01-kinship-mini:bam.kinship:sample-set:king"
                && row.proof_source == PROOF_SOURCE_BAM
                && row.source_contract_status == "complete"
                && row.output_declaration_status == "complete"
                && row.raw_outputs_declared
                && row.normalized_metrics_declared
                && row.logs_declared
                && row.manifest_declared
                && row.index_coverage_status == INDEX_STATUS_NOT_APPLICABLE
        }));
        assert!(report.rows.iter().any(|row| {
            row.result_id == "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools"
                && row.proof_source == PROOF_SOURCE_VCF
                && row.source_contract_status == "complete"
                && row.output_declaration_status == "complete"
                && row.raw_outputs_declared
                && row.normalized_metrics_declared
                && row.logs_declared
                && row.manifest_declared
                && row.index_coverage_status == INDEX_STATUS_COVERED
        }));
    }
}
