use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::bam_corpus_assignment::collect_bam_corpus_assignment_rows;
use super::fastq_corpus_assignment::{
    collect_fastq_corpus_assignment_rows, FastqCorpusAssignmentStatus,
};
use super::stage_centric_report::{collect_stage_centric_stage_reports, StageCentricStageReport};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_CORPUS_CENTRIC_REPORT_PATH: &str =
    "benchmarks/readiness/corpus-centric-report.md";
const CORPUS_CENTRIC_REPORT_SCHEMA_VERSION: &str = "bijux.bench.readiness.corpus_centric_report.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CorpusCentricStageRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) canonical_stage_rank: usize,
    pub(crate) fixture_ids: Vec<String>,
    pub(crate) report_section_id: String,
    pub(crate) report_section_title: String,
    pub(crate) summary_table_id: String,
    pub(crate) summary_table_title: String,
    pub(crate) comparison_contract_status: String,
    pub(crate) shared_metric_field_count: usize,
    pub(crate) shared_metric_fields: Vec<String>,
    pub(crate) tool_count: usize,
    pub(crate) benchmark_ready_tool_count: usize,
    pub(crate) blocked_tool_count: usize,
    pub(crate) tool_ids: Vec<String>,
    pub(crate) blocked_tool_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CorpusCentricCorpusReport {
    pub(crate) corpus_family_id: String,
    pub(crate) domains: Vec<String>,
    pub(crate) fixture_ids: Vec<String>,
    pub(crate) stage_count: usize,
    pub(crate) tool_row_count: usize,
    pub(crate) benchmark_ready_tool_row_count: usize,
    pub(crate) blocked_stage_count: usize,
    pub(crate) blocked_stage_ids: Vec<String>,
    pub(crate) stages: Vec<CorpusCentricStageRow>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CorpusCentricReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) corpus_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_row_count: usize,
    pub(crate) benchmark_ready_tool_row_count: usize,
    pub(crate) blocked_tool_row_count: usize,
    pub(crate) blocked_corpus_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) corpus_stage_counts: BTreeMap<String, usize>,
    pub(crate) corpora: Vec<CorpusCentricCorpusReport>,
}

pub(crate) fn run_render_corpus_centric_report(
    args: &parse::BenchReadinessRenderCorpusCentricReportArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_corpus_centric_report(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_CORPUS_CENTRIC_REPORT_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_corpus_centric_report(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<CorpusCentricReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let corpora = collect_corpus_centric_corpora(repo_root)?;
    let corpus_count = corpora.len();
    let stage_count = corpora.iter().map(|corpus| corpus.stage_count).sum::<usize>();
    let tool_row_count = corpora.iter().map(|corpus| corpus.tool_row_count).sum::<usize>();
    let benchmark_ready_tool_row_count =
        corpora.iter().map(|corpus| corpus.benchmark_ready_tool_row_count).sum::<usize>();
    let blocked_tool_row_count = tool_row_count.saturating_sub(benchmark_ready_tool_row_count);
    let blocked_corpus_count =
        corpora.iter().filter(|corpus| corpus.blocked_stage_count > 0).count();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut corpus_stage_counts = BTreeMap::<String, usize>::new();
    for corpus in &corpora {
        corpus_stage_counts.insert(corpus.corpus_family_id.clone(), corpus.stage_count);
        for stage in &corpus.stages {
            *domain_counts.entry(stage.domain.clone()).or_default() += 1;
        }
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_corpus_centric_markdown(&corpora))
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(CorpusCentricReport {
        schema_version: CORPUS_CENTRIC_REPORT_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        corpus_count,
        stage_count,
        tool_row_count,
        benchmark_ready_tool_row_count,
        blocked_tool_row_count,
        blocked_corpus_count,
        domain_counts,
        corpus_stage_counts,
        corpora,
    })
}

fn collect_corpus_centric_corpora(repo_root: &Path) -> Result<Vec<CorpusCentricCorpusReport>> {
    let stage_reports = collect_stage_centric_stage_reports(repo_root)?;
    let stage_assignments = load_stage_assignments(repo_root)?;
    let mut stages_by_corpus = BTreeMap::<String, Vec<CorpusCentricStageRow>>::new();

    for stage in stage_reports {
        let Some(assignment) =
            stage_assignments.get(&(stage.domain.clone(), stage.stage_id.clone()))
        else {
            continue;
        };
        stages_by_corpus
            .entry(assignment.corpus_family_id.clone())
            .or_default()
            .push(render_corpus_stage_row(stage, &assignment.fixture_ids));
    }

    let mut corpora = stages_by_corpus
        .into_iter()
        .map(|(corpus_family_id, mut stages)| {
            stages.sort_by(|left, right| {
                left.domain
                    .cmp(&right.domain)
                    .then_with(|| left.canonical_stage_rank.cmp(&right.canonical_stage_rank))
                    .then_with(|| left.stage_id.cmp(&right.stage_id))
            });
            let tool_row_count = stages.iter().map(|stage| stage.tool_count).sum::<usize>();
            let benchmark_ready_tool_row_count =
                stages.iter().map(|stage| stage.benchmark_ready_tool_count).sum::<usize>();
            let blocked_stage_ids = stages
                .iter()
                .filter(|stage| stage.blocked_tool_count > 0)
                .map(|stage| stage.stage_id.clone())
                .collect::<Vec<_>>();

            CorpusCentricCorpusReport {
                corpus_family_id,
                domains: stages
                    .iter()
                    .map(|stage| stage.domain.clone())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect(),
                fixture_ids: stages
                    .iter()
                    .flat_map(|stage| stage.fixture_ids.iter().cloned())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect(),
                stage_count: stages.len(),
                tool_row_count,
                benchmark_ready_tool_row_count,
                blocked_stage_count: blocked_stage_ids.len(),
                blocked_stage_ids,
                stages,
            }
        })
        .collect::<Vec<_>>();

    corpora.sort_by(|left, right| left.corpus_family_id.cmp(&right.corpus_family_id));
    ensure_corpus_centric_report_contract(&corpora)?;
    Ok(corpora)
}

#[derive(Debug, Clone)]
struct StageAssignment {
    corpus_family_id: String,
    fixture_ids: Vec<String>,
}

fn load_stage_assignments(repo_root: &Path) -> Result<BTreeMap<(String, String), StageAssignment>> {
    let mut assignments = BTreeMap::<(String, String), StageAssignment>::new();
    let (_, _, fastq_rows) = collect_fastq_corpus_assignment_rows(repo_root)?;
    for row in fastq_rows {
        match row.assignment_status {
            FastqCorpusAssignmentStatus::Assigned => {
                let stage_id = row.stage_id.clone();
                let corpus_family_id = row
                    .corpus_family_id
                    .clone()
                    .ok_or_else(|| anyhow!("assigned FASTQ corpus row is missing corpus family"))?;
                let fixture_id = row
                    .fixture_id
                    .clone()
                    .ok_or_else(|| anyhow!("assigned FASTQ corpus row is missing fixture id"))?;
                let entry =
                    assignments.entry(("fastq".to_string(), stage_id.clone())).or_insert_with(|| {
                        StageAssignment {
                            corpus_family_id: corpus_family_id.clone(),
                            fixture_ids: Vec::new(),
                        }
                    });
                if entry.corpus_family_id != corpus_family_id {
                    return Err(anyhow!("FASTQ stage `{}` drifted across corpus families", stage_id));
                }
                entry.fixture_ids.push(fixture_id);
            }
            FastqCorpusAssignmentStatus::AssetBacked => {
                let stage_id = row.stage_id.clone();
                let benchmark_scope_id = row.benchmark_scope_id.clone().ok_or_else(|| {
                    anyhow!("asset-backed FASTQ corpus row is missing benchmark scope id")
                })?;
                let entry =
                    assignments.entry(("fastq".to_string(), stage_id.clone())).or_insert_with(|| {
                        StageAssignment {
                            corpus_family_id: benchmark_scope_id.clone(),
                            fixture_ids: Vec::new(),
                        }
                    });
                if entry.corpus_family_id != benchmark_scope_id {
                    return Err(anyhow!(
                        "FASTQ stage `{}` drifted across asset-backed benchmark scopes",
                        stage_id
                    ));
                }
                entry.fixture_ids.push(benchmark_scope_id);
            }
            FastqCorpusAssignmentStatus::Excluded => continue,
        }
    }

    let (_, _, bam_rows) = collect_bam_corpus_assignment_rows(repo_root)?;
    for row in bam_rows {
        let entry =
            assignments.entry(("bam".to_string(), row.stage_id.clone())).or_insert_with(|| {
                StageAssignment {
                    corpus_family_id: row.corpus_family_id.clone(),
                    fixture_ids: Vec::new(),
                }
            });
        if entry.corpus_family_id != row.corpus_family_id {
            return Err(anyhow!("BAM stage `{}` drifted across corpus families", row.stage_id));
        }
        entry.fixture_ids.push(row.fixture_id);
    }

    for assignment in assignments.values_mut() {
        assignment.fixture_ids.sort();
        assignment.fixture_ids.dedup();
    }

    Ok(assignments)
}

fn render_corpus_stage_row(
    stage: StageCentricStageReport,
    fixture_ids: &[String],
) -> CorpusCentricStageRow {
    CorpusCentricStageRow {
        domain: stage.domain,
        stage_id: stage.stage_id,
        canonical_stage_rank: stage.canonical_stage_rank,
        fixture_ids: fixture_ids.to_vec(),
        report_section_id: stage.report_section_id,
        report_section_title: stage.report_section_title,
        summary_table_id: stage.summary_table_id,
        summary_table_title: stage.summary_table_title,
        comparison_contract_status: stage.comparison_contract_status,
        shared_metric_field_count: stage.shared_metric_field_count,
        shared_metric_fields: stage.shared_metric_fields,
        tool_count: stage.tool_count,
        benchmark_ready_tool_count: stage.benchmark_ready_tool_count,
        blocked_tool_count: stage.blocked_tool_count,
        tool_ids: stage.tools.iter().map(|tool| tool.tool_id.clone()).collect(),
        blocked_tool_ids: stage.blocked_tool_ids,
    }
}

fn ensure_corpus_centric_report_contract(corpora: &[CorpusCentricCorpusReport]) -> Result<()> {
    if corpora.len() != 8 {
        return Err(anyhow!(
            "corpus-centric report must retain exactly 8 corpora, found {}",
            corpora.len()
        ));
    }
    let stage_count = corpora.iter().map(|corpus| corpus.stage_count).sum::<usize>();
    if stage_count != 50 {
        return Err(anyhow!(
            "corpus-centric report must retain exactly 50 assigned stages, found {}",
            stage_count
        ));
    }
    let tool_row_count = corpora.iter().map(|corpus| corpus.tool_row_count).sum::<usize>();
    if tool_row_count != 122 {
        return Err(anyhow!(
            "corpus-centric report must retain exactly 122 assigned stage-tool rows, found {}",
            tool_row_count
        ));
    }
    let blocked_corpus_count =
        corpora.iter().filter(|corpus| corpus.blocked_stage_count > 0).count();
    if blocked_corpus_count != 2 {
        return Err(anyhow!(
            "corpus-centric report must retain exactly 2 corpora with blocked stages, found {}",
            blocked_corpus_count
        ));
    }

    ensure_corpus(
        corpora,
        "reference-index-assets",
        1,
        2,
        0,
        &["reference-index-assets"],
        &[],
    )?;
    ensure_corpus(
        corpora,
        "corpus-01",
        20,
        63,
        2,
        &["corpus-01-mini"],
        &["fastq.trim_reads", "fastq.filter_low_complexity"],
    )?;
    ensure_corpus(corpora, "corpus-02", 1, 4, 0, &["corpus-02-edna-mini"], &[])?;
    ensure_corpus(
        corpora,
        "corpus-03",
        5,
        6,
        1,
        &["corpus-03-amplicon-mini"],
        &["fastq.normalize_abundance"],
    )?;
    ensure_corpus(
        corpora,
        "corpus-01-adna-bam",
        5,
        16,
        0,
        &["corpus-01-adna-bam-mini", "corpus-01-adna-damage-mini"],
        &[],
    )?;
    ensure_corpus(corpora, "corpus-01-bam", 16, 28, 0, &["corpus-01-bam-mini"], &[])?;
    ensure_corpus(corpora, "corpus-01-genotyping", 1, 1, 0, &["corpus-01-genotyping-mini"], &[])?;
    ensure_corpus(corpora, "corpus-01-kinship", 1, 2, 0, &["corpus-01-kinship-mini"], &[])?;

    ensure_stage(corpora, "corpus-02", "fastq.screen_taxonomy", 4)?;
    ensure_stage(corpora, "corpus-03", "fastq.cluster_otus", 1)?;
    ensure_stage(corpora, "corpus-03", "fastq.infer_asvs", 1)?;
    ensure_stage(corpora, "corpus-03", "fastq.remove_chimeras", 1)?;
    ensure_stage(corpora, "corpus-01-adna-bam", "bam.damage", 6)?;
    ensure_stage(corpora, "corpus-01-adna-bam", "bam.contamination", 3)?;
    ensure_stage(corpora, "corpus-01-genotyping", "bam.genotyping", 1)?;
    ensure_stage(corpora, "corpus-01-kinship", "bam.kinship", 2)?;
    Ok(())
}

fn ensure_corpus(
    corpora: &[CorpusCentricCorpusReport],
    corpus_family_id: &str,
    expected_stage_count: usize,
    expected_tool_row_count: usize,
    expected_blocked_stage_count: usize,
    expected_fixture_ids: &[&str],
    expected_blocked_stage_ids: &[&str],
) -> Result<()> {
    let corpus = corpora
        .iter()
        .find(|corpus| corpus.corpus_family_id == corpus_family_id)
        .ok_or_else(|| anyhow!("corpus-centric report is missing corpus `{}`", corpus_family_id))?;
    if corpus.stage_count != expected_stage_count
        || corpus.tool_row_count != expected_tool_row_count
        || corpus.blocked_stage_count != expected_blocked_stage_count
        || corpus.fixture_ids
            != expected_fixture_ids.iter().map(|id| (*id).to_string()).collect::<Vec<_>>()
        || corpus.blocked_stage_ids
            != expected_blocked_stage_ids.iter().map(|id| (*id).to_string()).collect::<Vec<_>>()
    {
        return Err(anyhow!(
            "corpus-centric report corpus `{}` drifted from its governed contract: stage_count={} tool_row_count={} blocked_stage_count={} fixture_ids={:?} blocked_stage_ids={:?}",
            corpus_family_id,
            corpus.stage_count,
            corpus.tool_row_count,
            corpus.blocked_stage_count,
            corpus.fixture_ids,
            corpus.blocked_stage_ids
        ));
    }
    Ok(())
}

fn ensure_stage(
    corpora: &[CorpusCentricCorpusReport],
    corpus_family_id: &str,
    stage_id: &str,
    expected_tool_count: usize,
) -> Result<()> {
    let corpus = corpora
        .iter()
        .find(|corpus| corpus.corpus_family_id == corpus_family_id)
        .ok_or_else(|| anyhow!("corpus-centric report is missing corpus `{}`", corpus_family_id))?;
    let stage =
        corpus.stages.iter().find(|stage| stage.stage_id == stage_id).ok_or_else(|| {
            anyhow!("corpus `{}` is missing stage `{}`", corpus_family_id, stage_id)
        })?;
    if stage.tool_count != expected_tool_count {
        return Err(anyhow!(
            "corpus `{}` stage `{}` drifted from its governed tool count",
            corpus_family_id,
            stage_id
        ));
    }
    Ok(())
}

fn render_corpus_centric_markdown(corpora: &[CorpusCentricCorpusReport]) -> String {
    let stage_count = corpora.iter().map(|corpus| corpus.stage_count).sum::<usize>();
    let tool_row_count = corpora.iter().map(|corpus| corpus.tool_row_count).sum::<usize>();
    let benchmark_ready_tool_row_count =
        corpora.iter().map(|corpus| corpus.benchmark_ready_tool_row_count).sum::<usize>();
    let blocked_tool_row_count = tool_row_count.saturating_sub(benchmark_ready_tool_row_count);
    let blocked_corpus_count =
        corpora.iter().filter(|corpus| corpus.blocked_stage_count > 0).count();

    let mut rendered = String::from("# Corpus-Centric Benchmark Report\n\n");
    rendered.push_str("## Summary\n\n");
    rendered.push_str(&format!(
        "- Corpus count: {}\n- Assigned stages: {}\n- Assigned stage-tool rows: {}\n- Benchmark-ready rows: {}\n- Blocked rows: {}\n- Corpora with blocked stages: {}\n\n",
        corpora.len(),
        stage_count,
        tool_row_count,
        benchmark_ready_tool_row_count,
        blocked_tool_row_count,
        blocked_corpus_count,
    ));
    rendered.push_str("| Corpus | Domains | Fixtures | Stages | Tool rows | Ready | Blocked stages | Blocked stage ids |\n");
    rendered.push_str("| --- | --- | --- | ---: | ---: | ---: | ---: | --- |\n");
    for corpus in corpora {
        let blocked_stage_summary = if corpus.blocked_stage_ids.is_empty() {
            "none".to_string()
        } else {
            corpus.blocked_stage_ids.join(", ")
        };
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&corpus.corpus_family_id),
            sanitize_markdown_cell(&corpus.domains.join(", ")),
            sanitize_markdown_cell(&corpus.fixture_ids.join(", ")),
            corpus.stage_count,
            corpus.tool_row_count,
            corpus.benchmark_ready_tool_row_count,
            corpus.blocked_stage_count,
            sanitize_markdown_cell(&blocked_stage_summary),
        ));
    }

    for corpus in corpora {
        rendered.push_str(&format!("\n## {}\n\n", corpus.corpus_family_id));
        rendered.push_str(&format!(
            "- Domains: {}\n- Fixtures: {}\n- Stages: {}\n- Tool rows: {}\n- Benchmark-ready rows: {}\n- Blocked stages: {}\n\n",
            corpus.domains.join(", "),
            corpus.fixture_ids.join(", "),
            corpus.stage_count,
            corpus.tool_row_count,
            corpus.benchmark_ready_tool_row_count,
            corpus.blocked_stage_count,
        ));
        rendered.push_str("| Domain | Stage | Fixtures | Report section | Tools | Ready | Blocked | Shared metrics | Blocked tools |\n");
        rendered.push_str("| --- | --- | --- | --- | ---: | ---: | ---: | --- | --- |\n");
        for stage in &corpus.stages {
            let shared_metric_summary = if stage.shared_metric_fields.is_empty() {
                stage.comparison_contract_status.clone()
            } else {
                stage.shared_metric_fields.join(", ")
            };
            let blocked_tool_summary = if stage.blocked_tool_ids.is_empty() {
                "none".to_string()
            } else {
                stage.blocked_tool_ids.join(", ")
            };
            rendered.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                sanitize_markdown_cell(&stage.domain),
                sanitize_markdown_cell(&stage.stage_id),
                sanitize_markdown_cell(&stage.fixture_ids.join(", ")),
                sanitize_markdown_cell(&stage.report_section_title),
                stage.tool_count,
                stage.benchmark_ready_tool_count,
                stage.blocked_tool_count,
                sanitize_markdown_cell(&shared_metric_summary),
                sanitize_markdown_cell(&blocked_tool_summary),
            ));
        }
    }

    rendered
}

fn repo_relative_path(repo_root: &Path, output_path: &Path) -> PathBuf {
    if output_path.is_absolute() {
        output_path.to_path_buf()
    } else {
        repo_root.join(output_path)
    }
}

fn path_relative_to_repo(repo_root: &Path, output_path: &Path) -> String {
    output_path.strip_prefix(repo_root).unwrap_or(output_path).to_string_lossy().replace('\\', "/")
}

fn sanitize_markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::{render_corpus_centric_report, DEFAULT_CORPUS_CENTRIC_REPORT_PATH};
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace crates directory")
            .parent()
            .expect("repository root")
            .to_path_buf()
    }

    #[test]
    fn corpus_centric_report_tracks_governed_corpus_stage_inventory() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let report = render_corpus_centric_report(
            &repo_root,
            tempdir.path().join(DEFAULT_CORPUS_CENTRIC_REPORT_PATH),
        )
        .expect("render corpus-centric report");

        assert_eq!(report.corpus_count, 8);
        assert_eq!(report.stage_count, 50);
        assert_eq!(report.tool_row_count, 122);
        assert_eq!(report.benchmark_ready_tool_row_count, 118);
        assert_eq!(report.blocked_tool_row_count, 4);
        assert_eq!(report.blocked_corpus_count, 2);
    }

    #[test]
    fn corpus_centric_report_writes_named_corpus_rows() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let output_path = tempdir.path().join(DEFAULT_CORPUS_CENTRIC_REPORT_PATH);
        let report =
            render_corpus_centric_report(&repo_root, output_path.clone()).expect("render report");

        assert!(report.output_path.ends_with(DEFAULT_CORPUS_CENTRIC_REPORT_PATH));
        let markdown = std::fs::read_to_string(output_path).expect("read markdown");
        assert!(markdown.contains("# Corpus-Centric Benchmark Report"));
        assert!(markdown.contains("## corpus-02"));
        assert!(markdown.contains("| fastq | fastq.screen_taxonomy | corpus-02-edna-mini | Contamination Screening | 4 | 4 | 0 | not_declared | none |"));
        assert!(markdown.contains("## corpus-03"));
        assert!(markdown.contains("| fastq | fastq.normalize_abundance | corpus-03-amplicon-mini | Amplicon Interpretation | 2 | 1 | 1 | not_declared | seqfu (support) |"));
        assert!(markdown.contains("## reference-index-assets"));
        assert!(markdown.contains("| fastq | fastq.index_reference | reference-index-assets | Reference Preparation | 2 | 2 | 0 | index_build_exit_code | none |"));
        assert!(markdown.contains("## corpus-01-adna-bam"));
        assert!(markdown.contains("| bam | bam.damage | corpus-01-adna-damage-mini | Ancient Signal | 6 | 6 | 0 | terminal_c_to_t_5p, terminal_g_to_a_3p, damage_signal, runtime_s, memory_mb | none |"));
    }
}
