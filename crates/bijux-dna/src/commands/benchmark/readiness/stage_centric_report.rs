use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::bam_comparable_metrics::render_bam_comparable_metrics;
use super::bam_report_map::collect_bam_report_map_rows;
use super::fastq_comparable_metrics::render_fastq_comparable_metrics;
use super::fastq_report_map::collect_fastq_report_stage_metadata;
use super::pair_readiness::{collect_pair_readiness_rows, PairAssetStatus, PairReadinessGap};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_STAGE_CENTRIC_REPORT_PATH: &str =
    "benchmarks/readiness/stage-centric-report.md";
const STAGE_CENTRIC_REPORT_SCHEMA_VERSION: &str = "bijux.bench.readiness.stage_centric_report.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct StageCentricToolRow {
    pub(crate) tool_id: String,
    pub(crate) benchmark_status: String,
    pub(crate) readiness_gap: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
    pub(crate) asset_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct StageCentricStageReport {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) canonical_stage_rank: usize,
    pub(crate) report_section_id: String,
    pub(crate) report_section_title: String,
    pub(crate) summary_table_id: String,
    pub(crate) summary_table_title: String,
    pub(crate) anchor_tool_id: String,
    pub(crate) anchor_support_status: String,
    pub(crate) comparison_contract_status: String,
    pub(crate) shared_metric_field_count: usize,
    pub(crate) shared_metric_fields: Vec<String>,
    pub(crate) tool_count: usize,
    pub(crate) benchmark_ready_tool_count: usize,
    pub(crate) blocked_tool_count: usize,
    pub(crate) readiness_gap_counts: BTreeMap<String, usize>,
    pub(crate) blocked_tool_ids: Vec<String>,
    pub(crate) tools: Vec<StageCentricToolRow>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct StageCentricReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) multi_tool_stage_count: usize,
    pub(crate) blocked_stage_count: usize,
    pub(crate) declared_shared_metric_stage_count: usize,
    pub(crate) not_declared_shared_metric_stage_count: usize,
    pub(crate) row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) blocked_row_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) section_counts: BTreeMap<String, usize>,
    pub(crate) readiness_gap_counts: BTreeMap<String, usize>,
    pub(crate) stages: Vec<StageCentricStageReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StageMetadata {
    canonical_stage_rank: usize,
    report_section_id: String,
    report_section_title: String,
    summary_table_id: String,
    summary_table_title: String,
    anchor_tool_id: String,
    anchor_support_status: String,
}

pub(crate) fn run_render_stage_centric_report(
    args: &parse::BenchReadinessRenderStageCentricReportArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_stage_centric_report(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_STAGE_CENTRIC_REPORT_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_stage_centric_report(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<StageCentricReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let stages = collect_stage_centric_stage_reports(repo_root)?;
    let row_count = stages.iter().map(|stage| stage.tool_count).sum::<usize>();
    let benchmark_ready_row_count =
        stages.iter().map(|stage| stage.benchmark_ready_tool_count).sum::<usize>();
    let blocked_row_count = row_count.saturating_sub(benchmark_ready_row_count);
    let stage_count = stages.len();
    let multi_tool_stage_count = stages.iter().filter(|stage| stage.tool_count > 1).count();
    let blocked_stage_count = stages.iter().filter(|stage| stage.blocked_tool_count > 0).count();
    let declared_shared_metric_stage_count =
        stages.iter().filter(|stage| stage.comparison_contract_status == "declared").count();
    let not_declared_shared_metric_stage_count =
        stages.iter().filter(|stage| stage.comparison_contract_status == "not_declared").count();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut section_counts = BTreeMap::<String, usize>::new();
    let mut readiness_gap_counts = BTreeMap::<String, usize>::new();
    for stage in &stages {
        *domain_counts.entry(stage.domain.clone()).or_default() += 1;
        *section_counts.entry(stage.report_section_id.clone()).or_default() += 1;
        for tool in &stage.tools {
            *readiness_gap_counts.entry(tool.readiness_gap.clone()).or_default() += 1;
        }
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_stage_centric_markdown(&stages))
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(StageCentricReport {
        schema_version: STAGE_CENTRIC_REPORT_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        multi_tool_stage_count,
        blocked_stage_count,
        declared_shared_metric_stage_count,
        not_declared_shared_metric_stage_count,
        row_count,
        benchmark_ready_row_count,
        blocked_row_count,
        domain_counts,
        section_counts,
        readiness_gap_counts,
        stages,
    })
}

pub(crate) fn collect_stage_centric_stage_reports(
    repo_root: &Path,
) -> Result<Vec<StageCentricStageReport>> {
    let rows = collect_pair_readiness_rows(repo_root)?;
    let stage_metadata = load_stage_metadata(repo_root)?;
    let shared_metric_fields_by_stage = load_shared_metric_fields_by_stage(repo_root)?;
    let mut rows_by_stage = BTreeMap::<(String, String), Vec<StageCentricToolRow>>::new();

    for row in rows {
        rows_by_stage.entry((row.domain.clone(), row.stage_id.clone())).or_default().push(
            StageCentricToolRow {
                tool_id: row.tool_id,
                benchmark_status: row.benchmark_status,
                readiness_gap: pair_readiness_gap_label(row.readiness_gap).to_string(),
                support_status: row.support_status,
                adapter_status: row.adapter_status,
                parser_status: row.parser_status,
                corpus_status: row.corpus_status,
                asset_status: pair_asset_status_label(row.asset_status).to_string(),
                reason: row.reason,
            },
        );
    }

    let mut stages = rows_by_stage
        .into_iter()
        .map(|((domain, stage_id), mut tools)| {
            let metadata =
                stage_metadata.get(&(domain.clone(), stage_id.clone())).ok_or_else(|| {
                    anyhow!(
                        "stage-centric report is missing stage metadata for `{}` / `{}`",
                        domain,
                        stage_id
                    )
                })?;
            tools.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
            let benchmark_ready_tool_count =
                tools.iter().filter(|tool| tool.benchmark_status == "benchmark_ready").count();
            let blocked_tool_ids = tools
                .iter()
                .filter(|tool| tool.benchmark_status != "benchmark_ready")
                .map(|tool| format!("{} ({})", tool.tool_id, tool.readiness_gap))
                .collect::<Vec<_>>();
            let mut readiness_gap_counts = BTreeMap::<String, usize>::new();
            for tool in &tools {
                *readiness_gap_counts.entry(tool.readiness_gap.clone()).or_default() += 1;
            }
            let shared_metric_fields = shared_metric_fields_by_stage
                .get(&(domain.clone(), stage_id.clone()))
                .cloned()
                .unwrap_or_default();
            let comparison_contract_status = if !shared_metric_fields.is_empty() {
                "declared"
            } else if tools.len() > 1 {
                "not_declared"
            } else {
                "not_applicable"
            };

            Ok(StageCentricStageReport {
                domain,
                stage_id,
                canonical_stage_rank: metadata.canonical_stage_rank,
                report_section_id: metadata.report_section_id.clone(),
                report_section_title: metadata.report_section_title.clone(),
                summary_table_id: metadata.summary_table_id.clone(),
                summary_table_title: metadata.summary_table_title.clone(),
                anchor_tool_id: metadata.anchor_tool_id.clone(),
                anchor_support_status: metadata.anchor_support_status.clone(),
                comparison_contract_status: comparison_contract_status.to_string(),
                shared_metric_field_count: shared_metric_fields.len(),
                shared_metric_fields,
                tool_count: tools.len(),
                benchmark_ready_tool_count,
                blocked_tool_count: tools.len().saturating_sub(benchmark_ready_tool_count),
                readiness_gap_counts,
                blocked_tool_ids,
                tools,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    stages.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.canonical_stage_rank.cmp(&right.canonical_stage_rank))
            .then_with(|| left.stage_id.cmp(&right.stage_id))
    });
    ensure_stage_centric_report_contract(&stages)?;
    Ok(stages)
}

fn load_stage_metadata(repo_root: &Path) -> Result<BTreeMap<(String, String), StageMetadata>> {
    let mut metadata_by_stage = BTreeMap::<(String, String), StageMetadata>::new();
    for (stage_id, row) in collect_fastq_report_stage_metadata(repo_root)? {
        metadata_by_stage.insert(
            ("fastq".to_string(), stage_id),
            StageMetadata {
                canonical_stage_rank: row.canonical_stage_rank,
                report_section_id: row.report_section_id,
                report_section_title: row.report_section_title,
                summary_table_id: row.summary_table_id,
                summary_table_title: row.summary_table_title,
                anchor_tool_id: row.anchor_tool_id,
                anchor_support_status: row.anchor_support_status,
            },
        );
    }
    for row in collect_bam_report_map_rows(repo_root)? {
        metadata_by_stage.insert(
            ("bam".to_string(), row.stage_id.clone()),
            StageMetadata {
                canonical_stage_rank: row.canonical_stage_rank,
                report_section_id: row.report_section_id,
                report_section_title: row.report_section_title,
                summary_table_id: row.summary_table_id,
                summary_table_title: row.summary_table_title,
                anchor_tool_id: row.anchor_tool_id,
                anchor_support_status: row.anchor_support_status,
            },
        );
    }
    Ok(metadata_by_stage)
}

fn load_shared_metric_fields_by_stage(
    repo_root: &Path,
) -> Result<BTreeMap<(String, String), Vec<String>>> {
    let mut by_stage = BTreeMap::<(String, String), Vec<String>>::new();
    let scratch_root = std::env::temp_dir().join(format!(
        "bijux-stage-centric-report-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&scratch_root)
        .with_context(|| format!("create {}", scratch_root.display()))?;
    let fastq_report = render_fastq_comparable_metrics(
        repo_root,
        scratch_root.join("fastq-comparable-metrics.tsv"),
    )?;
    for row in fastq_report.rows {
        by_stage.insert(("fastq".to_string(), row.stage_id), row.shared_metric_fields);
    }
    let bam_report =
        render_bam_comparable_metrics(repo_root, scratch_root.join("bam-comparable-metrics.tsv"))?;
    for row in bam_report.rows {
        by_stage.insert(("bam".to_string(), row.stage_id), row.shared_metric_fields);
    }
    let _ = fs::remove_dir_all(&scratch_root);
    Ok(by_stage)
}

fn ensure_stage_centric_report_contract(stages: &[StageCentricStageReport]) -> Result<()> {
    if stages.len() != 51 {
        return Err(anyhow!(
            "stage-centric report must retain exactly 51 stages, found {}",
            stages.len()
        ));
    }
    let row_count = stages.iter().map(|stage| stage.tool_count).sum::<usize>();
    if row_count != 122 {
        return Err(anyhow!(
            "stage-centric report must retain exactly 122 stage-tool rows, found {}",
            row_count
        ));
    }
    let multi_tool_stage_count = stages.iter().filter(|stage| stage.tool_count > 1).count();
    if multi_tool_stage_count != 29 {
        return Err(anyhow!(
            "stage-centric report must retain exactly 29 multi-tool stages, found {}",
            multi_tool_stage_count
        ));
    }
    let blocked_stage_count = stages.iter().filter(|stage| stage.blocked_tool_count > 0).count();
    if blocked_stage_count != 3 {
        return Err(anyhow!(
            "stage-centric report must retain exactly 3 blocked stages, found {}",
            blocked_stage_count
        ));
    }
    let declared_shared_metric_stage_count =
        stages.iter().filter(|stage| stage.comparison_contract_status == "declared").count();
    if declared_shared_metric_stage_count != 18 {
        return Err(anyhow!(
            "stage-centric report must retain exactly 18 stages with declared shared metrics, found {}",
            declared_shared_metric_stage_count
        ));
    }
    let not_declared_shared_metric_stage_count =
        stages.iter().filter(|stage| stage.comparison_contract_status == "not_declared").count();
    if not_declared_shared_metric_stage_count != 11 {
        return Err(anyhow!(
            "stage-centric report must retain exactly 11 multi-tool stages without declared shared metrics, found {}",
            not_declared_shared_metric_stage_count
        ));
    }

    ensure_stage(stages, "fastq", "fastq.trim_reads", 14, 1, "not_declared", &[])?;
    ensure_stage(
        stages,
        "fastq",
        "fastq.index_reference",
        2,
        0,
        "declared",
        &["index_build_exit_code"],
    )?;
    ensure_stage(
        stages,
        "fastq",
        "fastq.profile_overrepresented_sequences",
        3,
        0,
        "declared",
        &["sequence_count", "flagged_sequences", "top_fraction"],
    )?;
    ensure_stage(
        stages,
        "bam",
        "bam.damage",
        6,
        0,
        "declared",
        &["terminal_c_to_t_5p", "terminal_g_to_a_3p", "damage_signal", "runtime_s", "memory_mb"],
    )?;
    ensure_stage(
        stages,
        "bam",
        "bam.contamination",
        3,
        0,
        "declared",
        &["scope", "prerequisites_passed", "estimate", "ci_low", "ci_high"],
    )?;
    Ok(())
}

fn ensure_stage(
    stages: &[StageCentricStageReport],
    domain: &str,
    stage_id: &str,
    expected_tool_count: usize,
    expected_blocked_tool_count: usize,
    expected_comparison_contract_status: &str,
    expected_shared_metric_fields: &[&str],
) -> Result<()> {
    let stage = stages
        .iter()
        .find(|stage| stage.domain == domain && stage.stage_id == stage_id)
        .ok_or_else(|| {
            anyhow!("stage-centric report is missing stage `{}` / `{}`", domain, stage_id)
        })?;
    if stage.tool_count != expected_tool_count
        || stage.blocked_tool_count != expected_blocked_tool_count
        || stage.comparison_contract_status != expected_comparison_contract_status
        || stage.shared_metric_fields
            != expected_shared_metric_fields
                .iter()
                .map(|field| (*field).to_string())
                .collect::<Vec<_>>()
    {
        return Err(anyhow!(
            "stage-centric report stage `{}` / `{}` drifted from its governed contract",
            domain,
            stage_id
        ));
    }
    Ok(())
}

fn render_stage_centric_markdown(stages: &[StageCentricStageReport]) -> String {
    let row_count = stages.iter().map(|stage| stage.tool_count).sum::<usize>();
    let benchmark_ready_row_count =
        stages.iter().map(|stage| stage.benchmark_ready_tool_count).sum::<usize>();
    let blocked_row_count = row_count.saturating_sub(benchmark_ready_row_count);
    let multi_tool_stage_count = stages.iter().filter(|stage| stage.tool_count > 1).count();
    let blocked_stage_count = stages.iter().filter(|stage| stage.blocked_tool_count > 0).count();

    let mut rendered = String::from("# Stage-Centric Benchmark Report\n\n");
    rendered.push_str("## Summary\n\n");
    rendered.push_str(&format!(
        "- Stage count: {}\n- Multi-tool stages: {}\n- Stage-tool rows: {}\n- Benchmark-ready rows: {}\n- Blocked rows: {}\n- Stages with blockers: {}\n\n",
        stages.len(),
        multi_tool_stage_count,
        row_count,
        benchmark_ready_row_count,
        blocked_row_count,
        blocked_stage_count,
    ));
    rendered.push_str("| Domain | Stage | Report section | Tools | Ready | Blocked | Shared metrics | Blocked tools |\n");
    rendered.push_str("| --- | --- | --- | ---: | ---: | ---: | --- | --- |\n");
    for stage in stages {
        let blocked_tool_summary = if stage.blocked_tool_ids.is_empty() {
            "none".to_string()
        } else {
            stage.blocked_tool_ids.join(", ")
        };
        let shared_metric_summary = if stage.shared_metric_fields.is_empty() {
            stage.comparison_contract_status.clone()
        } else {
            stage.shared_metric_fields.join(", ")
        };
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&stage.domain),
            sanitize_markdown_cell(&stage.stage_id),
            sanitize_markdown_cell(&stage.report_section_title),
            stage.tool_count,
            stage.benchmark_ready_tool_count,
            stage.blocked_tool_count,
            sanitize_markdown_cell(&shared_metric_summary),
            sanitize_markdown_cell(&blocked_tool_summary),
        ));
    }

    for stage in stages {
        rendered.push_str(&format!("\n## {}\n\n", stage.stage_id));
        rendered.push_str(&format!(
            "- Domain: {}\n- Report section: {}\n- Summary table: {}\n- Anchor tool: {} ({})\n- Tools: {}\n- Ready tools: {}\n- Blocked tools: {}\n- Shared metric contract: {}\n- Shared metrics: {}\n\n",
            stage.domain,
            stage.report_section_title,
            stage.summary_table_title,
            stage.anchor_tool_id,
            stage.anchor_support_status,
            stage.tool_count,
            stage.benchmark_ready_tool_count,
            stage.blocked_tool_count,
            stage.comparison_contract_status,
            if stage.shared_metric_fields.is_empty() {
                "none".to_string()
            } else {
                stage.shared_metric_fields.join(", ")
            },
        ));
        rendered.push_str(
            "| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |\n",
        );
        rendered.push_str("| --- | --- | --- | --- | --- | --- | --- | --- |\n");
        for tool in &stage.tools {
            rendered.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                sanitize_markdown_cell(&tool.tool_id),
                sanitize_markdown_cell(&tool.benchmark_status),
                sanitize_markdown_cell(&tool.readiness_gap),
                sanitize_markdown_cell(&tool.support_status),
                sanitize_markdown_cell(&tool.adapter_status),
                sanitize_markdown_cell(&tool.parser_status),
                sanitize_markdown_cell(&tool.corpus_status),
                sanitize_markdown_cell(&tool.asset_status),
            ));
        }
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

fn sanitize_markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
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

#[cfg(test)]
mod tests {
    use super::{render_stage_centric_report, DEFAULT_STAGE_CENTRIC_REPORT_PATH};
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
    fn stage_centric_report_tracks_multi_tool_stage_contracts() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let report = render_stage_centric_report(
            &repo_root,
            tempdir.path().join(DEFAULT_STAGE_CENTRIC_REPORT_PATH),
        )
        .expect("render stage-centric report");

        assert_eq!(report.stage_count, 51);
        assert_eq!(report.multi_tool_stage_count, 29);
        assert_eq!(report.blocked_stage_count, 3);
        assert_eq!(report.declared_shared_metric_stage_count, 18);
        assert_eq!(report.not_declared_shared_metric_stage_count, 11);
        assert_eq!(report.row_count, 122);
        assert_eq!(report.benchmark_ready_row_count, 118);
        assert_eq!(report.blocked_row_count, 4);

        let trim_reads = report
            .stages
            .iter()
            .find(|stage| stage.domain == "fastq" && stage.stage_id == "fastq.trim_reads")
            .expect("trim reads stage");
        assert_eq!(trim_reads.tool_count, 14);
        assert_eq!(trim_reads.blocked_tool_count, 1);
        assert_eq!(trim_reads.comparison_contract_status, "not_declared");

        let damage = report
            .stages
            .iter()
            .find(|stage| stage.domain == "bam" && stage.stage_id == "bam.damage")
            .expect("bam damage stage");
        let profile_overrepresented = report
            .stages
            .iter()
            .find(|stage| {
                stage.domain == "fastq"
                    && stage.stage_id == "fastq.profile_overrepresented_sequences"
            })
            .expect("profile overrepresented stage");
        assert_eq!(profile_overrepresented.tool_count, 3);
        assert_eq!(profile_overrepresented.blocked_tool_count, 0);
        assert_eq!(profile_overrepresented.shared_metric_field_count, 3);
        assert_eq!(damage.tool_count, 6);
        assert_eq!(damage.shared_metric_field_count, 5);
    }

    #[test]
    fn stage_centric_report_writes_named_markdown_rows() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let output_path = tempdir.path().join(DEFAULT_STAGE_CENTRIC_REPORT_PATH);
        let report =
            render_stage_centric_report(&repo_root, output_path.clone()).expect("render report");

        assert!(report.output_path.ends_with(DEFAULT_STAGE_CENTRIC_REPORT_PATH));
        let markdown = std::fs::read_to_string(output_path).expect("read markdown");
        assert!(markdown.contains("# Stage-Centric Benchmark Report"));
        assert!(markdown.contains("## fastq.trim_reads"));
        assert!(markdown.contains("| seqpurge | not_benchmark_ready | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |"));
        assert!(markdown.contains("## bam.damage"));
        assert!(markdown.contains("| damageprofiler | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |"));
    }
}
