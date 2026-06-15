use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_stage_tool_matrix::{
    collect_all_domain_active_stage_tool_matrix_rows, AllDomainActiveStageToolMatrixRow,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_RETAINED_TOOLS_PATH: &str =
    "benchmarks/readiness/all-domains/retained-tools.tsv";
const ALL_DOMAIN_RETAINED_TOOLS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_retained_tools.v1";
const BENCHMARK_READY_STATUS: &str = "benchmark_ready";
const NO_BENCHMARK_READY_STAGE_IDS: &str = "none";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainRetainedToolRow {
    pub(crate) tool_id: String,
    pub(crate) domains: Vec<String>,
    pub(crate) active_stage_count: usize,
    pub(crate) benchmark_ready_stage_count: usize,
    pub(crate) active_binding_count: usize,
    pub(crate) benchmark_ready_binding_count: usize,
    pub(crate) benchmark_statuses: Vec<String>,
    pub(crate) active_stage_ids: Vec<String>,
    pub(crate) benchmark_ready_stage_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainRetainedToolsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) active_matrix_tool_count: usize,
    pub(crate) benchmark_ready_tool_count: usize,
    pub(crate) mixed_status_tool_count: usize,
    pub(crate) not_benchmark_ready_only_tool_count: usize,
    pub(crate) domain_tool_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<AllDomainRetainedToolRow>,
}

#[derive(Default)]
struct ToolAccumulator {
    domains: BTreeSet<String>,
    benchmark_statuses: BTreeSet<String>,
    active_stage_ids: BTreeSet<String>,
    benchmark_ready_stage_ids: BTreeSet<String>,
    active_binding_count: usize,
    benchmark_ready_binding_count: usize,
}

pub(crate) fn run_render_all_domain_retained_tools(
    args: &parse::BenchReadinessRenderAllDomainRetainedToolsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_retained_tools(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_RETAINED_TOOLS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_retained_tools(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainRetainedToolsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let source_rows = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?;
    let rows = collect_all_domain_retained_tool_rows(&source_rows)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_all_domain_retained_tools_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let row_count = rows.len();
    let benchmark_ready_tool_count =
        rows.iter().filter(|row| row.benchmark_ready_binding_count > 0).count();
    let mixed_status_tool_count =
        rows.iter().filter(|row| row.benchmark_statuses.len() > 1).count();
    let not_benchmark_ready_only_tool_count =
        rows.iter().filter(|row| row.benchmark_ready_binding_count == 0).count();
    let mut domain_tool_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        for domain in &row.domains {
            *domain_tool_counts.entry(domain.clone()).or_default() += 1;
        }
    }

    Ok(AllDomainRetainedToolsReport {
        schema_version: ALL_DOMAIN_RETAINED_TOOLS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count,
        active_matrix_tool_count: row_count,
        benchmark_ready_tool_count,
        mixed_status_tool_count,
        not_benchmark_ready_only_tool_count,
        domain_tool_counts,
        rows,
    })
}

fn collect_all_domain_retained_tool_rows(
    source_rows: &[AllDomainActiveStageToolMatrixRow],
) -> Result<Vec<AllDomainRetainedToolRow>> {
    let mut by_tool = BTreeMap::<String, ToolAccumulator>::new();
    for row in source_rows {
        let entry = by_tool.entry(row.tool_id.clone()).or_default();
        entry.domains.insert(row.domain.clone());
        entry.benchmark_statuses.insert(row.status.clone());
        entry.active_stage_ids.insert(row.stage_id.clone());
        entry.active_binding_count += 1;
        if row.status == BENCHMARK_READY_STATUS {
            entry.benchmark_ready_stage_ids.insert(row.stage_id.clone());
            entry.benchmark_ready_binding_count += 1;
        }
    }

    let rows = by_tool
        .into_iter()
        .map(|(tool_id, accumulator)| AllDomainRetainedToolRow {
            tool_id,
            domains: accumulator.domains.into_iter().collect(),
            active_stage_count: accumulator.active_stage_ids.len(),
            benchmark_ready_stage_count: accumulator.benchmark_ready_stage_ids.len(),
            active_binding_count: accumulator.active_binding_count,
            benchmark_ready_binding_count: accumulator.benchmark_ready_binding_count,
            benchmark_statuses: accumulator.benchmark_statuses.into_iter().collect(),
            active_stage_ids: accumulator.active_stage_ids.into_iter().collect(),
            benchmark_ready_stage_ids: accumulator.benchmark_ready_stage_ids.into_iter().collect(),
        })
        .collect::<Vec<_>>();

    ensure_all_domain_retained_tool_contract(source_rows, &rows)?;
    Ok(rows)
}

fn ensure_all_domain_retained_tool_contract(
    source_rows: &[AllDomainActiveStageToolMatrixRow],
    rows: &[AllDomainRetainedToolRow],
) -> Result<()> {
    let inventory_tool_ids = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>();
    if inventory_tool_ids.len() != rows.len() {
        return Err(anyhow!(
            "all-domain retained tool inventory must keep exactly one row per tool_id"
        ));
    }

    let active_matrix_tool_ids =
        source_rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>();
    if inventory_tool_ids != active_matrix_tool_ids {
        return Err(anyhow!(
            "all-domain retained tool inventory drifted from the governed active stage-tool scope"
        ));
    }

    for row in rows {
        if row.tool_id.trim().is_empty()
            || row.domains.is_empty()
            || row.active_stage_ids.is_empty()
            || row.active_binding_count == 0
            || row.active_stage_count == 0
            || row.benchmark_statuses.is_empty()
        {
            return Err(anyhow!(
                "all-domain retained tool row `{}` is missing a required retained-scope field",
                row.tool_id
            ));
        }
        if row.active_stage_count != row.active_stage_ids.len() {
            return Err(anyhow!(
                "all-domain retained tool row `{}` drifted from its active stage ids",
                row.tool_id
            ));
        }
        if row.benchmark_ready_stage_count != row.benchmark_ready_stage_ids.len() {
            return Err(anyhow!(
                "all-domain retained tool row `{}` drifted from its benchmark-ready stage ids",
                row.tool_id
            ));
        }
        if row.benchmark_ready_binding_count > row.active_binding_count
            || row.benchmark_ready_stage_count > row.active_stage_count
        {
            return Err(anyhow!(
                "all-domain retained tool row `{}` overcounts benchmark-ready coverage",
                row.tool_id
            ));
        }
        if row.active_stage_count == 0 {
            return Err(anyhow!("retained tool `{}` serves zero active stages", row.tool_id));
        }
    }

    Ok(())
}

fn render_all_domain_retained_tools_tsv(rows: &[AllDomainRetainedToolRow]) -> String {
    let mut rendered = String::from(
        "tool_id\tdomains\tactive_stage_count\tbenchmark_ready_stage_count\tactive_binding_count\tbenchmark_ready_binding_count\tbenchmark_statuses\tactive_stage_ids\tbenchmark_ready_stage_ids\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.tool_id,
            row.domains.join(","),
            row.active_stage_count,
            row.benchmark_ready_stage_count,
            row.active_binding_count,
            row.benchmark_ready_binding_count,
            row.benchmark_statuses.join(","),
            row.active_stage_ids.join(","),
            joined_stage_ids_or_none(&row.benchmark_ready_stage_ids),
        ));
    }
    rendered
}

fn joined_stage_ids_or_none(stage_ids: &[String]) -> String {
    if stage_ids.is_empty() {
        NO_BENCHMARK_READY_STAGE_IDS.to_string()
    } else {
        stage_ids.join(",")
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
