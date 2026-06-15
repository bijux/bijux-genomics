use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::bam_report_map::collect_bam_report_map_rows;
use super::fastq_report_map::collect_fastq_report_stage_metadata;
use super::pair_readiness::{collect_pair_readiness_rows, PairAssetStatus, PairReadinessGap};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_TOOL_CENTRIC_REPORT_PATH: &str =
    "benchmarks/readiness/tool-centric-report.md";
const TOOL_CENTRIC_REPORT_SCHEMA_VERSION: &str = "bijux.bench.readiness.tool_centric_report.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ToolCentricStageRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) report_section_id: String,
    pub(crate) report_section_title: String,
    pub(crate) summary_table_id: String,
    pub(crate) summary_table_title: String,
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
pub(crate) struct ToolCentricToolReport {
    pub(crate) tool_id: String,
    pub(crate) domains: Vec<String>,
    pub(crate) stage_count: usize,
    pub(crate) benchmark_ready_stage_count: usize,
    pub(crate) blocked_stage_count: usize,
    pub(crate) report_section_ids: Vec<String>,
    pub(crate) readiness_gap_counts: BTreeMap<String, usize>,
    pub(crate) blocked_stage_ids: Vec<String>,
    pub(crate) stages: Vec<ToolCentricStageRow>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolCentricReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) tool_count: usize,
    pub(crate) unique_stage_count: usize,
    pub(crate) row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) blocked_row_count: usize,
    pub(crate) blocked_tool_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) readiness_gap_counts: BTreeMap<String, usize>,
    pub(crate) tools: Vec<ToolCentricToolReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StageReportPlacement {
    report_section_id: String,
    report_section_title: String,
    summary_table_id: String,
    summary_table_title: String,
}

pub(crate) fn run_render_tool_centric_report(
    args: &parse::BenchReadinessRenderToolCentricReportArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_tool_centric_report(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_TOOL_CENTRIC_REPORT_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_tool_centric_report(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<ToolCentricReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let tools = collect_tool_centric_tools(repo_root)?;
    let row_count = tools.iter().map(|tool| tool.stage_count).sum::<usize>();
    let benchmark_ready_row_count =
        tools.iter().map(|tool| tool.benchmark_ready_stage_count).sum::<usize>();
    let blocked_row_count = row_count.saturating_sub(benchmark_ready_row_count);
    let blocked_tool_count = tools.iter().filter(|tool| tool.blocked_stage_count > 0).count();
    let unique_stage_count = tools
        .iter()
        .flat_map(|tool| {
            tool.stages.iter().map(|stage| (stage.domain.clone(), stage.stage_id.clone()))
        })
        .collect::<BTreeSet<_>>()
        .len();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut readiness_gap_counts = BTreeMap::<String, usize>::new();
    for tool in &tools {
        for stage in &tool.stages {
            *domain_counts.entry(stage.domain.clone()).or_default() += 1;
            *readiness_gap_counts.entry(stage.readiness_gap.clone()).or_default() += 1;
        }
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_tool_centric_markdown(&tools, blocked_tool_count))
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(ToolCentricReport {
        schema_version: TOOL_CENTRIC_REPORT_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        tool_count: tools.len(),
        unique_stage_count,
        row_count,
        benchmark_ready_row_count,
        blocked_row_count,
        blocked_tool_count,
        domain_counts,
        readiness_gap_counts,
        tools,
    })
}

fn collect_tool_centric_tools(repo_root: &Path) -> Result<Vec<ToolCentricToolReport>> {
    let rows = collect_pair_readiness_rows(repo_root)?;
    let placements = load_stage_report_placements(repo_root)?;
    let mut rows_by_tool = BTreeMap::<String, Vec<ToolCentricStageRow>>::new();

    for row in rows {
        let placement =
            placements.get(&(row.domain.clone(), row.stage_id.clone())).ok_or_else(|| {
                anyhow!(
                    "tool-centric report is missing stage placement for `{}` / `{}`",
                    row.domain,
                    row.stage_id
                )
            })?;
        rows_by_tool.entry(row.tool_id.clone()).or_default().push(ToolCentricStageRow {
            domain: row.domain,
            stage_id: row.stage_id,
            report_section_id: placement.report_section_id.clone(),
            report_section_title: placement.report_section_title.clone(),
            summary_table_id: placement.summary_table_id.clone(),
            summary_table_title: placement.summary_table_title.clone(),
            benchmark_status: row.benchmark_status,
            readiness_gap: pair_readiness_gap_label(row.readiness_gap).to_string(),
            support_status: row.support_status,
            adapter_status: row.adapter_status,
            parser_status: row.parser_status,
            corpus_status: row.corpus_status,
            asset_status: pair_asset_status_label(row.asset_status).to_string(),
            reason: row.reason,
        });
    }

    let mut tools = rows_by_tool
        .into_iter()
        .map(|(tool_id, mut stages)| {
            stages.sort_by(|left, right| {
                left.domain.cmp(&right.domain).then_with(|| left.stage_id.cmp(&right.stage_id))
            });
            let benchmark_ready_stage_count =
                stages.iter().filter(|stage| stage.benchmark_status == "benchmark_ready").count();
            let blocked_stage_ids = stages
                .iter()
                .filter(|stage| stage.benchmark_status != "benchmark_ready")
                .map(|stage| format!("{} ({})", stage.stage_id, stage.readiness_gap))
                .collect::<Vec<_>>();
            let mut readiness_gap_counts = BTreeMap::<String, usize>::new();
            for stage in &stages {
                *readiness_gap_counts.entry(stage.readiness_gap.clone()).or_default() += 1;
            }

            ToolCentricToolReport {
                tool_id,
                domains: stages
                    .iter()
                    .map(|stage| stage.domain.clone())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect(),
                stage_count: stages.len(),
                benchmark_ready_stage_count,
                blocked_stage_count: stages.len().saturating_sub(benchmark_ready_stage_count),
                report_section_ids: stages
                    .iter()
                    .map(|stage| stage.report_section_id.clone())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect(),
                readiness_gap_counts,
                blocked_stage_ids,
                stages,
            }
        })
        .collect::<Vec<_>>();

    tools.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    ensure_tool_centric_report_contract(&tools)?;
    Ok(tools)
}

fn load_stage_report_placements(
    repo_root: &Path,
) -> Result<BTreeMap<(String, String), StageReportPlacement>> {
    let mut placements = BTreeMap::<(String, String), StageReportPlacement>::new();
    for (stage_id, row) in collect_fastq_report_stage_metadata(repo_root)? {
        placements.insert(
            ("fastq".to_string(), stage_id),
            StageReportPlacement {
                report_section_id: row.report_section_id,
                report_section_title: row.report_section_title,
                summary_table_id: row.summary_table_id,
                summary_table_title: row.summary_table_title,
            },
        );
    }
    for row in collect_bam_report_map_rows(repo_root)? {
        placements.insert(
            ("bam".to_string(), row.stage_id),
            StageReportPlacement {
                report_section_id: row.report_section_id,
                report_section_title: row.report_section_title,
                summary_table_id: row.summary_table_id,
                summary_table_title: row.summary_table_title,
            },
        );
    }
    Ok(placements)
}

fn ensure_tool_centric_report_contract(tools: &[ToolCentricToolReport]) -> Result<()> {
    if tools.len() != 67 {
        return Err(anyhow!(
            "tool-centric report must retain exactly 67 tools, found {}",
            tools.len()
        ));
    }
    let row_count = tools.iter().map(|tool| tool.stage_count).sum::<usize>();
    if row_count != 122 {
        return Err(anyhow!(
            "tool-centric report must retain exactly 122 stage-tool rows, found {row_count}"
        ));
    }
    let unique_stage_count = tools
        .iter()
        .flat_map(|tool| {
            tool.stages.iter().map(|stage| (stage.domain.clone(), stage.stage_id.clone()))
        })
        .collect::<BTreeSet<_>>()
        .len();
    if unique_stage_count != 51 {
        return Err(anyhow!(
            "tool-centric report must retain exactly 51 unique benchmark stages, found {unique_stage_count}"
        ));
    }

    ensure_tool_stages(
        tools,
        "samtools",
        &[
            "bam.coverage",
            "bam.duplication_metrics",
            "bam.endogenous_content",
            "bam.filter",
            "bam.length_filter",
            "bam.mapping_summary",
            "bam.mapq_filter",
            "bam.markdup",
            "bam.qc_pre",
            "bam.validate",
        ],
        0,
    )?;
    ensure_tool_stages(
        tools,
        "picard",
        &[
            "bam.duplication_metrics",
            "bam.gc_bias",
            "bam.insert_size",
            "bam.length_filter",
            "bam.mapping_summary",
            "bam.markdup",
        ],
        0,
    )?;
    ensure_tool_stages(
        tools,
        "fastp",
        &[
            "fastq.filter_low_complexity",
            "fastq.filter_reads",
            "fastq.profile_read_lengths",
            "fastq.trim_polyg_tails",
            "fastq.trim_reads",
        ],
        1,
    )?;
    ensure_tool_stages(
        tools,
        "vsearch",
        &["fastq.cluster_otus", "fastq.merge_pairs", "fastq.remove_chimeras"],
        0,
    )?;
    ensure_tool_stages(tools, "kraken2", &["fastq.screen_taxonomy"], 0)?;
    ensure_tool_stages(
        tools,
        "bowtie2",
        &["bam.align", "fastq.deplete_host", "fastq.deplete_reference_contaminants"],
        0,
    )?;
    ensure_tool_stages(tools, "gatk", &["bam.recalibration"], 0)?;
    Ok(())
}

fn ensure_tool_stages(
    tools: &[ToolCentricToolReport],
    tool_id: &str,
    expected_stage_ids: &[&str],
    expected_blocked_stage_count: usize,
) -> Result<()> {
    let tool = tools
        .iter()
        .find(|tool| tool.tool_id == tool_id)
        .ok_or_else(|| anyhow!("tool-centric report is missing tool `{tool_id}`"))?;
    let stage_ids = tool.stages.iter().map(|stage| stage.stage_id.as_str()).collect::<Vec<_>>();
    if stage_ids != expected_stage_ids {
        return Err(anyhow!(
            "tool-centric report tool `{tool_id}` drifted from its governed stage list: expected {expected_stage_ids:?}, found {stage_ids:?}"
        ));
    }
    if tool.blocked_stage_count != expected_blocked_stage_count {
        return Err(anyhow!(
            "tool-centric report tool `{tool_id}` must retain {} blocked stages, found {}",
            expected_blocked_stage_count,
            tool.blocked_stage_count
        ));
    }
    Ok(())
}

fn render_tool_centric_markdown(
    tools: &[ToolCentricToolReport],
    blocked_tool_count: usize,
) -> String {
    let row_count = tools.iter().map(|tool| tool.stage_count).sum::<usize>();
    let benchmark_ready_row_count =
        tools.iter().map(|tool| tool.benchmark_ready_stage_count).sum::<usize>();
    let blocked_row_count = row_count.saturating_sub(benchmark_ready_row_count);

    let mut rendered = String::from("# Tool-Centric Benchmark Report\n\n");
    rendered.push_str("## Summary\n\n");
    rendered.push_str(&format!(
        "- Tool count: {}\n- Stage-tool rows: {}\n- Benchmark-ready rows: {}\n- Blocked rows: {}\n- Tools with blockers: {}\n\n",
        tools.len(),
        row_count,
        benchmark_ready_row_count,
        blocked_row_count,
        blocked_tool_count,
    ));
    rendered.push_str("| Tool | Domains | Stage rows | Ready | Blocked | Blocked stages |\n");
    rendered.push_str("| --- | --- | ---: | ---: | ---: | --- |\n");
    for tool in tools {
        let blocked_stage_summary = if tool.blocked_stage_ids.is_empty() {
            "none".to_string()
        } else {
            tool.blocked_stage_ids.join(", ")
        };
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&tool.tool_id),
            sanitize_markdown_cell(&tool.domains.join(", ")),
            tool.stage_count,
            tool.benchmark_ready_stage_count,
            tool.blocked_stage_count,
            sanitize_markdown_cell(&blocked_stage_summary),
        ));
    }

    for tool in tools {
        rendered.push_str(&format!("\n## {}\n\n", tool.tool_id));
        rendered.push_str(&format!(
            "- Domains: {}\n- Stage rows: {}\n- Benchmark-ready rows: {}\n- Blocked rows: {}\n- Report sections: {}\n\n",
            tool.domains.join(", "),
            tool.stage_count,
            tool.benchmark_ready_stage_count,
            tool.blocked_stage_count,
            tool.report_section_ids.join(", "),
        ));
        rendered.push_str("| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |\n");
        rendered.push_str("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");
        for stage in &tool.stages {
            rendered.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                sanitize_markdown_cell(&stage.domain),
                sanitize_markdown_cell(&stage.stage_id),
                sanitize_markdown_cell(&stage.report_section_title),
                sanitize_markdown_cell(&stage.summary_table_title),
                sanitize_markdown_cell(&stage.benchmark_status),
                sanitize_markdown_cell(&stage.readiness_gap),
                sanitize_markdown_cell(&stage.support_status),
                sanitize_markdown_cell(&stage.adapter_status),
                sanitize_markdown_cell(&stage.parser_status),
                sanitize_markdown_cell(&stage.corpus_status),
                sanitize_markdown_cell(&stage.asset_status),
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
    use super::{render_tool_centric_report, DEFAULT_TOOL_CENTRIC_REPORT_PATH};
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
    fn tool_centric_report_tracks_named_tool_stage_lists() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let report = render_tool_centric_report(
            &repo_root,
            tempdir.path().join(DEFAULT_TOOL_CENTRIC_REPORT_PATH),
        )
        .expect("render tool-centric report");

        assert_eq!(report.tool_count, 67);
        assert_eq!(report.unique_stage_count, 51);
        assert_eq!(report.row_count, 122);
        assert_eq!(report.benchmark_ready_row_count, 118);
        assert_eq!(report.blocked_row_count, 4);
        assert_eq!(report.blocked_tool_count, 4);

        let samtools = report
            .tools
            .iter()
            .find(|tool| tool.tool_id == "samtools")
            .expect("samtools tool report");
        assert_eq!(samtools.stage_count, 10);
        assert_eq!(samtools.blocked_stage_count, 0);

        let fastp =
            report.tools.iter().find(|tool| tool.tool_id == "fastp").expect("fastp tool report");
        assert_eq!(fastp.stage_count, 5);
        assert_eq!(fastp.blocked_stage_count, 1);
        assert_eq!(
            fastp.blocked_stage_ids,
            vec!["fastq.filter_low_complexity (support)".to_string()]
        );
    }

    #[test]
    fn tool_centric_report_writes_named_markdown_rows() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let output_path = tempdir.path().join(DEFAULT_TOOL_CENTRIC_REPORT_PATH);
        let report =
            render_tool_centric_report(&repo_root, output_path.clone()).expect("render report");

        assert!(report.output_path.ends_with(DEFAULT_TOOL_CENTRIC_REPORT_PATH));
        let markdown = std::fs::read_to_string(output_path).expect("read markdown");
        assert!(markdown.contains("# Tool-Centric Benchmark Report"));
        assert!(markdown.contains("## samtools"));
        assert!(markdown
            .contains("| fastp | fastq | 5 | 4 | 1 | fastq.filter_low_complexity (support) |"));
        assert!(
            markdown.contains("| bam | bam.recalibration | Downstream Readiness | Variant and Bias Readiness | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | assigned |")
        );
    }
}
