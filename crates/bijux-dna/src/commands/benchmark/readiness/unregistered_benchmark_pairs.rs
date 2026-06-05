use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::catalog::load_registry_tool_matrix;
use super::tool_serving_map::{
    render_bam_tool_serving_map, render_fastq_tool_serving_map, ToolServingMapReport,
    DEFAULT_BAM_TOOL_SERVING_MAP_PATH, DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_UNREGISTERED_BENCHMARK_PAIRS_PATH: &str =
    "target/bench-readiness/unregistered-benchmark-pairs.tsv";
const UNREGISTERED_BENCHMARK_PAIRS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.unregistered_benchmark_pairs.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct UnregisteredBenchmarkPairRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) support_status: String,
    pub(crate) registry_status: String,
    pub(crate) registered_stage_ids: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UnregisteredBenchmarkPairsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) unregistered_pair_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<UnregisteredBenchmarkPairRow>,
}

pub(crate) fn run_render_unregistered_benchmark_pairs(
    args: &parse::BenchReadinessRenderUnregisteredBenchmarkPairsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_unregistered_benchmark_pairs(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_UNREGISTERED_BENCHMARK_PAIRS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_unregistered_benchmark_pairs(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<UnregisteredBenchmarkPairsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let registry = load_registry_tool_matrix(repo_root)?;
    let fastq_map = render_fastq_tool_serving_map(
        repo_root,
        PathBuf::from(DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH),
    )?;
    let bam_map =
        render_bam_tool_serving_map(repo_root, PathBuf::from(DEFAULT_BAM_TOOL_SERVING_MAP_PATH))?;

    let mut rows = collect_unregistered_rows(&registry, &fastq_map);
    rows.extend(collect_unregistered_rows(&registry, &bam_map));
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_unregistered_benchmark_pairs_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let mut domain_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
    }

    Ok(UnregisteredBenchmarkPairsReport {
        schema_version: UNREGISTERED_BENCHMARK_PAIRS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        unregistered_pair_count: rows.len(),
        domain_counts,
        ok: rows.is_empty(),
        rows,
    })
}

fn collect_unregistered_rows(
    registry: &super::catalog::RegistryToolMatrix,
    report: &ToolServingMapReport,
) -> Vec<UnregisteredBenchmarkPairRow> {
    let mut rows = Vec::new();
    for row in &report.rows {
        if registry.tool_stage_pairs.contains(&(row.stage_id.clone(), row.tool_id.clone())) {
            continue;
        }
        let registry_status = if registry.known_tool_ids.contains(&row.tool_id) {
            "tool_registered_pair_missing"
        } else {
            "tool_missing"
        };
        let registered_stage_ids =
            registry.stage_ids_by_tool.get(&row.tool_id).cloned().unwrap_or_default();
        rows.push(UnregisteredBenchmarkPairRow {
            domain: report.domain.to_string(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            support_status: row.support_status.clone(),
            registry_status: registry_status.to_string(),
            registered_stage_ids: registered_stage_ids.clone(),
            reason: format!(
                "benchmark matrix references `{}` / `{}` but configs/ci/registry/tool_registry.toml does not register that pair; registry status: {}; registered stages for `{}`: {}",
                row.stage_id,
                row.tool_id,
                registry_status,
                row.tool_id,
                if registered_stage_ids.is_empty() {
                    "<none>".to_string()
                } else {
                    registered_stage_ids.join(", ")
                }
            ),
        });
    }
    rows
}

fn render_unregistered_benchmark_pairs_tsv(rows: &[UnregisteredBenchmarkPairRow]) -> String {
    let mut rendered = String::from(
        "domain\tstage_id\ttool_id\tsupport_status\tregistry_status\tregistered_stage_ids\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.registry_status),
            sanitize_tsv(&row.registered_stage_ids.join(",")),
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

    use super::{
        render_unregistered_benchmark_pairs, DEFAULT_UNREGISTERED_BENCHMARK_PAIRS_PATH,
        UNREGISTERED_BENCHMARK_PAIRS_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn unregistered_benchmark_pairs_report_retains_registry_drift() {
        let root = repo_root();
        let report = render_unregistered_benchmark_pairs(
            &root,
            PathBuf::from(DEFAULT_UNREGISTERED_BENCHMARK_PAIRS_PATH),
        )
        .expect("render unregistered benchmark pairs");

        assert_eq!(report.schema_version, UNREGISTERED_BENCHMARK_PAIRS_SCHEMA_VERSION);
        assert_eq!(report.unregistered_pair_count, 9);
        assert!(!report.ok, "report must fail while registry drift remains");
        assert_eq!(report.domain_counts.get("fastq"), Some(&5));
        assert_eq!(report.domain_counts.get("bam"), Some(&4));
        assert!(report.rows.iter().any(|row| {
            row.domain == "fastq"
                && row.stage_id == "fastq.estimate_library_complexity_prealign"
                && row.tool_id == "bijux_dna"
                && row.registry_status == "tool_registered_pair_missing"
                && row.registered_stage_ids == vec!["fastq.detect_duplicates_premerge".to_string()]
        }));
        assert!(report.rows.iter().any(|row| {
            row.domain == "bam"
                && row.stage_id == "bam.genotyping"
                && row.tool_id == "angsd"
                && row.registry_status == "tool_registered_pair_missing"
                && row.registered_stage_ids
                    == vec!["bam.kinship".to_string(), "bam.sex".to_string()]
        }));
        assert!(
            !report.rows.iter().any(|row| {
                row.domain == "bam"
                    && row.stage_id == "bam.haplogroups"
                    && row.tool_id == "yleaf"
            }),
            "bam.haplogroups / yleaf must leave the registry-drift slice once yleaf is registered in production"
        );
    }
}
