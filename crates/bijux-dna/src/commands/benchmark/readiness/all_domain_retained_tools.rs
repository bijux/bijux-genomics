use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_api::v1::api::bench::{
    derive_all_domain_retained_tool_rows, validate_all_domain_retained_tools,
    ALL_DOMAIN_RETAINED_TOOLS_SCHEMA_V1,
};
pub(crate) use bijux_dna_api::v1::api::bench::{
    AllDomainRetainedToolRow, AllDomainRetainedToolsReport,
};

use super::all_domain_active_stage_tool_matrix::collect_all_domain_executable_active_stage_tool_matrix_rows;
use super::catalog::load_external_artifact_tool_contracts;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_RETAINED_TOOLS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/tools/retained-tool-inventory.tsv";
const NO_BENCHMARK_READY_STAGE_IDS: &str = "none";

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
    let rows = collect_all_domain_retained_tool_rows(repo_root)?;

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

    let rendered = render_all_domain_retained_tools_tsv(&rows);
    let report = AllDomainRetainedToolsReport {
        schema_version: ALL_DOMAIN_RETAINED_TOOLS_SCHEMA_V1.to_string(),
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count,
        active_matrix_tool_count: row_count,
        benchmark_ready_tool_count,
        mixed_status_tool_count,
        not_benchmark_ready_only_tool_count,
        domain_tool_counts,
        rows,
    };
    validate_all_domain_retained_tools(&report)
        .context("validate all-domain retained-tool inventory")?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, rendered)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(report)
}

pub(crate) fn collect_all_domain_retained_tool_rows(
    repo_root: &Path,
) -> Result<Vec<AllDomainRetainedToolRow>> {
    let source_rows = collect_all_domain_executable_active_stage_tool_matrix_rows(repo_root)?;
    let external_artifacts = load_external_artifact_tool_contracts(repo_root)?;
    derive_all_domain_retained_tool_rows(&source_rows, &external_artifacts)
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
