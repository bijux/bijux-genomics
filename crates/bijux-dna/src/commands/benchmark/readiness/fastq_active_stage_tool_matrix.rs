use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_stage_tool_matrix::collect_all_domain_active_stage_tool_matrix_rows;
use super::fastq_command_adapter_coverage::{
    collect_fastq_command_adapter_coverage_rows, FastqBenchmarkStatus,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FASTQ_ACTIVE_STAGE_TOOL_MATRIX_PATH: &str =
    "benchmarks/readiness/fastq/fastq-active-stage-tool-matrix.tsv";
const FASTQ_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_active_stage_tool_matrix.v1";
const ACTIVE_SCOPE_PROOF_PATH: &str =
    "benchmarks/readiness/all-domains/active-stage-tool-matrix.tsv";
const REMOVED_FROM_SCOPE_PROOF_PATH: &str = "benchmarks/readiness/removed-from-scope.tsv";
const BENCHMARK_READY_STATUS: &str = "benchmark_ready";
const PLANNED_SUPPORT_STATUS: &str = "planned_contract";
const DECLARED_ONLY_ADAPTER_STATUS: &str = "declared_only";
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FastqActiveStageToolMatrixRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) schema_id: String,
    pub(crate) benchmark_status: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
    pub(crate) scope_proof_path: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqActiveStageToolMatrixReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) retained_row_count: usize,
    pub(crate) retained_stage_count: usize,
    pub(crate) retained_tool_count: usize,
    pub(crate) row_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) removed_row_count: usize,
    pub(crate) removed_stage_count: usize,
    pub(crate) removed_tool_count: usize,
    pub(crate) removed_from_scope_path: &'static str,
    pub(crate) support_status_counts: BTreeMap<String, usize>,
    pub(crate) parser_status_counts: BTreeMap<String, usize>,
    pub(crate) corpus_status_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<FastqActiveStageToolMatrixRow>,
}

#[derive(Debug, Clone)]
struct FastqActiveStageToolMatrixCollection {
    retained_row_count: usize,
    retained_stage_count: usize,
    retained_tool_count: usize,
    removed_row_count: usize,
    removed_stage_count: usize,
    removed_tool_count: usize,
    rows: Vec<FastqActiveStageToolMatrixRow>,
}

pub(crate) fn run_render_fastq_active_stage_tool_matrix(
    args: &parse::BenchReadinessRenderFastqActiveStageToolMatrixArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_active_stage_tool_matrix(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_ACTIVE_STAGE_TOOL_MATRIX_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_fastq_active_stage_tool_matrix(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqActiveStageToolMatrixReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let collection = collect_fastq_active_stage_tool_matrix_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_fastq_active_stage_tool_matrix_tsv(&collection.rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let mut support_status_counts = BTreeMap::<String, usize>::new();
    let mut parser_status_counts = BTreeMap::<String, usize>::new();
    let mut corpus_status_counts = BTreeMap::<String, usize>::new();
    for row in &collection.rows {
        *support_status_counts.entry(row.support_status.clone()).or_default() += 1;
        *parser_status_counts.entry(row.parser_status.clone()).or_default() += 1;
        *corpus_status_counts.entry(row.corpus_status.clone()).or_default() += 1;
    }

    Ok(FastqActiveStageToolMatrixReport {
        schema_version: FASTQ_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        retained_row_count: collection.retained_row_count,
        retained_stage_count: collection.retained_stage_count,
        retained_tool_count: collection.retained_tool_count,
        row_count: collection.rows.len(),
        stage_count: collection
            .rows
            .iter()
            .map(|row| row.stage_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        tool_count: collection
            .rows
            .iter()
            .map(|row| row.tool_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        removed_row_count: collection.removed_row_count,
        removed_stage_count: collection.removed_stage_count,
        removed_tool_count: collection.removed_tool_count,
        removed_from_scope_path: REMOVED_FROM_SCOPE_PROOF_PATH,
        support_status_counts,
        parser_status_counts,
        corpus_status_counts,
        rows: collection.rows,
    })
}

fn collect_fastq_active_stage_tool_matrix_rows(
    repo_root: &Path,
) -> Result<FastqActiveStageToolMatrixCollection> {
    let (retained_stage_count, retained_tool_count, coverage_rows) =
        collect_fastq_command_adapter_coverage_rows(repo_root)?;
    let coverage_by_binding = coverage_rows
        .iter()
        .map(|row| {
            (
                BindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() },
                row.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let removed_rows = coverage_rows
        .iter()
        .filter(|row| row.benchmark_status != FastqBenchmarkStatus::BenchmarkReady)
        .cloned()
        .collect::<Vec<_>>();

    let mut rows = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?
        .into_iter()
        .filter(|row| row.domain == "fastq")
        .map(|active_row| {
            let key = BindingKey {
                stage_id: active_row.stage_id.clone(),
                tool_id: active_row.tool_id.clone(),
            };
            let coverage_row = coverage_by_binding.get(&key).ok_or_else(|| {
                anyhow!(
                    "FASTQ active-stage-tool matrix is missing command-adapter coverage for `{}` / `{}`",
                    active_row.stage_id,
                    active_row.tool_id
                )
            })?;
            build_fastq_active_stage_tool_matrix_row(active_row, coverage_row)
        })
        .collect::<Result<Vec<_>>>()?;

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_fastq_active_stage_tool_matrix_contract(&rows)?;

    Ok(FastqActiveStageToolMatrixCollection {
        retained_row_count: coverage_rows.len(),
        retained_stage_count,
        retained_tool_count,
        removed_row_count: removed_rows.len(),
        removed_stage_count: removed_rows
            .iter()
            .map(|row| row.stage_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        removed_tool_count: removed_rows
            .iter()
            .map(|row| row.tool_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        rows,
    })
}

fn build_fastq_active_stage_tool_matrix_row(
    active_row: super::all_domain_active_stage_tool_matrix::AllDomainActiveStageToolMatrixRow,
    coverage_row: &super::fastq_command_adapter_coverage::FastqCommandAdapterCoverageRow,
) -> Result<FastqActiveStageToolMatrixRow> {
    if coverage_row.benchmark_status != FastqBenchmarkStatus::BenchmarkReady {
        return Err(anyhow!(
            "FASTQ active-stage-tool matrix cannot retain non-benchmark-ready row `{}` / `{}`",
            active_row.stage_id,
            active_row.tool_id
        ));
    }
    if coverage_row.support_status == PLANNED_SUPPORT_STATUS {
        return Err(anyhow!(
            "FASTQ active-stage-tool matrix cannot retain planned support row `{}` / `{}`",
            active_row.stage_id,
            active_row.tool_id
        ));
    }
    if coverage_row.adapter_status == DECLARED_ONLY_ADAPTER_STATUS {
        return Err(anyhow!(
            "FASTQ active-stage-tool matrix cannot retain declared-only adapter row `{}` / `{}`",
            active_row.stage_id,
            active_row.tool_id
        ));
    }
    if coverage_row.corpus_status == "planner_only" {
        return Err(anyhow!(
            "FASTQ active-stage-tool matrix cannot retain excluded corpus row `{}` / `{}`",
            active_row.stage_id,
            active_row.tool_id
        ));
    }

    Ok(FastqActiveStageToolMatrixRow {
        stage_id: active_row.stage_id.clone(),
        tool_id: active_row.tool_id.clone(),
        corpus_id: active_row.corpus_id,
        asset_profile_id: active_row.asset_profile_id,
        adapter_id: active_row.adapter_id,
        parser_id: active_row.parser_id,
        schema_id: active_row.schema_id,
        benchmark_status: BENCHMARK_READY_STATUS.to_string(),
        support_status: coverage_row.support_status.clone(),
        adapter_status: coverage_row.adapter_status.clone(),
        parser_status: coverage_row.parser_status.clone(),
        corpus_status: coverage_row.corpus_status.clone(),
        scope_proof_path: ACTIVE_SCOPE_PROOF_PATH.to_string(),
        reason: format!(
            "binding `{}` / `{}` remains in governed FASTQ active scope because it is benchmark_ready with executable adapter coverage, normalized parser coverage, and governed benchmark-scope coverage",
            active_row.stage_id, active_row.tool_id
        ),
    })
}

fn ensure_fastq_active_stage_tool_matrix_contract(
    rows: &[FastqActiveStageToolMatrixRow],
) -> Result<()> {
    let mut seen = BTreeSet::<BindingKey>::new();
    for row in rows {
        if !seen.insert(BindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() })
        {
            return Err(anyhow!(
                "FASTQ active-stage-tool matrix contains duplicate `{}` / `{}`",
                row.stage_id,
                row.tool_id
            ));
        }
        if row.benchmark_status != BENCHMARK_READY_STATUS {
            return Err(anyhow!(
                "FASTQ active-stage-tool matrix row `{}` / `{}` must stay benchmark_ready",
                row.stage_id,
                row.tool_id
            ));
        }
        if row.support_status == PLANNED_SUPPORT_STATUS
            || row.adapter_status == DECLARED_ONLY_ADAPTER_STATUS
            || row.corpus_status == "planner_only"
        {
            return Err(anyhow!(
                "FASTQ active-stage-tool matrix row `{}` / `{}` leaked a non-active readiness status",
                row.stage_id,
                row.tool_id
            ));
        }
        if row.corpus_id.trim().is_empty()
            || row.asset_profile_id.trim().is_empty()
            || row.adapter_id.trim().is_empty()
            || row.parser_id.trim().is_empty()
            || row.schema_id.trim().is_empty()
        {
            return Err(anyhow!(
                "FASTQ active-stage-tool matrix row `{}` / `{}` contains a blank required column",
                row.stage_id,
                row.tool_id
            ));
        }
    }
    Ok(())
}

fn render_fastq_active_stage_tool_matrix_tsv(rows: &[FastqActiveStageToolMatrixRow]) -> String {
    let mut rendered = String::from(
        "stage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tbenchmark_status\tsupport_status\tadapter_status\tparser_status\tcorpus_status\tscope_proof_path\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.corpus_id),
            sanitize_tsv(&row.asset_profile_id),
            sanitize_tsv(&row.adapter_id),
            sanitize_tsv(&row.parser_id),
            sanitize_tsv(&row.schema_id),
            sanitize_tsv(&row.benchmark_status),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.corpus_status),
            sanitize_tsv(&row.scope_proof_path),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn sanitize_tsv(value: &str) -> String {
    value.replace('\t', " ").replace('\n', " ")
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
