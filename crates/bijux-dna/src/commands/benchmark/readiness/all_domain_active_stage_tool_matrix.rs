use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_stage_tool_table::collect_all_domain_stage_tool_table_rows;
use super::bam_command_adapter_coverage::{
    collect_bam_command_adapter_coverage_rows, BamBenchmarkStatus,
};
use super::bam_normalized_metrics_schema::collect_bam_normalized_metrics_schema_report_rows;
use super::fastq_command_adapter_coverage::{
    collect_fastq_command_adapter_coverage_rows, FastqBenchmarkStatus,
};
use super::fastq_normalized_metrics_schema::collect_fastq_normalized_metrics_schema_report_rows;
use super::vcf_normalized_metrics_schema::collect_vcf_normalized_metrics_schema_report_rows;
use super::vcf_tool_serving_map::collect_vcf_tool_serving_map_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH: &str =
    "benchmarks/readiness/all-domains/active-stage-tool-matrix.tsv";
const ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_active_stage_tool_matrix.v1";
pub(crate) const STATUS_BENCHMARK_READY: &str = "benchmark_ready";
pub(crate) const STATUS_NOT_BENCHMARK_READY: &str = "not_benchmark_ready";
pub(crate) const STATUS_PLANNED: &str = "planned";
pub(crate) const STATUS_FUTURE: &str = "future";
pub(crate) const ADAPTER_STATUS_RUNNABLE: &str = "runnable";
pub(crate) const ADAPTER_STATUS_PLANNABLE: &str = "plannable";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    domain: String,
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainActiveStageToolMatrixRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) schema_id: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainActiveStageToolMatrixReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) status_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<AllDomainActiveStageToolMatrixRow>,
}

pub(crate) fn run_render_all_domain_active_stage_tool_matrix(
    args: &parse::BenchReadinessRenderAllDomainActiveStageToolMatrixArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_active_stage_tool_matrix(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_active_stage_tool_matrix(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainActiveStageToolMatrixReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_all_domain_active_stage_tool_matrix_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let row_count = rows.len();
    let stage_count = rows
        .iter()
        .map(|row| (row.domain.as_str(), row.stage_id.as_str()))
        .collect::<BTreeSet<_>>()
        .len();
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *status_counts.entry(row.status.clone()).or_default() += 1;
    }

    Ok(AllDomainActiveStageToolMatrixReport {
        schema_version: ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count,
        stage_count,
        tool_count,
        domain_counts,
        status_counts,
        rows,
    })
}

pub(crate) fn collect_all_domain_active_stage_tool_matrix_rows(
    repo_root: &Path,
) -> Result<Vec<AllDomainActiveStageToolMatrixRow>> {
    let lifecycle_active_rows =
        collect_all_domain_lifecycle_active_stage_tool_matrix_rows(repo_root)?;
    let adapter_status_by_binding = collect_all_domain_adapter_status_by_binding(repo_root)?;
    let mut rows = Vec::with_capacity(lifecycle_active_rows.len());
    for row in lifecycle_active_rows {
        if row_has_executable_adapter(&row, &adapter_status_by_binding)? {
            rows.push(row);
        }
    }
    ensure_all_domain_active_stage_tool_matrix_contract(repo_root, &rows)?;
    Ok(rows)
}

pub(crate) fn collect_all_domain_lifecycle_active_stage_tool_matrix_rows(
    repo_root: &Path,
) -> Result<Vec<AllDomainActiveStageToolMatrixRow>> {
    Ok(collect_all_domain_active_stage_tool_matrix_candidate_rows(repo_root)?
        .into_iter()
        .filter(|row| is_active_scope_status(&row.status))
        .collect::<Vec<_>>())
}

pub(crate) fn collect_all_domain_active_stage_tool_matrix_candidate_rows(
    repo_root: &Path,
) -> Result<Vec<AllDomainActiveStageToolMatrixRow>> {
    let base_rows = collect_all_domain_stage_tool_table_rows(repo_root)?;
    let status_by_binding = collect_status_by_binding(repo_root)?;
    let schema_id_by_stage = collect_schema_id_by_stage()?;

    let mut rows = base_rows
        .iter()
        .map(|row| {
            let key = binding_key(&row.domain, &row.stage_id, &row.tool_id);
            let status = status_by_binding.get(&key).cloned().ok_or_else(|| {
                anyhow!(
                    "all-domain active stage-tool matrix is missing status coverage for `{}` / `{}` / `{}`",
                    row.domain,
                    row.stage_id,
                    row.tool_id
                )
            })?;
            let schema_id = schema_id_by_stage
                .get(&(row.domain.clone(), row.stage_id.clone()))
                .cloned()
                .ok_or_else(|| {
                    anyhow!(
                        "all-domain active stage-tool matrix is missing schema coverage for `{}` / `{}`",
                        row.domain,
                        row.stage_id
                    )
                })?;

            Ok(AllDomainActiveStageToolMatrixRow {
                domain: row.domain.clone(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                corpus_id: row.corpus_id.clone(),
                asset_profile_id: row.asset_profile_id.clone(),
                adapter_id: row.adapter_id.clone(),
                parser_id: row.parser_id.clone(),
                schema_id,
                status,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    Ok(rows)
}

fn collect_status_by_binding(repo_root: &Path) -> Result<BTreeMap<BindingKey, String>> {
    let mut rows = BTreeMap::<BindingKey, String>::new();

    for row in collect_fastq_command_adapter_coverage_rows(repo_root)?.2 {
        rows.insert(
            binding_key("fastq", &row.stage_id, &row.tool_id),
            fastq_status_label(&row.support_status, row.benchmark_status).to_string(),
        );
    }

    for row in collect_bam_command_adapter_coverage_rows(repo_root)?.2 {
        rows.insert(
            binding_key("bam", &row.stage_id, &row.tool_id),
            bam_status_label(&row.support_status, row.benchmark_status).to_string(),
        );
    }

    for row in collect_vcf_tool_serving_map_rows()? {
        rows.insert(
            binding_key("vcf", &row.stage_id, &row.tool_id),
            vcf_status_label(&row.support_status, &row.benchmark_status).to_string(),
        );
    }

    Ok(rows)
}

pub(crate) fn collect_all_domain_adapter_status_by_binding(
    repo_root: &Path,
) -> Result<BTreeMap<(String, String, String), String>> {
    let mut rows = BTreeMap::<(String, String, String), String>::new();

    for row in collect_fastq_command_adapter_coverage_rows(repo_root)?.2 {
        rows.insert(
            binding_tuple("fastq", &row.stage_id, &row.tool_id),
            row.adapter_status.clone(),
        );
    }

    for row in collect_bam_command_adapter_coverage_rows(repo_root)?.2 {
        rows.insert(binding_tuple("bam", &row.stage_id, &row.tool_id), row.adapter_status.clone());
    }

    for row in collect_vcf_tool_serving_map_rows()? {
        rows.insert(binding_tuple("vcf", &row.stage_id, &row.tool_id), row.adapter_status.clone());
    }

    Ok(rows)
}

fn collect_schema_id_by_stage() -> Result<BTreeMap<(String, String), String>> {
    let mut rows = BTreeMap::<(String, String), String>::new();

    for row in collect_fastq_normalized_metrics_schema_report_rows()? {
        rows.insert((String::from("fastq"), row.stage_id), row.extension_id);
    }
    for row in collect_bam_normalized_metrics_schema_report_rows()? {
        rows.insert((String::from("bam"), row.stage_id), row.extension_id);
    }
    for row in collect_vcf_normalized_metrics_schema_report_rows()? {
        rows.insert((String::from("vcf"), row.stage_id), row.schema_id);
    }

    Ok(rows)
}

fn ensure_all_domain_active_stage_tool_matrix_contract(
    repo_root: &Path,
    rows: &[AllDomainActiveStageToolMatrixRow],
) -> Result<()> {
    let lifecycle_active_rows =
        collect_all_domain_lifecycle_active_stage_tool_matrix_rows(repo_root)?;
    let adapter_status_by_binding = collect_all_domain_adapter_status_by_binding(repo_root)?;
    let row_keys = rows
        .iter()
        .map(|row| binding_key(&row.domain, &row.stage_id, &row.tool_id))
        .collect::<BTreeSet<_>>();
    if row_keys.len() != rows.len() {
        return Err(anyhow!(
            "all-domain active stage-tool matrix must keep exactly one row per domain/stage_id/tool_id"
        ));
    }

    let mut base_keys = BTreeSet::<BindingKey>::new();
    for row in &lifecycle_active_rows {
        if row_has_executable_adapter(row, &adapter_status_by_binding)? {
            base_keys.insert(binding_key(&row.domain, &row.stage_id, &row.tool_id));
        }
    }
    if row_keys != base_keys {
        return Err(anyhow!(
            "all-domain active stage-tool matrix drifted from the governed active-scope stage-tool surface"
        ));
    }

    let mut active_stage_keys = BTreeSet::<(String, String)>::new();
    for row in &lifecycle_active_rows {
        if row_has_executable_adapter(row, &adapter_status_by_binding)? {
            active_stage_keys.insert((row.domain.clone(), row.stage_id.clone()));
        }
    }
    let matrix_stage_keys =
        rows.iter().map(|row| (row.domain.clone(), row.stage_id.clone())).collect::<BTreeSet<_>>();
    if matrix_stage_keys != active_stage_keys {
        return Err(anyhow!(
            "all-domain active stage-tool matrix drifted from the governed active-scope stage coverage"
        ));
    }

    let mut active_tool_keys = BTreeSet::<String>::new();
    for row in &lifecycle_active_rows {
        if row_has_executable_adapter(row, &adapter_status_by_binding)? {
            active_tool_keys.insert(row.tool_id.clone());
        }
    }
    let matrix_tool_keys = rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>();
    if matrix_tool_keys != active_tool_keys {
        return Err(anyhow!(
            "all-domain active stage-tool matrix drifted from the governed active-scope tool coverage"
        ));
    }

    for row in rows {
        if row.domain.trim().is_empty()
            || row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.corpus_id.trim().is_empty()
            || row.asset_profile_id.trim().is_empty()
            || row.adapter_id.trim().is_empty()
            || row.parser_id.trim().is_empty()
            || row.schema_id.trim().is_empty()
            || row.status.trim().is_empty()
        {
            return Err(anyhow!(
                "all-domain active stage-tool matrix row `{}` / `{}` / `{}` contains a blank required column",
                row.domain,
                row.stage_id,
                row.tool_id
            ));
        }
        if !matches!(
            row.status.as_str(),
            STATUS_BENCHMARK_READY | STATUS_NOT_BENCHMARK_READY | STATUS_PLANNED | STATUS_FUTURE
        ) {
            return Err(anyhow!(
                "all-domain active stage-tool matrix row `{}` / `{}` / `{}` has unsupported status `{}`",
                row.domain,
                row.stage_id,
                row.tool_id,
                row.status
            ));
        }
        if !row_has_executable_adapter(row, &adapter_status_by_binding)? {
            return Err(anyhow!(
                "all-domain active stage-tool matrix row `{}` / `{}` / `{}` lacks an executable adapter contract",
                row.domain,
                row.stage_id,
                row.tool_id
            ));
        }
    }

    Ok(())
}

pub(crate) fn is_active_scope_status(status: &str) -> bool {
    matches!(status, STATUS_BENCHMARK_READY | STATUS_NOT_BENCHMARK_READY)
}

pub(crate) fn is_executable_adapter_status(adapter_status: &str) -> bool {
    matches!(adapter_status, ADAPTER_STATUS_RUNNABLE | ADAPTER_STATUS_PLANNABLE)
}

fn render_all_domain_active_stage_tool_matrix_tsv(
    rows: &[AllDomainActiveStageToolMatrixRow],
) -> String {
    let mut rendered = String::from(
        "domain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tstatus\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.domain,
            row.stage_id,
            row.tool_id,
            row.corpus_id,
            row.asset_profile_id,
            row.adapter_id,
            row.parser_id,
            row.schema_id,
            row.status,
        ));
    }
    rendered
}

fn fastq_status_label(
    support_status: &str,
    benchmark_status: FastqBenchmarkStatus,
) -> &'static str {
    if support_status == "planned_contract" {
        STATUS_PLANNED
    } else if support_status.contains("future") {
        STATUS_FUTURE
    } else if benchmark_status == FastqBenchmarkStatus::BenchmarkReady {
        STATUS_BENCHMARK_READY
    } else {
        STATUS_NOT_BENCHMARK_READY
    }
}

fn bam_status_label(support_status: &str, benchmark_status: BamBenchmarkStatus) -> &'static str {
    if support_status == "planned" || support_status == "planned_contract" {
        STATUS_PLANNED
    } else if support_status.contains("future") {
        STATUS_FUTURE
    } else if benchmark_status == BamBenchmarkStatus::BenchmarkReady {
        STATUS_BENCHMARK_READY
    } else {
        STATUS_NOT_BENCHMARK_READY
    }
}

fn vcf_status_label(support_status: &str, benchmark_status: &str) -> &'static str {
    if support_status == "planned" || support_status == "planned_contract" {
        STATUS_PLANNED
    } else if support_status.contains("future") {
        STATUS_FUTURE
    } else if benchmark_status == STATUS_BENCHMARK_READY {
        STATUS_BENCHMARK_READY
    } else {
        STATUS_NOT_BENCHMARK_READY
    }
}

fn binding_key(domain: &str, stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
    }
}

fn binding_tuple(domain: &str, stage_id: &str, tool_id: &str) -> (String, String, String) {
    (domain.to_string(), stage_id.to_string(), tool_id.to_string())
}

fn row_has_executable_adapter(
    row: &AllDomainActiveStageToolMatrixRow,
    adapter_status_by_binding: &BTreeMap<(String, String, String), String>,
) -> Result<bool> {
    let adapter_status = adapter_status_by_binding
        .get(&binding_tuple(&row.domain, &row.stage_id, &row.tool_id))
        .ok_or_else(|| {
            anyhow!(
                "all-domain active stage-tool matrix is missing adapter status coverage for `{}` / `{}` / `{}`",
                row.domain,
                row.stage_id,
                row.tool_id
            )
        })?;
    Ok(is_executable_adapter_status(adapter_status))
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
