use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::catalog::{load_stage_admissions, ReadinessDomain};
use super::tool_serving_map::{
    render_bam_tool_serving_map, render_fastq_tool_serving_map, ToolServingMapRow,
    DEFAULT_BAM_TOOL_SERVING_MAP_PATH, DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_MISSING_BENCHMARK_PAIRS_PATH: &str =
    "target/bench-readiness/missing-benchmark-pairs.tsv";
const MISSING_BENCHMARK_PAIRS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.missing_benchmark_pairs.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MissingBenchmarkPairRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) support_status: String,
    pub(crate) registered_tool_ids: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MissingBenchmarkPairsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) missing_pair_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<MissingBenchmarkPairRow>,
}

pub(crate) fn run_render_missing_benchmark_pairs(
    args: &parse::BenchReadinessRenderMissingBenchmarkPairsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_missing_benchmark_pairs(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_MISSING_BENCHMARK_PAIRS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_missing_benchmark_pairs(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<MissingBenchmarkPairsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let fastq_map = render_fastq_tool_serving_map(
        repo_root,
        PathBuf::from(DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH),
    )?;
    let bam_map =
        render_bam_tool_serving_map(repo_root, PathBuf::from(DEFAULT_BAM_TOOL_SERVING_MAP_PATH))?;

    let registered_pairs = BTreeMap::from([
        (ReadinessDomain::Fastq, registered_pairs_by_stage(&fastq_map.rows)),
        (ReadinessDomain::Bam, registered_pairs_by_stage(&bam_map.rows)),
    ]);

    let mut rows = Vec::new();
    for domain in [ReadinessDomain::Fastq, ReadinessDomain::Bam] {
        let stage_admissions = load_stage_admissions(repo_root, domain)?;
        let registered_by_stage = registered_pairs.get(&domain).expect("registered pair map");
        for (stage_id, admissions) in stage_admissions {
            let registered_tool_ids = registered_by_stage
                .get(stage_id.as_str())
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect::<Vec<_>>();
            let registered_tool_set = registered_tool_ids.iter().cloned().collect::<BTreeSet<_>>();
            for admission in admissions {
                if registered_tool_set.contains(&admission.tool_id) {
                    continue;
                }
                rows.push(MissingBenchmarkPairRow {
                    domain: domain.as_str().to_string(),
                    stage_id: stage_id.clone(),
                    tool_id: admission.tool_id.clone(),
                    support_status: admission.support_status.clone(),
                    registered_tool_ids: registered_tool_ids.clone(),
                    reason: format!(
                        "domain-compatible pair `{}` / `{}` is admitted by governed contracts but absent from the benchmark matrix; current registered tools: {}",
                        stage_id,
                        admission.tool_id,
                        if registered_tool_ids.is_empty() {
                            "<none>".to_string()
                        } else {
                            registered_tool_ids.join(", ")
                        }
                    ),
                });
            }
        }
    }
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_missing_benchmark_pairs_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let mut domain_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
    }

    Ok(MissingBenchmarkPairsReport {
        schema_version: MISSING_BENCHMARK_PAIRS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        missing_pair_count: rows.len(),
        domain_counts,
        ok: rows.is_empty(),
        rows,
    })
}

fn registered_pairs_by_stage(rows: &[ToolServingMapRow]) -> BTreeMap<String, BTreeSet<String>> {
    let mut registered = BTreeMap::<String, BTreeSet<String>>::new();
    for row in rows {
        registered.entry(row.stage_id.clone()).or_default().insert(row.tool_id.clone());
    }
    registered
}

fn render_missing_benchmark_pairs_tsv(rows: &[MissingBenchmarkPairRow]) -> String {
    let mut rendered =
        String::from("domain\tstage_id\ttool_id\tsupport_status\tregistered_tool_ids\treason\n");
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.registered_tool_ids.join(",")),
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
        render_missing_benchmark_pairs, DEFAULT_MISSING_BENCHMARK_PAIRS_PATH,
        MISSING_BENCHMARK_PAIRS_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn missing_benchmark_pairs_report_retains_domain_contract_gaps() {
        let root = repo_root();
        let report = render_missing_benchmark_pairs(
            &root,
            PathBuf::from(DEFAULT_MISSING_BENCHMARK_PAIRS_PATH),
        )
        .expect("render missing benchmark pairs");

        assert_eq!(report.schema_version, MISSING_BENCHMARK_PAIRS_SCHEMA_VERSION);
        assert_eq!(report.missing_pair_count, 9);
        assert!(!report.ok, "missing benchmark pair report must fail while governed gaps remain");
        assert!(report.rows.iter().any(|row| {
            row.domain == "bam"
                && row.stage_id == "bam.damage"
                && row.tool_id == "addeam"
                && row.support_status == "supported"
        }));
        assert!(report.rows.iter().any(|row| {
            row.domain == "bam"
                && row.stage_id == "bam.overlap_correction"
                && row.tool_id == "samtools"
                && row.support_status == "planned"
        }));
    }
}
