use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::bam_command_adapter_coverage::{
    collect_bam_command_adapter_coverage_rows, BamCommandAdapterCoverageRow,
};
use super::fastq_command_adapter_coverage::{
    collect_fastq_command_adapter_coverage_rows, FastqCommandAdapterCoverageRow,
};
use super::stage_tool_assets::collect_stage_tool_asset_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_PAIR_READINESS_PATH: &str = "target/bench-readiness/pair-readiness.tsv";
const PAIR_READINESS_SCHEMA_VERSION: &str = "bijux.bench.readiness.pair_readiness.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PairReadinessGap {
    None,
    Asset,
    Corpus,
    Parser,
    Adapter,
    Support,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PairAssetStatus {
    Assigned,
    Missing,
    NotRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct PairReadinessRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) benchmark_status: String,
    pub(crate) readiness_gap: PairReadinessGap,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
    pub(crate) asset_status: PairAssetStatus,
    pub(crate) required_asset_roles: Vec<String>,
    pub(crate) assigned_asset_roles: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PairReadinessReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) not_benchmark_ready_row_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) readiness_gap_counts: BTreeMap<String, usize>,
    pub(crate) asset_status_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<PairReadinessRow>,
}

pub(crate) fn run_render_pair_readiness(
    args: &parse::BenchReadinessRenderPairReadinessArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_pair_readiness(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_PAIR_READINESS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_pair_readiness(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<PairReadinessReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_pair_readiness_rows(repo_root)?;
    let row_count = rows.len();
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let not_benchmark_ready_row_count = row_count.saturating_sub(benchmark_ready_row_count);
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut readiness_gap_counts = BTreeMap::<String, usize>::new();
    let mut asset_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *readiness_gap_counts
            .entry(pair_readiness_gap_label(row.readiness_gap).to_string())
            .or_default() += 1;
        *asset_status_counts
            .entry(pair_asset_status_label(row.asset_status).to_string())
            .or_default() += 1;
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_pair_readiness_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(PairReadinessReport {
        schema_version: PAIR_READINESS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count,
        benchmark_ready_row_count,
        not_benchmark_ready_row_count,
        domain_counts,
        readiness_gap_counts,
        asset_status_counts,
        rows,
    })
}

fn collect_pair_readiness_rows(repo_root: &Path) -> Result<Vec<PairReadinessRow>> {
    let (_, _, fastq_rows) = collect_fastq_command_adapter_coverage_rows(repo_root)?;
    let (_, _, bam_rows) = collect_bam_command_adapter_coverage_rows(repo_root)?;
    let asset_rows = collect_stage_tool_asset_rows(repo_root)?;
    let asset_roles_by_binding = asset_rows.into_iter().fold(
        BTreeMap::<(String, String, String), Vec<String>>::new(),
        |mut acc, row| {
            acc.entry((row.domain.clone(), row.stage_id.clone(), row.tool_id.clone()))
                .or_default()
                .push(row.asset_role);
            acc
        },
    );

    let mut rows = fastq_rows
        .into_iter()
        .map(|row| render_fastq_pair_readiness_row(row, &asset_roles_by_binding))
        .chain(
            bam_rows
                .into_iter()
                .map(|row| render_bam_pair_readiness_row(row, &asset_roles_by_binding)),
        )
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_pair_readiness_contract(&rows)?;
    Ok(rows)
}

fn render_fastq_pair_readiness_row(
    row: FastqCommandAdapterCoverageRow,
    asset_roles_by_binding: &BTreeMap<(String, String, String), Vec<String>>,
) -> PairReadinessRow {
    let required_asset_roles = required_asset_roles("fastq", &row.stage_id, &row.tool_id);
    let assigned_asset_roles =
        assigned_asset_roles(asset_roles_by_binding, "fastq", &row.stage_id, &row.tool_id);
    let support_ready = matches!(
        row.support_status.as_str(),
        "governed_execution" | "governed_benchmark_cohort" | "observer_specialized_benchmark"
    );
    let adapter_ready = matches!(row.adapter_status.as_str(), "runnable" | "plannable");
    let parser_ready = row.parser_status != "not_normalized";
    let corpus_ready = row.corpus_status.starts_with("fixture:");
    let asset_status = resolve_asset_status(&required_asset_roles, &assigned_asset_roles);
    let readiness_gap = resolve_pair_readiness_gap(
        support_ready,
        adapter_ready,
        parser_ready,
        corpus_ready,
        asset_status,
    );
    let benchmark_status = if readiness_gap == PairReadinessGap::None {
        "benchmark_ready"
    } else {
        "not_benchmark_ready"
    };
    let reason = build_pair_reason(
        "fastq",
        &row.stage_id,
        &row.tool_id,
        readiness_gap,
        &row.support_status,
        &row.adapter_status,
        &row.parser_status,
        &row.corpus_status,
        asset_status,
        &required_asset_roles,
        &assigned_asset_roles,
    );

    PairReadinessRow {
        domain: "fastq".to_string(),
        stage_id: row.stage_id,
        tool_id: row.tool_id,
        benchmark_status: benchmark_status.to_string(),
        readiness_gap,
        support_status: row.support_status,
        adapter_status: row.adapter_status,
        parser_status: row.parser_status,
        corpus_status: row.corpus_status,
        asset_status,
        required_asset_roles,
        assigned_asset_roles,
        reason,
    }
}

fn render_bam_pair_readiness_row(
    row: BamCommandAdapterCoverageRow,
    asset_roles_by_binding: &BTreeMap<(String, String, String), Vec<String>>,
) -> PairReadinessRow {
    let required_asset_roles = required_asset_roles("bam", &row.stage_id, &row.tool_id);
    let assigned_asset_roles =
        assigned_asset_roles(asset_roles_by_binding, "bam", &row.stage_id, &row.tool_id);
    let support_ready = row.support_status == "supported";
    let adapter_ready = matches!(row.adapter_status.as_str(), "runnable" | "plannable");
    let parser_ready = row.parser_status == "parser_fixture_validated";
    let corpus_ready = row.corpus_status.starts_with("fixture:");
    let asset_status = resolve_asset_status(&required_asset_roles, &assigned_asset_roles);
    let readiness_gap = resolve_pair_readiness_gap(
        support_ready,
        adapter_ready,
        parser_ready,
        corpus_ready,
        asset_status,
    );
    let benchmark_status = if readiness_gap == PairReadinessGap::None {
        "benchmark_ready"
    } else {
        "not_benchmark_ready"
    };
    let reason = build_pair_reason(
        "bam",
        &row.stage_id,
        &row.tool_id,
        readiness_gap,
        &row.support_status,
        &row.adapter_status,
        &row.parser_status,
        &row.corpus_status,
        asset_status,
        &required_asset_roles,
        &assigned_asset_roles,
    );

    PairReadinessRow {
        domain: "bam".to_string(),
        stage_id: row.stage_id,
        tool_id: row.tool_id,
        benchmark_status: benchmark_status.to_string(),
        readiness_gap,
        support_status: row.support_status,
        adapter_status: row.adapter_status,
        parser_status: row.parser_status,
        corpus_status: row.corpus_status,
        asset_status,
        required_asset_roles,
        assigned_asset_roles,
        reason,
    }
}

fn resolve_pair_readiness_gap(
    support_ready: bool,
    adapter_ready: bool,
    parser_ready: bool,
    corpus_ready: bool,
    asset_status: PairAssetStatus,
) -> PairReadinessGap {
    if !support_ready {
        PairReadinessGap::Support
    } else if !adapter_ready {
        PairReadinessGap::Adapter
    } else if !parser_ready {
        PairReadinessGap::Parser
    } else if !corpus_ready {
        PairReadinessGap::Corpus
    } else if asset_status == PairAssetStatus::Missing {
        PairReadinessGap::Asset
    } else {
        PairReadinessGap::None
    }
}

fn resolve_asset_status(
    required_asset_roles: &[String],
    assigned_asset_roles: &[String],
) -> PairAssetStatus {
    if required_asset_roles.is_empty() {
        return PairAssetStatus::NotRequired;
    }
    let assigned_roles = assigned_asset_roles.iter().cloned().collect::<BTreeSet<_>>();
    if required_asset_roles.iter().all(|role| assigned_roles.contains(role)) {
        PairAssetStatus::Assigned
    } else {
        PairAssetStatus::Missing
    }
}

fn assigned_asset_roles(
    asset_roles_by_binding: &BTreeMap<(String, String, String), Vec<String>>,
    domain: &str,
    stage_id: &str,
    tool_id: &str,
) -> Vec<String> {
    let mut roles = asset_roles_by_binding
        .get(&(domain.to_string(), stage_id.to_string(), tool_id.to_string()))
        .cloned()
        .unwrap_or_default();
    roles.sort();
    roles.dedup();
    roles
}

fn required_asset_roles(domain: &str, stage_id: &str, tool_id: &str) -> Vec<String> {
    let roles: &[&str] = match (domain, stage_id, tool_id) {
        ("fastq", "fastq.screen_taxonomy", "centrifuge")
        | ("fastq", "fastq.screen_taxonomy", "kaiju")
        | ("fastq", "fastq.screen_taxonomy", "kraken2")
        | ("fastq", "fastq.screen_taxonomy", "krakenuniq") => {
            &["taxonomy_database_root", "database_artifact_id"]
        }
        ("fastq", "fastq.deplete_host", "bowtie2")
        | ("fastq", "fastq.deplete_reference_contaminants", "bowtie2") => {
            &["reference_catalog_id", "reference_index_artifact_id"]
        }
        ("fastq", "fastq.deplete_rrna", "sortmerna") => &["rrna_reference", "database_artifact_id"],
        ("fastq", "fastq.index_reference", "bowtie2_build") => {
            &["reference_fasta", "reference_index_output"]
        }
        ("bam", "bam.contamination", "contammix")
        | ("bam", "bam.contamination", "schmutzi")
        | ("bam", "bam.contamination", "verifybamid2")
        | ("bam", "bam.haplogroups", "yleaf")
        | ("bam", "bam.kinship", "angsd")
        | ("bam", "bam.kinship", "king") => &["reference_fasta", "reference_panel"],
        ("bam", "bam.sex", "angsd") | ("bam", "bam.sex", "rxy") | ("bam", "bam.sex", "yleaf") => {
            &["reference_fasta"]
        }
        ("bam", "bam.genotyping", "angsd") => &["reference_fasta", "sites_vcf", "regions"],
        ("bam", "bam.recalibration", "gatk") => &["reference_fasta", "known_sites"],
        _ => &[],
    };
    roles.iter().map(|role| (*role).to_string()).collect()
}

#[allow(clippy::too_many_arguments)]
fn build_pair_reason(
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    readiness_gap: PairReadinessGap,
    support_status: &str,
    adapter_status: &str,
    parser_status: &str,
    corpus_status: &str,
    asset_status: PairAssetStatus,
    required_asset_roles: &[String],
    assigned_asset_roles: &[String],
) -> String {
    match readiness_gap {
        PairReadinessGap::None => format!(
            "row `{domain}` / `{stage_id}` / `{tool_id}` is benchmark_ready with `{adapter_status}` adapter status, `{parser_status}` parser status, `{corpus_status}` corpus status, and `{}` asset status",
            pair_asset_status_label(asset_status)
        ),
        PairReadinessGap::Support => format!(
            "row `{domain}` / `{stage_id}` / `{tool_id}` is blocked by benchmark support status `{support_status}`"
        ),
        PairReadinessGap::Adapter => format!(
            "row `{domain}` / `{stage_id}` / `{tool_id}` is blocked by adapter status `{adapter_status}`"
        ),
        PairReadinessGap::Parser => format!(
            "row `{domain}` / `{stage_id}` / `{tool_id}` is blocked by parser status `{parser_status}`"
        ),
        PairReadinessGap::Corpus => format!(
            "row `{domain}` / `{stage_id}` / `{tool_id}` is blocked by corpus status `{corpus_status}`"
        ),
        PairReadinessGap::Asset => format!(
            "row `{domain}` / `{stage_id}` / `{tool_id}` is blocked by asset status `{}`; required roles: {}; assigned roles: {}",
            pair_asset_status_label(asset_status),
            required_asset_roles.join(", "),
            if assigned_asset_roles.is_empty() {
                "<none>".to_string()
            } else {
                assigned_asset_roles.join(", ")
            }
        ),
    }
}

fn ensure_pair_readiness_contract(rows: &[PairReadinessRow]) -> Result<()> {
    if rows.len() != 123 {
        return Err(anyhow!(
            "pair readiness report must retain exactly 123 FASTQ/BAM rows, found {}",
            rows.len()
        ));
    }
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    if benchmark_ready_row_count != 112 {
        return Err(anyhow!(
            "pair readiness report must retain exactly 112 benchmark_ready rows, found {}",
            benchmark_ready_row_count
        ));
    }
    ensure_row(
        rows,
        "fastq",
        "fastq.screen_taxonomy",
        "kraken2",
        PairReadinessGap::None,
        PairAssetStatus::Assigned,
        "fixture:corpus-02-edna-mini",
    )?;
    ensure_row(
        rows,
        "fastq",
        "fastq.trim_reads",
        "trimmomatic",
        PairReadinessGap::None,
        PairAssetStatus::NotRequired,
        "fixture:corpus-01-mini",
    )?;
    ensure_row(
        rows,
        "bam",
        "bam.kinship",
        "king",
        PairReadinessGap::None,
        PairAssetStatus::Assigned,
        "fixture:corpus-01-kinship-mini",
    )?;
    ensure_row(
        rows,
        "fastq",
        "fastq.index_reference",
        "bowtie2_build",
        PairReadinessGap::Corpus,
        PairAssetStatus::Assigned,
        "planner_only",
    )?;
    Ok(())
}

fn ensure_row(
    rows: &[PairReadinessRow],
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    expected_gap: PairReadinessGap,
    expected_asset_status: PairAssetStatus,
    expected_corpus_status: &str,
) -> Result<()> {
    let row = rows
        .iter()
        .find(|row| row.domain == domain && row.stage_id == stage_id && row.tool_id == tool_id)
        .ok_or_else(|| {
            anyhow!("missing pair readiness row for `{domain}` / `{stage_id}` / `{tool_id}`")
        })?;
    if row.readiness_gap != expected_gap
        || row.asset_status != expected_asset_status
        || row.corpus_status != expected_corpus_status
    {
        return Err(anyhow!(
            "pair readiness row `{domain}` / `{stage_id}` / `{tool_id}` drifted from its governed contract: gap=`{}`, asset_status=`{}`, corpus_status=`{}`",
            pair_readiness_gap_label(row.readiness_gap),
            pair_asset_status_label(row.asset_status),
            row.corpus_status,
        ));
    }
    Ok(())
}

fn render_pair_readiness_tsv(rows: &[PairReadinessRow]) -> String {
    let mut rendered = String::from(
        "domain\tstage_id\ttool_id\tbenchmark_status\treadiness_gap\tsupport_status\tadapter_status\tparser_status\tcorpus_status\tasset_status\trequired_asset_roles\tassigned_asset_roles\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.benchmark_status),
            sanitize_tsv(pair_readiness_gap_label(row.readiness_gap)),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.corpus_status),
            sanitize_tsv(pair_asset_status_label(row.asset_status)),
            sanitize_tsv(&row.required_asset_roles.join(",")),
            sanitize_tsv(&row.assigned_asset_roles.join(",")),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn pair_readiness_gap_label(gap: PairReadinessGap) -> &'static str {
    match gap {
        PairReadinessGap::None => "none",
        PairReadinessGap::Asset => "asset",
        PairReadinessGap::Corpus => "corpus",
        PairReadinessGap::Parser => "parser",
        PairReadinessGap::Adapter => "adapter",
        PairReadinessGap::Support => "support",
    }
}

fn pair_asset_status_label(status: PairAssetStatus) -> &'static str {
    match status {
        PairAssetStatus::Assigned => "assigned",
        PairAssetStatus::Missing => "missing",
        PairAssetStatus::NotRequired => "not_required",
    }
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
        render_pair_readiness, resolve_pair_readiness_gap, PairAssetStatus, PairReadinessGap,
        DEFAULT_PAIR_READINESS_PATH, PAIR_READINESS_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn pair_readiness_report_tracks_governed_gap_columns() {
        let root = repo_root();
        let report = render_pair_readiness(&root, PathBuf::from(DEFAULT_PAIR_READINESS_PATH))
            .expect("render pair readiness");

        assert_eq!(report.schema_version, PAIR_READINESS_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_PAIR_READINESS_PATH);
        assert_eq!(report.row_count, 123);
        assert_eq!(report.benchmark_ready_row_count, 112);
        assert_eq!(report.not_benchmark_ready_row_count, 11);
        assert_eq!(report.domain_counts.get("fastq").copied(), Some(74));
        assert_eq!(report.domain_counts.get("bam").copied(), Some(49));
        assert_eq!(report.asset_status_counts.get("assigned").copied(), Some(19));
        assert!(report.rows.iter().any(|row| {
            row.domain == "fastq"
                && row.stage_id == "fastq.screen_taxonomy"
                && row.tool_id == "kraken2"
                && row.adapter_status == "runnable"
                && row.parser_status == "benchmark_normalized"
                && row.corpus_status == "fixture:corpus-02-edna-mini"
                && row.asset_status == PairAssetStatus::Assigned
                && row.readiness_gap == PairReadinessGap::None
        }));
        assert!(report.rows.iter().any(|row| {
            row.domain == "bam"
                && row.stage_id == "bam.kinship"
                && row.tool_id == "king"
                && row.asset_status == PairAssetStatus::Assigned
                && row.readiness_gap == PairReadinessGap::None
        }));
        assert!(report.rows.iter().any(|row| {
            row.domain == "fastq"
                && row.stage_id == "fastq.index_reference"
                && row.tool_id == "bowtie2_build"
                && row.corpus_status == "planner_only"
                && row.asset_status == PairAssetStatus::Assigned
                && row.readiness_gap == PairReadinessGap::Corpus
        }));
    }

    #[test]
    fn pair_readiness_reports_asset_as_exact_missing_component() {
        let gap = resolve_pair_readiness_gap(true, true, true, true, PairAssetStatus::Missing);
        assert_eq!(gap, PairReadinessGap::Asset);
    }
}
