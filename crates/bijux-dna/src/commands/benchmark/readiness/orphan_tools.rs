use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::catalog::{load_benchmark_stage_ids, load_tool_contracts, ReadinessDomain};
use super::tool_serving_map::{
    render_bam_tool_serving_map, render_fastq_tool_serving_map, DEFAULT_BAM_TOOL_SERVING_MAP_PATH,
    DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ORPHAN_TOOLS_PATH: &str = "target/bench-readiness/orphan-tools.tsv";
const ORPHAN_TOOLS_SCHEMA_VERSION: &str = "bijux.bench.readiness.orphan_tools.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OrphanToolRow {
    pub(crate) domain: String,
    pub(crate) tool_id: String,
    pub(crate) decision: String,
    pub(crate) declared_stage_ids: Vec<String>,
    pub(crate) benchmark_stage_ids: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OrphanToolsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) orphan_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<OrphanToolRow>,
}

pub(crate) fn run_render_orphan_tools(
    args: &parse::BenchReadinessRenderOrphanToolsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_orphan_tools(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_ORPHAN_TOOLS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_orphan_tools(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<OrphanToolsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let fastq_map = render_fastq_tool_serving_map(
        repo_root,
        PathBuf::from(DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH),
    )?;
    let bam_map =
        render_bam_tool_serving_map(repo_root, PathBuf::from(DEFAULT_BAM_TOOL_SERVING_MAP_PATH))?;

    let covered_tool_ids = BTreeMap::from([
        (
            ReadinessDomain::Fastq,
            fastq_map.rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>(),
        ),
        (
            ReadinessDomain::Bam,
            bam_map.rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>(),
        ),
    ]);

    let mut rows = Vec::new();
    for domain in [ReadinessDomain::Fastq, ReadinessDomain::Bam] {
        let benchmark_stage_ids = load_benchmark_stage_ids(repo_root, domain)?;
        let tool_contracts = load_tool_contracts(repo_root, domain)?;
        let covered = covered_tool_ids.get(&domain).expect("covered tool ids");
        for contract in tool_contracts {
            if covered.contains(&contract.tool_id) {
                continue;
            }
            let declared_stage_ids = contract.admitted_stage_ids();
            let benchmark_stage_ids = contract.benchmark_stage_overlap(&benchmark_stage_ids);
            let (decision, reason) =
                orphan_decision(&contract.tool_id, &declared_stage_ids, &benchmark_stage_ids);
            rows.push(OrphanToolRow {
                domain: domain.as_str().to_string(),
                tool_id: contract.tool_id,
                decision: decision.to_string(),
                declared_stage_ids,
                benchmark_stage_ids,
                reason,
            });
        }
    }
    rows.sort_by(|left, right| {
        left.domain.cmp(&right.domain).then_with(|| left.tool_id.cmp(&right.tool_id))
    });

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_orphan_tools_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let mut domain_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
    }

    Ok(OrphanToolsReport {
        schema_version: ORPHAN_TOOLS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        orphan_count: rows.len(),
        domain_counts,
        rows,
    })
}

fn orphan_decision(
    tool_id: &str,
    declared_stage_ids: &[String],
    benchmark_stage_ids: &[String],
) -> (&'static str, String) {
    if !benchmark_stage_ids.is_empty() {
        return (
            "register_to_stage",
            format!(
                "tool `{tool_id}` already declares benchmarked stages {}; register it to the benchmark serving map",
                benchmark_stage_ids.join(", ")
            ),
        );
    }
    if declared_stage_ids.is_empty() {
        return (
            "remove_from_scope",
            format!(
                "tool `{tool_id}` declares no admitted stages, so it cannot serve the governed benchmark scope"
            ),
        );
    }
    (
        "future_tool",
        format!(
            "tool `{tool_id}` only declares non-benchmarked stages {}; keep it visible for future benchmark expansion",
            declared_stage_ids.join(", ")
        ),
    )
}

fn render_orphan_tools_tsv(rows: &[OrphanToolRow]) -> String {
    let mut rendered = String::from(
        "domain\ttool_id\tdecision\tdeclared_stage_ids\tbenchmark_stage_ids\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.decision),
            sanitize_tsv(&row.declared_stage_ids.join(",")),
            sanitize_tsv(&row.benchmark_stage_ids.join(",")),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
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

    use super::{render_orphan_tools, DEFAULT_ORPHAN_TOOLS_PATH, ORPHAN_TOOLS_SCHEMA_VERSION};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn orphan_tools_report_retains_governed_tool_decisions() {
        let root = repo_root();
        let report = render_orphan_tools(&root, PathBuf::from(DEFAULT_ORPHAN_TOOLS_PATH))
            .expect("render orphan tools");

        assert_eq!(report.schema_version, ORPHAN_TOOLS_SCHEMA_VERSION);
        assert!(!report.rows.is_empty(), "orphan tool report must retain governed orphan rows");
        assert_eq!(report.orphan_count, 3);
        assert!(report.rows.iter().any(|row| {
            row.domain == "bam"
                && row.tool_id == "addeam"
                && row.decision == "register_to_stage"
                && row.benchmark_stage_ids == vec!["bam.damage".to_string()]
        }));
        assert!(report.rows.iter().any(|row| {
            row.domain == "bam"
                && row.tool_id == "damageprofiler"
                && row.decision == "register_to_stage"
                && row.benchmark_stage_ids
                    == vec!["bam.authenticity".to_string(), "bam.damage".to_string()]
        }));
        assert!(report.rows.iter().any(|row| {
            row.domain == "bam"
                && row.tool_id == "ngsbriggs"
                && row.decision == "register_to_stage"
                && row.benchmark_stage_ids == vec!["bam.damage".to_string()]
        }));
    }
}
