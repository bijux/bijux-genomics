use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_fastq::benchmark_corpus_assignment_for_stage_tool;
use serde::Serialize;

use super::fastq_command_adapter_coverage::{
    collect_fastq_command_adapter_coverage_rows, FastqBenchmarkStatus,
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

pub(crate) const DEFAULT_FASTQ_CORPUS_ASSIGNMENT_PATH: &str =
    "benchmarks/readiness/fastq-corpus-assignment.tsv";
const FASTQ_CORPUS_ASSIGNMENT_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_corpus_assignment.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FastqCorpusAssignmentStatus {
    Assigned,
    Excluded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FastqCorpusAssignmentRow {
    pub(crate) tool_id: String,
    pub(crate) stage_id: String,
    pub(crate) benchmark_status: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) assignment_status: FastqCorpusAssignmentStatus,
    pub(crate) corpus_family_id: Option<String>,
    pub(crate) fixture_id: Option<String>,
    pub(crate) excluded_reason: Option<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqCorpusAssignmentReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) benchmark_ready_assigned_row_count: usize,
    pub(crate) benchmark_ready_excluded_row_count: usize,
    pub(crate) corpus_family_counts: BTreeMap<String, usize>,
    pub(crate) excluded_reason_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<FastqCorpusAssignmentRow>,
}

pub(crate) fn run_render_fastq_corpus_assignment(
    args: &parse::BenchReadinessRenderFastqCorpusAssignmentArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_corpus_assignment(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_CORPUS_ASSIGNMENT_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_fastq_corpus_assignment(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqCorpusAssignmentReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let stage_count = load_local_stage_inventory(repo_root, BenchLocalDomain::Fastq)?.stage_count;
    let (_, tool_count, rows) = collect_fastq_corpus_assignment_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_fastq_corpus_assignment_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let benchmark_ready_assigned_row_count = rows
        .iter()
        .filter(|row| {
            row.benchmark_status == "benchmark_ready"
                && row.assignment_status == FastqCorpusAssignmentStatus::Assigned
        })
        .count();
    let benchmark_ready_excluded_row_count =
        benchmark_ready_row_count - benchmark_ready_assigned_row_count;

    let mut corpus_family_counts = BTreeMap::<String, usize>::new();
    let mut excluded_reason_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        if let Some(corpus_family_id) = &row.corpus_family_id {
            *corpus_family_counts.entry(corpus_family_id.clone()).or_default() += 1;
        }
        if let Some(excluded_reason) = &row.excluded_reason {
            *excluded_reason_counts.entry(excluded_reason.clone()).or_default() += 1;
        }
    }

    Ok(FastqCorpusAssignmentReport {
        schema_version: FASTQ_CORPUS_ASSIGNMENT_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        tool_count,
        row_count: rows.len(),
        benchmark_ready_row_count,
        benchmark_ready_assigned_row_count,
        benchmark_ready_excluded_row_count,
        corpus_family_counts,
        excluded_reason_counts,
        rows,
    })
}

pub(crate) fn collect_fastq_corpus_assignment_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<FastqCorpusAssignmentRow>)> {
    let compatibility_by_stage = load_fastq_stage_compatibility(repo_root)?;
    let (stage_count, tool_count, coverage_rows) =
        collect_fastq_command_adapter_coverage_rows(repo_root)?;
    let mut rows = Vec::with_capacity(coverage_rows.len());

    for row in coverage_rows {
        let stage_id = StageId::new(row.stage_id.clone());
        let tool_id = ToolId::new(row.tool_id.clone());
        let domain_assignment = benchmark_corpus_assignment_for_stage_tool(&stage_id, &tool_id)
            .ok_or_else(|| {
                anyhow!(
                    "missing FASTQ corpus assignment for `{}` / `{}`",
                    row.stage_id,
                    row.tool_id
                )
            })?;
        let stage_compatibility = compatibility_by_stage.get(&row.stage_id).ok_or_else(|| {
            anyhow!("missing FASTQ corpus compatibility for stage `{}`", row.stage_id)
        })?;

        let (assignment_status, corpus_family_id, fixture_id, excluded_reason, reason) =
            match domain_assignment {
                bijux_dna_domain_fastq::BenchmarkCorpusAssignment::Assigned {
                    family,
                    rationale,
                } => {
                    let compatibility_family = stage_compatibility
                        .corpus_family_id
                        .as_deref()
                        .ok_or_else(|| {
                            anyhow!(
                                "FASTQ stage `{}` is assigned to `{}` in the domain contract but missing corpus_family_id in local compatibility",
                                row.stage_id,
                                family.as_str()
                            )
                        })?;
                    let compatibility_fixture = stage_compatibility.fixture_id.as_deref().ok_or_else(|| {
                        anyhow!(
                            "FASTQ stage `{}` is assigned to `{}` in the domain contract but missing fixture_id in local compatibility",
                            row.stage_id,
                            family.as_str()
                        )
                    })?;
                    if compatibility_family != family.as_str() {
                        return Err(anyhow!(
                            "FASTQ stage `{}` assigns `{}` in the domain contract but `{}` in local compatibility",
                            row.stage_id,
                            family.as_str(),
                            compatibility_family
                        ));
                    }
                    (
                        FastqCorpusAssignmentStatus::Assigned,
                        Some(family.as_str().to_string()),
                        Some(compatibility_fixture.to_string()),
                        None,
                        format!(
                            "row `{}` / `{}` is {} and maps to `{}` via fixture `{}`: {}",
                            row.stage_id,
                            row.tool_id,
                            benchmark_status_label(row.benchmark_status),
                            family.as_str(),
                            compatibility_fixture,
                            rationale
                        ),
                    )
                }
                bijux_dna_domain_fastq::BenchmarkCorpusAssignment::Excluded {
                    reason_code,
                    rationale,
                } => {
                    if stage_compatibility.corpus_family_id.is_some()
                        || stage_compatibility.fixture_id.is_some()
                    {
                        return Err(anyhow!(
                            "FASTQ stage `{}` is excluded in the domain contract but still fixture-backed in local compatibility",
                            row.stage_id
                        ));
                    }
                    (
                        FastqCorpusAssignmentStatus::Excluded,
                        None,
                        None,
                        Some(reason_code.to_string()),
                        format!(
                            "row `{}` / `{}` is {} and remains excluded from governed corpus assignment: {}",
                            row.stage_id,
                            row.tool_id,
                            benchmark_status_label(row.benchmark_status),
                            rationale
                        ),
                    )
                }
            };

        rows.push(FastqCorpusAssignmentRow {
            tool_id: row.tool_id,
            stage_id: row.stage_id,
            benchmark_status: benchmark_status_label(row.benchmark_status).to_string(),
            support_status: row.support_status,
            adapter_status: row.adapter_status,
            parser_status: row.parser_status,
            assignment_status,
            corpus_family_id,
            fixture_id,
            excluded_reason,
            reason,
        });
    }

    rows.sort_by(|left, right| {
        left.tool_id.cmp(&right.tool_id).then_with(|| left.stage_id.cmp(&right.stage_id))
    });
    ensure_row_completeness(&rows)?;
    ensure_taxonomy_corpus_coverage(&rows)?;
    ensure_amplicon_corpus_coverage(&rows)?;
    Ok((stage_count, tool_count, rows))
}

fn load_fastq_stage_compatibility(
    repo_root: &Path,
) -> Result<BTreeMap<String, LocalCorpusStageCompatibilityEntryReport>> {
    let matrix_path = repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH);
    let report = validate_corpus_stage_compatibility_path(repo_root, &matrix_path)?;
    report
        .stages
        .into_iter()
        .filter(|stage| stage.stage_id.starts_with("fastq."))
        .map(|stage| Ok((stage.stage_id.clone(), stage)))
        .collect()
}

fn ensure_row_completeness(rows: &[FastqCorpusAssignmentRow]) -> Result<()> {
    let mut seen = BTreeSet::<(&str, &str)>::new();
    for row in rows {
        if !seen.insert((&row.stage_id, &row.tool_id)) {
            return Err(anyhow!(
                "FASTQ corpus assignment report repeats row `{}` / `{}`",
                row.stage_id,
                row.tool_id
            ));
        }
        match row.assignment_status {
            FastqCorpusAssignmentStatus::Assigned => {
                if row.corpus_family_id.is_none()
                    || row.fixture_id.is_none()
                    || row.excluded_reason.is_some()
                {
                    return Err(anyhow!(
                        "FASTQ corpus assignment row `{}` / `{}` is assigned but incomplete",
                        row.stage_id,
                        row.tool_id
                    ));
                }
            }
            FastqCorpusAssignmentStatus::Excluded => {
                if row.corpus_family_id.is_some()
                    || row.fixture_id.is_some()
                    || row.excluded_reason.is_none()
                {
                    return Err(anyhow!(
                        "FASTQ corpus assignment row `{}` / `{}` is excluded but incomplete",
                        row.stage_id,
                        row.tool_id
                    ));
                }
            }
        }
    }
    Ok(())
}

fn ensure_taxonomy_corpus_coverage(rows: &[FastqCorpusAssignmentRow]) -> Result<()> {
    let taxonomy_rows =
        rows.iter().filter(|row| row.stage_id == "fastq.screen_taxonomy").collect::<Vec<_>>();
    let expected_tool_ids = ["centrifuge", "kaiju", "kraken2", "krakenuniq"];
    if taxonomy_rows.len() != expected_tool_ids.len() {
        return Err(anyhow!(
            "FASTQ taxonomy corpus assignment expected {} rows but found {}",
            expected_tool_ids.len(),
            taxonomy_rows.len()
        ));
    }
    for tool_id in expected_tool_ids {
        let row = taxonomy_rows
            .iter()
            .find(|row| row.tool_id == tool_id)
            .ok_or_else(|| anyhow!("FASTQ taxonomy corpus assignment is missing `{tool_id}`"))?;
        if row.assignment_status != FastqCorpusAssignmentStatus::Assigned
            || row.corpus_family_id.as_deref() != Some("corpus-02")
            || row.fixture_id.as_deref() != Some("corpus-02-edna-mini")
            || row.excluded_reason.is_some()
        {
            return Err(anyhow!(
                "FASTQ taxonomy corpus assignment row `{}` must remain assigned to `corpus-02` via `corpus-02-edna-mini`",
                tool_id
            ));
        }
    }
    Ok(())
}

fn ensure_amplicon_corpus_coverage(rows: &[FastqCorpusAssignmentRow]) -> Result<()> {
    let expected_rows = [
        ("cutadapt", "fastq.normalize_primers"),
        ("vsearch", "fastq.remove_chimeras"),
        ("dada2", "fastq.infer_asvs"),
        ("vsearch", "fastq.cluster_otus"),
        ("seqkit", "fastq.normalize_abundance"),
        ("seqfu", "fastq.normalize_abundance"),
    ];
    for (tool_id, stage_id) in expected_rows {
        let row = rows
            .iter()
            .find(|row| row.tool_id == tool_id && row.stage_id == stage_id)
            .ok_or_else(|| {
                anyhow!(
                    "FASTQ amplicon corpus assignment is missing `{}` / `{}`",
                    stage_id,
                    tool_id
                )
            })?;
        if row.corpus_family_id.as_deref() != Some("corpus-03")
            || row.fixture_id.as_deref() != Some("corpus-03-amplicon-mini")
            || row.excluded_reason.is_some()
        {
            return Err(anyhow!(
                "FASTQ amplicon corpus assignment row `{}` / `{}` must remain assigned to `corpus-03` via `corpus-03-amplicon-mini`",
                stage_id,
                tool_id
            ));
        }
    }
    Ok(())
}

fn benchmark_status_label(status: FastqBenchmarkStatus) -> &'static str {
    match status {
        FastqBenchmarkStatus::BenchmarkReady => "benchmark_ready",
        FastqBenchmarkStatus::NotBenchmarkReady => "not_benchmark_ready",
    }
}

fn render_fastq_corpus_assignment_tsv(rows: &[FastqCorpusAssignmentRow]) -> String {
    let mut rendered = String::from(
        "tool_id\tstage_id\tbenchmark_status\tsupport_status\tadapter_status\tparser_status\tassignment_status\tcorpus_family_id\tfixture_id\texcluded_reason\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.benchmark_status),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(assignment_status_label(row.assignment_status)),
            sanitize_tsv(row.corpus_family_id.as_deref().unwrap_or("")),
            sanitize_tsv(row.fixture_id.as_deref().unwrap_or("")),
            sanitize_tsv(row.excluded_reason.as_deref().unwrap_or("")),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn assignment_status_label(status: FastqCorpusAssignmentStatus) -> &'static str {
    match status {
        FastqCorpusAssignmentStatus::Assigned => "assigned",
        FastqCorpusAssignmentStatus::Excluded => "excluded",
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

    use super::{render_fastq_corpus_assignment, DEFAULT_FASTQ_CORPUS_ASSIGNMENT_PATH};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn fastq_corpus_assignment_reports_explicit_family_or_exclusion() {
        let report = render_fastq_corpus_assignment(
            &repo_root(),
            PathBuf::from(DEFAULT_FASTQ_CORPUS_ASSIGNMENT_PATH),
        )
        .expect("render FASTQ corpus assignment");

        assert_eq!(report.schema_version, "bijux.bench.readiness.fastq_corpus_assignment.v1");
        assert_eq!(report.stage_count, 27);
        assert!(report.benchmark_ready_row_count > 0);
        assert_eq!(report.benchmark_ready_excluded_row_count, 0);
        assert!(report
            .rows
            .iter()
            .all(|row| { row.corpus_family_id.is_some() || row.excluded_reason.is_some() }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.validate_reads"
                && row.tool_id == "fastqc"
                && row.corpus_family_id.as_deref() == Some("corpus-01")
                && row.fixture_id.as_deref() == Some("corpus-01-mini")
                && row.excluded_reason.is_none()
        }));
        let taxonomy_rows = report
            .rows
            .iter()
            .filter(|row| row.stage_id == "fastq.screen_taxonomy")
            .collect::<Vec<_>>();
        assert_eq!(taxonomy_rows.len(), 4);
        for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
            assert!(taxonomy_rows.iter().any(|row| {
                row.tool_id == tool_id
                    && row.corpus_family_id.as_deref() == Some("corpus-02")
                    && row.fixture_id.as_deref() == Some("corpus-02-edna-mini")
                    && row.excluded_reason.is_none()
            }));
        }
        for (tool_id, stage_id) in [
            ("cutadapt", "fastq.normalize_primers"),
            ("vsearch", "fastq.remove_chimeras"),
            ("dada2", "fastq.infer_asvs"),
            ("vsearch", "fastq.cluster_otus"),
            ("seqkit", "fastq.normalize_abundance"),
            ("seqfu", "fastq.normalize_abundance"),
        ] {
            assert!(report.rows.iter().any(|row| {
                row.tool_id == tool_id
                    && row.stage_id == stage_id
                    && row.corpus_family_id.as_deref() == Some("corpus-03")
                    && row.fixture_id.as_deref() == Some("corpus-03-amplicon-mini")
                    && row.excluded_reason.is_none()
            }));
        }
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.normalize_primers"
                && row.tool_id == "cutadapt"
                && row.corpus_family_id.as_deref() == Some("corpus-03")
                && row.fixture_id.as_deref() == Some("corpus-03-amplicon-mini")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.index_reference"
                && row.tool_id == "bowtie2_build"
                && row.excluded_reason.as_deref()
                    == Some("reference_index_stage_has_no_read_corpus")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.profile_overrepresented_sequences"
                && row.tool_id == "fastqc"
                && row.excluded_reason.as_deref()
                    == Some("governed_overrepresented_sequence_fixture_missing")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.report_qc"
                && row.tool_id == "multiqc"
                && row.excluded_reason.as_deref() == Some("governed_multiqc_bundle_fixture_missing")
        }));
    }
}
