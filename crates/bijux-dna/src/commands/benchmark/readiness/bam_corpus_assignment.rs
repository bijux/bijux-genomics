use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_bam::benchmark_corpus_assignment_for_stage_tool;
use serde::Serialize;

use super::bam_command_adapter_coverage::{
    collect_bam_command_adapter_coverage_rows, BamBenchmarkStatus,
};
use crate::commands::benchmark::local_corpus_stage_compatibility::{
    validate_corpus_stage_compatibility_path, LocalCorpusStageCompatibilityEntryReport,
    DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH,
};
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_CORPUS_ASSIGNMENT_PATH: &str =
    "target/bench-readiness/bam-corpus-assignment.tsv";
const BAM_CORPUS_ASSIGNMENT_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_corpus_assignment.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BamCorpusAssignmentRow {
    pub(crate) tool_id: String,
    pub(crate) stage_id: String,
    pub(crate) benchmark_status: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_family_id: String,
    pub(crate) fixture_id: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamCorpusAssignmentReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) corpus_family_counts: BTreeMap<String, usize>,
    pub(crate) fixture_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<BamCorpusAssignmentRow>,
}

pub(crate) fn run_render_bam_corpus_assignment(
    args: &parse::BenchReadinessRenderBamCorpusAssignmentArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_corpus_assignment(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_CORPUS_ASSIGNMENT_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_corpus_assignment(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamCorpusAssignmentReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let stage_count = load_local_stage_inventory(repo_root, BenchLocalDomain::Bam)?.stage_count;
    let (_, tool_count, rows) = collect_bam_corpus_assignment_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_bam_corpus_assignment_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let mut corpus_family_counts = BTreeMap::<String, usize>::new();
    let mut fixture_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *corpus_family_counts.entry(row.corpus_family_id.clone()).or_default() += 1;
        *fixture_counts.entry(row.fixture_id.clone()).or_default() += 1;
    }

    Ok(BamCorpusAssignmentReport {
        schema_version: BAM_CORPUS_ASSIGNMENT_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        tool_count,
        row_count: rows.len(),
        benchmark_ready_row_count,
        corpus_family_counts,
        fixture_counts,
        rows,
    })
}

fn collect_bam_corpus_assignment_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<BamCorpusAssignmentRow>)> {
    let compatibility_by_stage = load_bam_stage_compatibility(repo_root)?;
    let (stage_count, tool_count, coverage_rows) = collect_bam_command_adapter_coverage_rows(repo_root)?;
    let mut rows = Vec::with_capacity(coverage_rows.len());

    for row in coverage_rows {
        let stage_id = StageId::new(row.stage_id.clone());
        let tool_id = ToolId::new(row.tool_id.clone());
        let domain_assignment = benchmark_corpus_assignment_for_stage_tool(&stage_id, &tool_id)
            .ok_or_else(|| {
                anyhow!(
                    "missing BAM corpus assignment for `{}` / `{}`",
                    row.stage_id,
                    row.tool_id
                )
            })?;
        let stage_compatibility = compatibility_by_stage.get(&row.stage_id).ok_or_else(|| {
            anyhow!("missing BAM corpus compatibility for stage `{}`", row.stage_id)
        })?;
        let compatibility_family = stage_compatibility.corpus_family_id.as_deref().ok_or_else(|| {
            anyhow!("BAM stage `{}` is missing corpus_family_id in local compatibility", row.stage_id)
        })?;
        let fixture_id = stage_compatibility.fixture_id.as_deref().ok_or_else(|| {
            anyhow!("BAM stage `{}` is missing fixture_id in local compatibility", row.stage_id)
        })?;
        let assigned_family = domain_assignment.assigned_family();
        if compatibility_family != assigned_family.as_str() {
            return Err(anyhow!(
                "BAM stage `{}` assigns `{}` in the domain contract but `{}` in local compatibility",
                row.stage_id,
                assigned_family.as_str(),
                compatibility_family
            ));
        }

        rows.push(BamCorpusAssignmentRow {
            tool_id: row.tool_id,
            stage_id: row.stage_id,
            benchmark_status: benchmark_status_label(row.benchmark_status).to_string(),
            support_status: row.support_status,
            adapter_status: row.adapter_status,
            parser_status: row.parser_status,
            corpus_family_id: compatibility_family.to_string(),
            fixture_id: fixture_id.to_string(),
            reason: format!(
                "row `{}` / `{}` is {} and maps to `{}` via fixture `{}`: {}",
                stage_id.as_str(),
                tool_id.as_str(),
                benchmark_status_label(row.benchmark_status),
                compatibility_family,
                fixture_id,
                domain_assignment.rationale()
            ),
        });
    }

    rows.sort_by(|left, right| {
        left.tool_id.cmp(&right.tool_id).then_with(|| left.stage_id.cmp(&right.stage_id))
    });
    ensure_row_completeness(&rows)?;
    Ok((stage_count, tool_count, rows))
}

fn load_bam_stage_compatibility(
    repo_root: &Path,
) -> Result<BTreeMap<String, LocalCorpusStageCompatibilityEntryReport>> {
    let matrix_path = repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH);
    let report = validate_corpus_stage_compatibility_path(repo_root, &matrix_path)?;
    report
        .stages
        .into_iter()
        .filter(|stage| stage.stage_id.starts_with("bam."))
        .map(|stage| Ok((stage.stage_id.clone(), stage)))
        .collect()
}

fn ensure_row_completeness(rows: &[BamCorpusAssignmentRow]) -> Result<()> {
    let mut seen = BTreeSet::<(&str, &str)>::new();
    for row in rows {
        if !seen.insert((&row.stage_id, &row.tool_id)) {
            return Err(anyhow!(
                "BAM corpus assignment report repeats row `{}` / `{}`",
                row.stage_id,
                row.tool_id
            ));
        }
    }
    Ok(())
}

fn benchmark_status_label(status: BamBenchmarkStatus) -> &'static str {
    match status {
        BamBenchmarkStatus::BenchmarkReady => "benchmark_ready",
        BamBenchmarkStatus::NotBenchmarkReady => "not_benchmark_ready",
    }
}

fn render_bam_corpus_assignment_tsv(rows: &[BamCorpusAssignmentRow]) -> String {
    let mut rendered = String::from(
        "tool_id\tstage_id\tbenchmark_status\tsupport_status\tadapter_status\tparser_status\tcorpus_family_id\tfixture_id\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.benchmark_status),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.corpus_family_id),
            sanitize_tsv(&row.fixture_id),
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

    use super::{render_bam_corpus_assignment, DEFAULT_BAM_CORPUS_ASSIGNMENT_PATH};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn bam_corpus_assignment_reports_precise_bam_fixture_routing() {
        let report = render_bam_corpus_assignment(
            &repo_root(),
            PathBuf::from(DEFAULT_BAM_CORPUS_ASSIGNMENT_PATH),
        )
        .expect("render BAM corpus assignment");

        assert_eq!(report.schema_version, "bijux.bench.readiness.bam_corpus_assignment.v1");
        assert_eq!(report.stage_count, 24);
        assert!(report.row_count > 0);
        assert_eq!(report.corpus_family_counts.get("corpus-01"), Some(&2));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.authenticity"
                && row.tool_id == "authenticct"
                && row.corpus_family_id == "corpus-01-adna-bam"
                && row.fixture_id == "corpus-01-adna-damage-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.genotyping"
                && row.tool_id == "angsd"
                && row.corpus_family_id == "corpus-01-genotyping"
                && row.fixture_id == "corpus-01-genotyping-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.kinship"
                && row.tool_id == "king"
                && row.corpus_family_id == "corpus-01-kinship"
                && row.fixture_id == "corpus-01-kinship-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.qc_pre"
                && row.tool_id == "samtools"
                && row.corpus_family_id == "corpus-01-bam"
                && row.fixture_id == "corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.align"
                && row.tool_id == "bwa"
                && row.corpus_family_id == "corpus-01"
                && row.fixture_id == "corpus-01-mini"
        }));
    }
}
