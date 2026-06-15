use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::bam_report_map::collect_bam_report_map_rows;
use super::essential_pipeline_corpus_assets::ESSENTIAL_PIPELINE_IDS;
use super::essential_pipeline_rendered_commands::collect_essential_pipeline_rendered_command_rows;
use super::fastq_report_map::collect_fastq_report_stage_metadata;
use crate::commands::benchmark::local_pipeline_dag::validate_pipeline_dag_path;
use crate::commands::benchmark::local_vcf_stage_catalog::build_vcf_stage_catalog_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ESSENTIAL_PIPELINE_REPORT_MAP_PATH: &str =
    "benchmarks/readiness/essential-pipeline-report-map.tsv";
const ESSENTIAL_PIPELINE_REPORT_MAP_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.essential_pipeline_report_map.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EssentialPipelineReportMapRow {
    pub(crate) pipeline_id: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) output_metric: String,
    pub(crate) report_section: String,
    pub(crate) failure_column: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EssentialPipelineReportMapReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) pipeline_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) row_count: usize,
    pub(crate) report_section_count: usize,
    pub(crate) failure_column_count: usize,
    pub(crate) pipeline_row_counts: BTreeMap<String, usize>,
    pub(crate) domain_row_counts: BTreeMap<String, usize>,
    pub(crate) report_section_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<EssentialPipelineReportMapRow>,
}

pub(crate) fn run_render_essential_pipeline_report_map(
    args: &parse::BenchReadinessRenderEssentialPipelineReportMapArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_essential_pipeline_report_map(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_REPORT_MAP_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_essential_pipeline_report_map(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<EssentialPipelineReportMapReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_essential_pipeline_report_map_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_essential_pipeline_report_map_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let stage_count = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let report_section_count =
        rows.iter().map(|row| row.report_section.as_str()).collect::<BTreeSet<_>>().len();
    let failure_column_count =
        rows.iter().map(|row| row.failure_column.as_str()).collect::<BTreeSet<_>>().len();
    let mut pipeline_row_counts = BTreeMap::<String, usize>::new();
    let mut domain_row_counts = BTreeMap::<String, usize>::new();
    let mut report_section_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *pipeline_row_counts.entry(row.pipeline_id.clone()).or_default() += 1;
        *domain_row_counts.entry(stage_domain(row.stage_id.as_str()).to_string()).or_default() += 1;
        *report_section_counts.entry(row.report_section.clone()).or_default() += 1;
    }

    Ok(EssentialPipelineReportMapReport {
        schema_version: ESSENTIAL_PIPELINE_REPORT_MAP_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        pipeline_count: ESSENTIAL_PIPELINE_IDS.len(),
        stage_count,
        tool_count,
        row_count: rows.len(),
        report_section_count,
        failure_column_count,
        pipeline_row_counts,
        domain_row_counts,
        report_section_counts,
        rows,
    })
}

pub(crate) fn collect_essential_pipeline_report_map_rows(
    repo_root: &Path,
) -> Result<Vec<EssentialPipelineReportMapRow>> {
    let rendered_rows = collect_essential_pipeline_rendered_command_rows(repo_root)?;
    let rendered_tool_by_node = rendered_rows
        .into_iter()
        .map(|row| ((row.pipeline_id.clone(), row.node_id.clone()), row))
        .collect::<BTreeMap<_, _>>();
    let fastq_section_by_stage = collect_fastq_report_stage_metadata(repo_root)?
        .into_iter()
        .map(|(stage_id, row)| (stage_id, row.report_section_id))
        .collect::<BTreeMap<_, _>>();
    let bam_section_by_stage = collect_bam_report_map_rows(repo_root)?
        .into_iter()
        .map(|row| (row.stage_id, row.report_section_id))
        .collect::<BTreeMap<_, _>>();
    let vcf_section_by_stage = build_vcf_stage_catalog_rows()?
        .into_iter()
        .map(|row| (row.stage_id, row.benchmark_category))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::new();
    let mut expected_output_count = 0usize;
    for pipeline_id in ESSENTIAL_PIPELINE_IDS {
        let config_path =
            crate::commands::benchmark::local_pipeline_dag::benchmark_local_pipeline_config_path(
                repo_root,
                pipeline_id,
            );
        let report_path = repo_root
            .join("benchmarks/readiness/local-ready/pipeline-dag")
            .join(format!("{pipeline_id}.json"));
        let report = validate_pipeline_dag_path(repo_root, &config_path, &report_path)?;
        for node_id in &report.topological_order {
            let node = report
                .nodes
                .iter()
                .find(|node| &node.node_id == node_id)
                .ok_or_else(|| {
                    anyhow!(
                        "essential pipeline report map is missing node `{node_id}` in pipeline `{pipeline_id}`"
                    )
                })?;
            let rendered_row =
                rendered_tool_by_node.get(&(pipeline_id.to_string(), node.node_id.clone())).ok_or_else(
                    || {
                        anyhow!(
                            "essential pipeline report map is missing a rendered command row for `{pipeline_id}` / `{}`",
                            node.node_id
                        )
                    },
                )?;
            let report_section = report_section_for_stage(
                node.stage_id.as_str(),
                &fastq_section_by_stage,
                &bam_section_by_stage,
                &vcf_section_by_stage,
            )?;
            let failure_column = failure_column_for_stage(node.stage_id.as_str())?;
            for output_metric in &node.outputs {
                expected_output_count += 1;
                rows.push(EssentialPipelineReportMapRow {
                    pipeline_id: pipeline_id.to_string(),
                    stage_id: node.stage_id.clone(),
                    tool_id: rendered_row.tool_id.clone(),
                    output_metric: output_metric.clone(),
                    report_section: report_section.clone(),
                    failure_column: failure_column.to_string(),
                });
            }
        }
    }

    rows.sort_by(|left, right| {
        left.pipeline_id
            .cmp(&right.pipeline_id)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.output_metric.cmp(&right.output_metric))
    });
    ensure_essential_pipeline_report_map_contract(&rows, expected_output_count)?;
    Ok(rows)
}

fn report_section_for_stage(
    stage_id: &str,
    fastq_section_by_stage: &BTreeMap<String, String>,
    bam_section_by_stage: &BTreeMap<String, String>,
    vcf_section_by_stage: &BTreeMap<String, String>,
) -> Result<String> {
    let section = match stage_domain(stage_id) {
        "fastq" => fastq_section_by_stage.get(stage_id),
        "bam" => bam_section_by_stage.get(stage_id),
        "vcf" => vcf_section_by_stage.get(stage_id),
        other => {
            return Err(anyhow!(
                "essential pipeline report map encountered unsupported stage domain `{other}` for `{stage_id}`"
            ))
        }
    };
    section.cloned().ok_or_else(|| {
        anyhow!("essential pipeline report map is missing a report section for `{stage_id}`")
    })
}

fn failure_column_for_stage(stage_id: &str) -> Result<&'static str> {
    match stage_domain(stage_id) {
        "fastq" | "bam" | "vcf" => Ok("failure_reason"),
        other => Err(anyhow!(
            "essential pipeline report map encountered unsupported stage domain `{other}` for `{stage_id}`"
        )),
    }
}

fn ensure_essential_pipeline_report_map_contract(
    rows: &[EssentialPipelineReportMapRow],
    expected_output_count: usize,
) -> Result<()> {
    let unique_rows = rows
        .iter()
        .map(|row| {
            format!("{}:{}:{}:{}", row.pipeline_id, row.stage_id, row.tool_id, row.output_metric)
        })
        .collect::<BTreeSet<_>>();
    if unique_rows.len() != rows.len() {
        return Err(anyhow!(
            "essential pipeline report map must keep one row per pipeline stage-tool-output binding"
        ));
    }
    if rows.len() != expected_output_count {
        return Err(anyhow!(
            "essential pipeline report map must cover every declared pipeline output, expected {expected_output_count} rows, found {}",
            rows.len()
        ));
    }

    for row in rows {
        if row.pipeline_id.trim().is_empty()
            || row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.output_metric.trim().is_empty()
            || row.report_section.trim().is_empty()
            || row.failure_column.trim().is_empty()
        {
            return Err(anyhow!(
                "essential pipeline report map rows must keep non-empty pipeline, stage, tool, output, section, and failure columns"
            ));
        }
    }

    Ok(())
}

fn render_essential_pipeline_report_map_tsv(rows: &[EssentialPipelineReportMapRow]) -> String {
    let mut rendered = String::from(
        "pipeline_id\tstage_id\ttool_id\toutput_metric\treport_section\tfailure_column\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.pipeline_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.output_metric),
            sanitize_tsv(&row.report_section),
            sanitize_tsv(&row.failure_column),
        ));
    }
    rendered
}

fn stage_domain(stage_id: &str) -> &'static str {
    if stage_id.starts_with("fastq.") {
        "fastq"
    } else if stage_id.starts_with("bam.") {
        "bam"
    } else if stage_id.starts_with("vcf.") {
        "vcf"
    } else {
        "unknown"
    }
}

fn sanitize_tsv(value: &str) -> String {
    value.replace('\t', " ").replace('\n', " ")
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
