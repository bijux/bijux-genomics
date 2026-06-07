use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::tool_serving_map::{
    render_bam_tool_serving_map, render_fastq_tool_serving_map, DEFAULT_BAM_TOOL_SERVING_MAP_PATH,
    DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_TOOL_ID_NORMALIZATION_PATH: &str =
    "benchmarks/readiness/tool-id-normalization.tsv";
const TOOL_ID_NORMALIZATION_SCHEMA_VERSION: &str = "bijux.bench.readiness.tool_id_normalization.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ToolIdNormalizationRow {
    pub(crate) normalized_tool_id: String,
    pub(crate) canonical_tool_id: String,
    pub(crate) alias_tool_ids: Vec<String>,
    pub(crate) domains: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolIdNormalizationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) cluster_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<ToolIdNormalizationRow>,
}

pub(crate) fn run_render_tool_id_normalization(
    args: &parse::BenchReadinessRenderToolIdNormalizationArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_tool_id_normalization(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_TOOL_ID_NORMALIZATION_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_tool_id_normalization(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<ToolIdNormalizationReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let fastq_map = render_fastq_tool_serving_map(
        repo_root,
        PathBuf::from(DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH),
    )?;
    let bam_map =
        render_bam_tool_serving_map(repo_root, PathBuf::from(DEFAULT_BAM_TOOL_SERVING_MAP_PATH))?;

    let mut domains_by_tool_id = BTreeMap::<String, BTreeSet<String>>::new();
    for row in &fastq_map.rows {
        domains_by_tool_id.entry(row.tool_id.clone()).or_default().insert("fastq".to_string());
    }
    for row in &bam_map.rows {
        domains_by_tool_id.entry(row.tool_id.clone()).or_default().insert("bam".to_string());
    }

    let mut tool_ids_by_normalized = BTreeMap::<String, BTreeSet<String>>::new();
    for tool_id in domains_by_tool_id.keys() {
        tool_ids_by_normalized
            .entry(normalize_tool_id(tool_id))
            .or_default()
            .insert(tool_id.clone());
    }

    let mut rows = Vec::new();
    for (normalized_tool_id, cluster) in tool_ids_by_normalized {
        if cluster.len() <= 1 {
            continue;
        }
        let cluster_tool_ids = cluster.into_iter().collect::<Vec<_>>();
        let canonical_tool_id = choose_canonical_tool_id(&cluster_tool_ids);
        let alias_tool_ids = cluster_tool_ids
            .iter()
            .filter(|tool_id| *tool_id != &canonical_tool_id)
            .cloned()
            .collect::<Vec<_>>();
        let domains = cluster_tool_ids
            .iter()
            .flat_map(|tool_id| {
                domains_by_tool_id
                    .get(tool_id)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .collect::<Vec<_>>()
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        rows.push(ToolIdNormalizationRow {
            normalized_tool_id: normalized_tool_id.clone(),
            canonical_tool_id: canonical_tool_id.clone(),
            alias_tool_ids: alias_tool_ids.clone(),
            domains,
            reason: format!(
                "benchmark tool IDs {} collapse to normalized key `{}`; use canonical `{}` to avoid cross-domain alias drift",
                cluster_tool_ids.join(", "),
                normalized_tool_id,
                canonical_tool_id,
            ),
        });
    }

    rows.sort_by(|left, right| left.normalized_tool_id.cmp(&right.normalized_tool_id));

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_tool_id_normalization_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(ToolIdNormalizationReport {
        schema_version: TOOL_ID_NORMALIZATION_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        cluster_count: rows.len(),
        ok: rows.is_empty(),
        rows,
    })
}

fn normalize_tool_id(tool_id: &str) -> String {
    tool_id
        .chars()
        .filter(|character| *character != '-' && *character != '_')
        .flat_map(char::to_lowercase)
        .collect()
}

fn choose_canonical_tool_id(cluster_tool_ids: &[String]) -> String {
    cluster_tool_ids
        .iter()
        .min_by_key(|tool_id| canonical_preference(tool_id))
        .cloned()
        .expect("canonical tool id")
}

fn canonical_preference(tool_id: &str) -> (usize, usize, String) {
    let hyphen_count = tool_id.matches('-').count();
    let underscore_count = tool_id.matches('_').count();
    let separator_penalty = hyphen_count + underscore_count;
    (separator_penalty, hyphen_count, tool_id.to_ascii_lowercase())
}

fn render_tool_id_normalization_tsv(rows: &[ToolIdNormalizationRow]) -> String {
    let mut rendered =
        String::from("normalized_tool_id\tcanonical_tool_id\talias_tool_ids\tdomains\treason\n");
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.normalized_tool_id),
            sanitize_tsv(&row.canonical_tool_id),
            sanitize_tsv(&row.alias_tool_ids.join(",")),
            sanitize_tsv(&row.domains.join(",")),
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
        normalize_tool_id, render_tool_id_normalization, DEFAULT_TOOL_ID_NORMALIZATION_PATH,
        TOOL_ID_NORMALIZATION_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn normalize_tool_id_folds_separator_aliases() {
        assert_eq!(normalize_tool_id("bowtie2-build"), "bowtie2build");
        assert_eq!(normalize_tool_id("bowtie2_build"), "bowtie2build");
    }

    #[test]
    fn tool_id_normalization_report_stays_empty_without_alias_clusters() {
        let root = repo_root();
        let report =
            render_tool_id_normalization(&root, PathBuf::from(DEFAULT_TOOL_ID_NORMALIZATION_PATH))
                .expect("render tool id normalization");

        assert_eq!(report.schema_version, TOOL_ID_NORMALIZATION_SCHEMA_VERSION);
        assert_eq!(report.cluster_count, 0);
        assert!(report.ok, "report must stay empty while no benchmark alias clusters exist");
        assert!(report.rows.is_empty());
    }
}
