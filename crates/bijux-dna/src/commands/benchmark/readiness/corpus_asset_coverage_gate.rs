use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::bam_command_adapter_coverage::{
    collect_bam_command_adapter_coverage_rows, BamBenchmarkStatus, BamCommandAdapterCoverageRow,
    BamReadinessGapKind,
};
use super::bam_corpus_assignment::{collect_bam_corpus_assignment_rows, BamCorpusAssignmentRow};
use super::fastq_command_adapter_coverage::{
    collect_fastq_command_adapter_coverage_rows, FastqBenchmarkStatus,
    FastqCommandAdapterCoverageRow, FastqReadinessGapKind,
};
use super::fastq_corpus_assignment::{
    collect_fastq_corpus_assignment_rows, FastqCorpusAssignmentRow, FastqCorpusAssignmentStatus,
};
use super::stage_tool_assets::{collect_stage_tool_asset_rows, StageToolAssetRow};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_CORPUS_ASSET_COVERAGE_GATE_PATH: &str =
    "target/bench-readiness/gate-corpus-assets-complete.json";
const CORPUS_ASSET_COVERAGE_GATE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.corpus_asset_coverage_gate.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CorpusAssetCoverageGateScope {
    BenchmarkSubmission,
    Excluded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CorpusAssetCoverageGateStatus {
    Pass,
    Fail,
    Excluded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CorpusAssignmentStatus {
    Assigned,
    Excluded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AssetAssignmentStatus {
    Assigned,
    NotRequired,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CorpusAssetCoverageGateRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) gate_scope: CorpusAssetCoverageGateScope,
    pub(crate) gate_status: CorpusAssetCoverageGateStatus,
    pub(crate) benchmark_status: String,
    pub(crate) readiness_gap: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
    pub(crate) corpus_assignment_status: CorpusAssignmentStatus,
    pub(crate) asset_assignment_status: AssetAssignmentStatus,
    pub(crate) required_asset_roles: Vec<String>,
    pub(crate) assigned_assets: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CorpusAssetCoverageGateReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) passes_gate: bool,
    pub(crate) row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) gate_row_count: usize,
    pub(crate) gate_passed_row_count: usize,
    pub(crate) gate_failed_row_count: usize,
    pub(crate) excluded_row_count: usize,
    pub(crate) benchmark_ready_asset_required_row_count: usize,
    pub(crate) benchmark_ready_asset_assigned_row_count: usize,
    pub(crate) benchmark_ready_asset_missing_row_count: usize,
    pub(crate) domain_stage_counts: BTreeMap<String, usize>,
    pub(crate) domain_tool_counts: BTreeMap<String, usize>,
    pub(crate) domain_row_counts: BTreeMap<String, usize>,
    pub(crate) gate_domain_row_counts: BTreeMap<String, usize>,
    pub(crate) excluded_readiness_gap_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<CorpusAssetCoverageGateRow>,
}

pub(crate) fn run_render_corpus_asset_coverage_gate(
    args: &parse::BenchReadinessRenderCorpusAssetCoverageGateArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_corpus_asset_coverage_gate(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CORPUS_ASSET_COVERAGE_GATE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_corpus_asset_coverage_gate(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<CorpusAssetCoverageGateReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let (domain_stage_counts, domain_tool_counts, rows) =
        collect_corpus_asset_coverage_gate_rows(repo_root)?;
    let row_count = rows.len();
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let gate_row_count = rows
        .iter()
        .filter(|row| row.gate_scope == CorpusAssetCoverageGateScope::BenchmarkSubmission)
        .count();
    let gate_passed_row_count =
        rows.iter().filter(|row| row.gate_status == CorpusAssetCoverageGateStatus::Pass).count();
    let gate_failed_row_count =
        rows.iter().filter(|row| row.gate_status == CorpusAssetCoverageGateStatus::Fail).count();
    let excluded_row_count = rows
        .iter()
        .filter(|row| row.gate_status == CorpusAssetCoverageGateStatus::Excluded)
        .count();
    let benchmark_ready_asset_required_row_count = rows
        .iter()
        .filter(|row| {
            row.gate_scope == CorpusAssetCoverageGateScope::BenchmarkSubmission
                && !row.required_asset_roles.is_empty()
        })
        .count();
    let benchmark_ready_asset_assigned_row_count = rows
        .iter()
        .filter(|row| {
            row.gate_scope == CorpusAssetCoverageGateScope::BenchmarkSubmission
                && row.asset_assignment_status == AssetAssignmentStatus::Assigned
        })
        .count();
    let benchmark_ready_asset_missing_row_count = rows
        .iter()
        .filter(|row| {
            row.gate_scope == CorpusAssetCoverageGateScope::BenchmarkSubmission
                && row.asset_assignment_status == AssetAssignmentStatus::Missing
        })
        .count();

    let mut domain_row_counts = BTreeMap::<String, usize>::new();
    let mut gate_domain_row_counts = BTreeMap::<String, usize>::new();
    let mut excluded_readiness_gap_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_row_counts.entry(row.domain.clone()).or_default() += 1;
        if row.gate_scope == CorpusAssetCoverageGateScope::BenchmarkSubmission {
            *gate_domain_row_counts.entry(row.domain.clone()).or_default() += 1;
        } else if row.readiness_gap != "none" {
            *excluded_readiness_gap_counts.entry(row.readiness_gap.clone()).or_default() += 1;
        }
    }

    let report = CorpusAssetCoverageGateReport {
        schema_version: CORPUS_ASSET_COVERAGE_GATE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        passes_gate: gate_failed_row_count == 0,
        row_count,
        benchmark_ready_row_count,
        gate_row_count,
        gate_passed_row_count,
        gate_failed_row_count,
        excluded_row_count,
        benchmark_ready_asset_required_row_count,
        benchmark_ready_asset_assigned_row_count,
        benchmark_ready_asset_missing_row_count,
        domain_stage_counts,
        domain_tool_counts,
        domain_row_counts,
        gate_domain_row_counts,
        excluded_readiness_gap_counts,
        rows,
    };
    let payload = serde_json::to_string_pretty(&report)
        .context("render corpus asset coverage gate to JSON")?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(&output_path, payload.as_bytes())?;
    Ok(report)
}

fn collect_corpus_asset_coverage_gate_rows(
    repo_root: &Path,
) -> Result<(BTreeMap<String, usize>, BTreeMap<String, usize>, Vec<CorpusAssetCoverageGateRow>)> {
    let (fastq_stage_count, fastq_tool_count, fastq_coverage_rows) =
        collect_fastq_command_adapter_coverage_rows(repo_root)?;
    let (bam_stage_count, bam_tool_count, bam_coverage_rows) =
        collect_bam_command_adapter_coverage_rows(repo_root)?;
    let (_, _, fastq_corpus_rows) = collect_fastq_corpus_assignment_rows(repo_root)?;
    let (_, _, bam_corpus_rows) = collect_bam_corpus_assignment_rows(repo_root)?;
    let asset_rows = collect_stage_tool_asset_rows(repo_root)?;

    let fastq_corpus_by_binding = fastq_corpus_rows
        .into_iter()
        .map(|row| ((row.stage_id.clone(), row.tool_id.clone()), row))
        .collect::<BTreeMap<_, _>>();
    let bam_corpus_by_binding = bam_corpus_rows
        .into_iter()
        .map(|row| ((row.stage_id.clone(), row.tool_id.clone()), row))
        .collect::<BTreeMap<_, _>>();
    let assets_by_binding = asset_rows.into_iter().fold(
        BTreeMap::<(String, String, String), Vec<StageToolAssetRow>>::new(),
        |mut acc, row| {
            acc.entry((row.domain.clone(), row.stage_id.clone(), row.tool_id.clone()))
                .or_default()
                .push(row);
            acc
        },
    );

    let mut domain_stage_counts = BTreeMap::from([
        ("fastq".to_string(), fastq_stage_count),
        ("bam".to_string(), bam_stage_count),
    ]);
    let mut domain_tool_counts = BTreeMap::from([
        ("fastq".to_string(), fastq_tool_count),
        ("bam".to_string(), bam_tool_count),
    ]);
    let mut rows = Vec::with_capacity(fastq_coverage_rows.len() + bam_coverage_rows.len());
    for row in fastq_coverage_rows {
        let corpus_row = fastq_corpus_by_binding
            .get(&(row.stage_id.clone(), row.tool_id.clone()))
            .ok_or_else(|| {
                anyhow!(
                    "missing FASTQ corpus assignment row for `{}` / `{}`",
                    row.stage_id,
                    row.tool_id
                )
            })?;
        let assigned_assets = assets_by_binding
            .get(&("fastq".to_string(), row.stage_id.clone(), row.tool_id.clone()))
            .cloned()
            .unwrap_or_default();
        rows.push(render_fastq_row(row, corpus_row, &assigned_assets));
    }
    for row in bam_coverage_rows {
        let corpus_row = bam_corpus_by_binding
            .get(&(row.stage_id.clone(), row.tool_id.clone()))
            .ok_or_else(|| {
                anyhow!(
                    "missing BAM corpus assignment row for `{}` / `{}`",
                    row.stage_id,
                    row.tool_id
                )
            })?;
        let assigned_assets = assets_by_binding
            .get(&("bam".to_string(), row.stage_id.clone(), row.tool_id.clone()))
            .cloned()
            .unwrap_or_default();
        rows.push(render_bam_row(row, corpus_row, &assigned_assets));
    }

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_gate_contract(&rows)?;
    domain_stage_counts.retain(|_, value| *value > 0);
    domain_tool_counts.retain(|_, value| *value > 0);
    Ok((domain_stage_counts, domain_tool_counts, rows))
}

fn render_fastq_row(
    coverage_row: FastqCommandAdapterCoverageRow,
    corpus_row: &FastqCorpusAssignmentRow,
    assigned_assets: &[StageToolAssetRow],
) -> CorpusAssetCoverageGateRow {
    let gate_scope = if coverage_row.benchmark_status == FastqBenchmarkStatus::BenchmarkReady {
        CorpusAssetCoverageGateScope::BenchmarkSubmission
    } else {
        CorpusAssetCoverageGateScope::Excluded
    };
    let corpus_assignment_status = match corpus_row.assignment_status {
        FastqCorpusAssignmentStatus::Assigned => CorpusAssignmentStatus::Assigned,
        FastqCorpusAssignmentStatus::Excluded => CorpusAssignmentStatus::Excluded,
    };
    let required_asset_roles =
        required_asset_roles("fastq", &coverage_row.stage_id, &coverage_row.tool_id);
    let assigned_assets = assigned_assets
        .iter()
        .map(|row| format!("{}={}", row.asset_role, row.asset_id))
        .collect::<Vec<_>>();
    let asset_assignment_status =
        resolve_asset_assignment_status(&required_asset_roles, assigned_assets.as_slice());
    let gate_status = match gate_scope {
        CorpusAssetCoverageGateScope::BenchmarkSubmission => {
            if corpus_assignment_status == CorpusAssignmentStatus::Assigned
                && asset_assignment_status != AssetAssignmentStatus::Missing
            {
                CorpusAssetCoverageGateStatus::Pass
            } else {
                CorpusAssetCoverageGateStatus::Fail
            }
        }
        CorpusAssetCoverageGateScope::Excluded => CorpusAssetCoverageGateStatus::Excluded,
    };
    let reason = render_reason(
        "fastq",
        &coverage_row.stage_id,
        &coverage_row.tool_id,
        corpus_assignment_status,
        asset_assignment_status,
        &required_asset_roles,
        &assigned_assets,
        &coverage_row.reason,
        gate_status,
    );

    CorpusAssetCoverageGateRow {
        domain: "fastq".to_string(),
        stage_id: coverage_row.stage_id,
        tool_id: coverage_row.tool_id,
        gate_scope,
        gate_status,
        benchmark_status: fastq_benchmark_status_label(coverage_row.benchmark_status).to_string(),
        readiness_gap: fastq_readiness_gap_label(coverage_row.readiness_gap).to_string(),
        support_status: coverage_row.support_status,
        adapter_status: coverage_row.adapter_status,
        parser_status: coverage_row.parser_status,
        corpus_status: coverage_row.corpus_status,
        corpus_assignment_status,
        asset_assignment_status,
        required_asset_roles,
        assigned_assets,
        reason,
    }
}

fn render_bam_row(
    coverage_row: BamCommandAdapterCoverageRow,
    corpus_row: &BamCorpusAssignmentRow,
    assigned_assets: &[StageToolAssetRow],
) -> CorpusAssetCoverageGateRow {
    let gate_scope = if coverage_row.benchmark_status == BamBenchmarkStatus::BenchmarkReady {
        CorpusAssetCoverageGateScope::BenchmarkSubmission
    } else {
        CorpusAssetCoverageGateScope::Excluded
    };
    let corpus_assignment_status = if corpus_row.fixture_id.trim().is_empty() {
        CorpusAssignmentStatus::Excluded
    } else {
        CorpusAssignmentStatus::Assigned
    };
    let required_asset_roles =
        required_asset_roles("bam", &coverage_row.stage_id, &coverage_row.tool_id);
    let assigned_assets = assigned_assets
        .iter()
        .map(|row| format!("{}={}", row.asset_role, row.asset_id))
        .collect::<Vec<_>>();
    let asset_assignment_status =
        resolve_asset_assignment_status(&required_asset_roles, assigned_assets.as_slice());
    let gate_status = match gate_scope {
        CorpusAssetCoverageGateScope::BenchmarkSubmission => {
            if corpus_assignment_status == CorpusAssignmentStatus::Assigned
                && asset_assignment_status != AssetAssignmentStatus::Missing
            {
                CorpusAssetCoverageGateStatus::Pass
            } else {
                CorpusAssetCoverageGateStatus::Fail
            }
        }
        CorpusAssetCoverageGateScope::Excluded => CorpusAssetCoverageGateStatus::Excluded,
    };
    let reason = render_reason(
        "bam",
        &coverage_row.stage_id,
        &coverage_row.tool_id,
        corpus_assignment_status,
        asset_assignment_status,
        &required_asset_roles,
        &assigned_assets,
        &coverage_row.reason,
        gate_status,
    );

    CorpusAssetCoverageGateRow {
        domain: "bam".to_string(),
        stage_id: coverage_row.stage_id,
        tool_id: coverage_row.tool_id,
        gate_scope,
        gate_status,
        benchmark_status: bam_benchmark_status_label(coverage_row.benchmark_status).to_string(),
        readiness_gap: bam_readiness_gap_label(coverage_row.readiness_gap).to_string(),
        support_status: coverage_row.support_status,
        adapter_status: coverage_row.adapter_status,
        parser_status: coverage_row.parser_status,
        corpus_status: coverage_row.corpus_status,
        corpus_assignment_status,
        asset_assignment_status,
        required_asset_roles,
        assigned_assets,
        reason,
    }
}

fn render_reason(
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    corpus_assignment_status: CorpusAssignmentStatus,
    asset_assignment_status: AssetAssignmentStatus,
    required_asset_roles: &[String],
    assigned_assets: &[String],
    excluded_reason: &str,
    gate_status: CorpusAssetCoverageGateStatus,
) -> String {
    match gate_status {
        CorpusAssetCoverageGateStatus::Pass => {
            if required_asset_roles.is_empty() {
                format!(
                    "row `{stage_id}` / `{tool_id}` stays inside the {domain} benchmark submission gate with governed corpus assignment and no required asset bindings"
                )
            } else {
                format!(
                    "row `{stage_id}` / `{tool_id}` stays inside the {domain} benchmark submission gate with governed corpus assignment and asset bindings `{}`",
                    assigned_assets.join(", ")
                )
            }
        }
        CorpusAssetCoverageGateStatus::Fail => {
            let mut blockers = Vec::new();
            if corpus_assignment_status != CorpusAssignmentStatus::Assigned {
                blockers.push("corpus assignment is not governed-assigned".to_string());
            }
            if asset_assignment_status == AssetAssignmentStatus::Missing {
                blockers.push(format!(
                    "required asset roles are missing: {}",
                    required_asset_roles.join(", ")
                ));
            }
            format!(
                "row `{stage_id}` / `{tool_id}` fails the {domain} corpus-asset gate because {}; assigned assets: {}",
                blockers.join("; "),
                if assigned_assets.is_empty() {
                    "none".to_string()
                } else {
                    assigned_assets.join(", ")
                }
            )
        }
        CorpusAssetCoverageGateStatus::Excluded => excluded_reason.to_string(),
    }
}

fn resolve_asset_assignment_status(
    required_asset_roles: &[String],
    assigned_assets: &[String],
) -> AssetAssignmentStatus {
    if required_asset_roles.is_empty() {
        return AssetAssignmentStatus::NotRequired;
    }
    let assigned_roles = assigned_assets
        .iter()
        .filter_map(|asset| asset.split_once('=').map(|(role, _)| role.to_string()))
        .collect::<BTreeSet<_>>();
    if required_asset_roles.iter().all(|role| assigned_roles.contains(role)) {
        AssetAssignmentStatus::Assigned
    } else {
        AssetAssignmentStatus::Missing
    }
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

fn ensure_gate_contract(rows: &[CorpusAssetCoverageGateRow]) -> Result<()> {
    if rows.iter().any(|row| row.gate_status == CorpusAssetCoverageGateStatus::Fail) {
        return Err(anyhow!(
            "corpus asset coverage gate currently contains failing benchmark rows"
        ));
    }
    ensure_row(
        rows,
        "fastq",
        "fastq.screen_taxonomy",
        "kraken2",
        AssetAssignmentStatus::Assigned,
        &["taxonomy_database_root", "database_artifact_id"],
    )?;
    ensure_row(
        rows,
        "fastq",
        "fastq.trim_reads",
        "trimmomatic",
        AssetAssignmentStatus::NotRequired,
        &[],
    )?;
    ensure_row(
        rows,
        "bam",
        "bam.kinship",
        "king",
        AssetAssignmentStatus::Assigned,
        &["reference_fasta", "reference_panel"],
    )?;
    ensure_excluded_row(rows, "fastq", "fastq.index_reference", "bowtie2_build")?;
    Ok(())
}

fn ensure_row(
    rows: &[CorpusAssetCoverageGateRow],
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    expected_asset_status: AssetAssignmentStatus,
    expected_roles: &[&str],
) -> Result<()> {
    let row = rows
        .iter()
        .find(|row| row.domain == domain && row.stage_id == stage_id && row.tool_id == tool_id)
        .ok_or_else(|| {
            anyhow!("missing corpus asset coverage row for `{domain}` / `{stage_id}` / `{tool_id}`")
        })?;
    if row.gate_scope != CorpusAssetCoverageGateScope::BenchmarkSubmission
        || row.gate_status != CorpusAssetCoverageGateStatus::Pass
        || row.corpus_assignment_status != CorpusAssignmentStatus::Assigned
        || row.asset_assignment_status != expected_asset_status
    {
        return Err(anyhow!(
            "corpus asset coverage row `{domain}` / `{stage_id}` / `{tool_id}` drifted from its governed gate contract"
        ));
    }
    let actual_roles = row.required_asset_roles.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let expected_roles = expected_roles.iter().copied().collect::<BTreeSet<_>>();
    if actual_roles != expected_roles {
        return Err(anyhow!(
            "corpus asset coverage row `{domain}` / `{stage_id}` / `{tool_id}` drifted its required asset roles"
        ));
    }
    Ok(())
}

fn ensure_excluded_row(
    rows: &[CorpusAssetCoverageGateRow],
    domain: &str,
    stage_id: &str,
    tool_id: &str,
) -> Result<()> {
    let row = rows
        .iter()
        .find(|row| row.domain == domain && row.stage_id == stage_id && row.tool_id == tool_id)
        .ok_or_else(|| anyhow!("missing excluded corpus asset coverage row for `{domain}` / `{stage_id}` / `{tool_id}`"))?;
    if row.gate_scope != CorpusAssetCoverageGateScope::Excluded
        || row.gate_status != CorpusAssetCoverageGateStatus::Excluded
    {
        return Err(anyhow!(
            "corpus asset coverage row `{domain}` / `{stage_id}` / `{tool_id}` must stay excluded from the benchmark submission gate"
        ));
    }
    Ok(())
}

fn fastq_benchmark_status_label(status: FastqBenchmarkStatus) -> &'static str {
    match status {
        FastqBenchmarkStatus::BenchmarkReady => "benchmark_ready",
        FastqBenchmarkStatus::NotBenchmarkReady => "not_benchmark_ready",
    }
}

fn bam_benchmark_status_label(status: BamBenchmarkStatus) -> &'static str {
    match status {
        BamBenchmarkStatus::BenchmarkReady => "benchmark_ready",
        BamBenchmarkStatus::NotBenchmarkReady => "not_benchmark_ready",
    }
}

fn fastq_readiness_gap_label(gap: FastqReadinessGapKind) -> &'static str {
    match gap {
        FastqReadinessGapKind::None => "none",
        FastqReadinessGapKind::Corpus => "corpus",
        FastqReadinessGapKind::Parser => "parser",
        FastqReadinessGapKind::Adapter => "adapter",
        FastqReadinessGapKind::Support => "support",
    }
}

fn bam_readiness_gap_label(gap: BamReadinessGapKind) -> &'static str {
    match gap {
        BamReadinessGapKind::None => "none",
        BamReadinessGapKind::Corpus => "corpus",
        BamReadinessGapKind::Parser => "parser",
        BamReadinessGapKind::Adapter => "adapter",
        BamReadinessGapKind::Support => "support",
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_corpus_asset_coverage_gate, AssetAssignmentStatus, CorpusAssetCoverageGateScope,
        CorpusAssetCoverageGateStatus, CORPUS_ASSET_COVERAGE_GATE_SCHEMA_VERSION,
        DEFAULT_CORPUS_ASSET_COVERAGE_GATE_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn corpus_asset_coverage_gate_reports_complete_benchmark_rows() {
        let root = repo_root();
        let report = render_corpus_asset_coverage_gate(
            &root,
            PathBuf::from(DEFAULT_CORPUS_ASSET_COVERAGE_GATE_PATH),
        )
        .expect("render corpus asset coverage gate");

        assert_eq!(report.schema_version, CORPUS_ASSET_COVERAGE_GATE_SCHEMA_VERSION);
        assert!(report.passes_gate);
        assert_eq!(report.row_count, 123);
        assert_eq!(report.benchmark_ready_row_count, 112);
        assert_eq!(report.gate_row_count, 112);
        assert_eq!(report.gate_passed_row_count, 112);
        assert_eq!(report.gate_failed_row_count, 0);
        assert_eq!(report.excluded_row_count, 11);
        assert_eq!(report.benchmark_ready_asset_required_row_count, 18);
        assert_eq!(report.benchmark_ready_asset_assigned_row_count, 18);
        assert_eq!(report.benchmark_ready_asset_missing_row_count, 0);
        assert!(report
            .rows
            .iter()
            .all(|row| row.gate_status != CorpusAssetCoverageGateStatus::Fail));
        assert!(report.rows.iter().any(|row| {
            row.domain == "fastq"
                && row.stage_id == "fastq.screen_taxonomy"
                && row.tool_id == "kraken2"
                && row.gate_scope == CorpusAssetCoverageGateScope::BenchmarkSubmission
                && row.asset_assignment_status == AssetAssignmentStatus::Assigned
                && row.required_asset_roles
                    == vec![
                        "taxonomy_database_root".to_string(),
                        "database_artifact_id".to_string(),
                    ]
        }));
        assert!(report.rows.iter().any(|row| {
            row.domain == "bam"
                && row.stage_id == "bam.kinship"
                && row.tool_id == "king"
                && row.gate_scope == CorpusAssetCoverageGateScope::BenchmarkSubmission
                && row.asset_assignment_status == AssetAssignmentStatus::Assigned
        }));
        assert!(report.rows.iter().any(|row| {
            row.domain == "fastq"
                && row.stage_id == "fastq.trim_reads"
                && row.tool_id == "trimmomatic"
                && row.gate_scope == CorpusAssetCoverageGateScope::BenchmarkSubmission
                && row.asset_assignment_status == AssetAssignmentStatus::NotRequired
        }));
    }
}
