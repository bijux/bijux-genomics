use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::{find_vcf_parser_fixture_inventory_row, VcfDomainStage};
use serde::Serialize;

use super::vcf_expected_benchmark_results::collect_vcf_expected_benchmark_result_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_REPORT_MAP_PATH: &str = "benchmarks/readiness/vcf-report-map.tsv";
const VCF_REPORT_MAP_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_report_map.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfReportMapRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) section_id: String,
    pub(crate) summary_table: String,
    pub(crate) metric_columns: Vec<String>,
    pub(crate) failure_columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfReportMapReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) section_count: usize,
    pub(crate) summary_table_count: usize,
    pub(crate) section_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<VcfReportMapRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VcfReportPlacement {
    section_id: &'static str,
    summary_table: &'static str,
}

pub(crate) fn run_render_vcf_report_map(
    args: &parse::BenchReadinessRenderVcfReportMapArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_report_map(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_REPORT_MAP_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_report_map(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfReportMapReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_vcf_report_map_rows(repo_root)?;
    let stage_count = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let section_count =
        rows.iter().map(|row| row.section_id.as_str()).collect::<BTreeSet<_>>().len();
    let summary_table_count =
        rows.iter().map(|row| row.summary_table.as_str()).collect::<BTreeSet<_>>().len();
    let mut section_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *section_counts.entry(row.section_id.clone()).or_default() += 1;
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_vcf_report_map_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(VcfReportMapReport {
        schema_version: VCF_REPORT_MAP_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        stage_count,
        tool_count,
        section_count,
        summary_table_count,
        section_counts,
        rows,
    })
}

pub(crate) fn collect_vcf_report_map_rows(repo_root: &Path) -> Result<Vec<VcfReportMapRow>> {
    let expected_rows = collect_vcf_expected_benchmark_result_rows(repo_root)?;
    let mut rows = Vec::with_capacity(expected_rows.len());

    for row in expected_rows {
        let stage = VcfDomainStage::try_from(row.stage_id.as_str())
            .map_err(|error| anyhow!("unknown VCF stage `{}`: {error}", row.stage_id))?;
        let placement = placement_for_section(row.report_section.as_str()).ok_or_else(|| {
            anyhow!(
                "VCF report map is missing placement for benchmark section `{}`",
                row.report_section
            )
        })?;
        let parser_fixture = find_vcf_parser_fixture_inventory_row(&row.tool_id, stage)
            .ok_or_else(|| {
                anyhow!(
                    "VCF report map is missing parser fixture inventory for `{}` / `{}`",
                    row.stage_id,
                    row.tool_id
                )
            })?;

        if parser_fixture.fixture_path.trim().is_empty() {
            return Err(anyhow!(
                "VCF report map parser fixture inventory for `{}` / `{}` is missing a fixture path",
                row.stage_id,
                row.tool_id
            ));
        }

        rows.push(VcfReportMapRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            section_id: placement.section_id.to_string(),
            summary_table: placement.summary_table.to_string(),
            metric_columns: row.expected_metrics,
            failure_columns: failure_columns_for_stage(stage),
        });
    }

    rows.sort_by(|left, right| {
        left.stage_id
            .cmp(&right.stage_id)
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.section_id.cmp(&right.section_id))
    });
    ensure_vcf_report_map_contract(&rows)?;
    Ok(rows)
}

fn placement_for_section(section_id: &str) -> Option<VcfReportPlacement> {
    match section_id {
        "reference_panel_preparation" => Some(VcfReportPlacement {
            section_id: "reference_panel_preparation",
            summary_table: "reference_panel_readiness",
        }),
        "variant_calling" => Some(VcfReportPlacement {
            section_id: "variant_calling",
            summary_table: "variant_calling_metrics",
        }),
        "damage_aware_filtering" => Some(VcfReportPlacement {
            section_id: "damage_aware_filtering",
            summary_table: "damage_filtering_metrics",
        }),
        "quality_control" => Some(VcfReportPlacement {
            section_id: "quality_control",
            summary_table: "quality_control_metrics",
        }),
        "likelihood_postprocess" => Some(VcfReportPlacement {
            section_id: "likelihood_postprocess",
            summary_table: "likelihood_postprocess_metrics",
        }),
        "phasing" => {
            Some(VcfReportPlacement { section_id: "phasing", summary_table: "phasing_metrics" })
        }
        "imputation" => Some(VcfReportPlacement {
            section_id: "imputation",
            summary_table: "imputation_metrics",
        }),
        "normalization" => Some(VcfReportPlacement {
            section_id: "normalization",
            summary_table: "normalization_metrics",
        }),
        "population_structure" => Some(VcfReportPlacement {
            section_id: "population_structure",
            summary_table: "population_structure_metrics",
        }),
        "runs_of_homozygosity" => Some(VcfReportPlacement {
            section_id: "runs_of_homozygosity",
            summary_table: "roh_metrics",
        }),
        "identity_by_descent" => Some(VcfReportPlacement {
            section_id: "identity_by_descent",
            summary_table: "ibd_metrics",
        }),
        "demography" => Some(VcfReportPlacement {
            section_id: "demography",
            summary_table: "demography_metrics",
        }),
        _ => None,
    }
}

fn failure_columns_for_stage(stage: VcfDomainStage) -> Vec<String> {
    let mut columns = vec![
        "result_status".to_string(),
        "reason".to_string(),
        "parser_id".to_string(),
        "failure_reason".to_string(),
        "observed_error".to_string(),
    ];
    if matches!(stage, VcfDomainStage::Postprocess | VcfDomainStage::PrepareReferencePanel) {
        columns.push("audit_manifest_path".to_string());
    }
    columns
}

fn ensure_vcf_report_map_contract(rows: &[VcfReportMapRow]) -> Result<()> {
    let unique_pairs =
        rows.iter().map(|row| format!("{}:{}", row.stage_id, row.tool_id)).collect::<BTreeSet<_>>();
    if unique_pairs.len() != rows.len() {
        return Err(anyhow!(
            "VCF report map must keep one row per expected VCF stage-tool result binding"
        ));
    }
    if rows.len() != 14 {
        return Err(anyhow!(
            "VCF report map must retain exactly 14 benchmark-ready rows, found {}",
            rows.len()
        ));
    }

    for row in rows {
        if row.section_id.trim().is_empty()
            || row.summary_table.trim().is_empty()
            || row.metric_columns.is_empty()
            || row.failure_columns.is_empty()
        {
            return Err(anyhow!(
                "VCF report map row `{}` / `{}` is missing required report columns",
                row.stage_id,
                row.tool_id
            ));
        }
    }

    let section_count =
        rows.iter().map(|row| row.section_id.as_str()).collect::<BTreeSet<_>>().len();
    let summary_table_count =
        rows.iter().map(|row| row.summary_table.as_str()).collect::<BTreeSet<_>>().len();
    if section_count != 7 || summary_table_count != 7 {
        return Err(anyhow!(
            "VCF report map must retain 7 sections and 7 summary tables for the governed ready slice, found {section_count} sections and {summary_table_count} tables"
        ));
    }

    require_row_mapping(
        rows,
        "vcf.call",
        "bcftools",
        "variant_calling",
        "variant_calling_metrics",
    )?;
    require_row_mapping(
        rows,
        "vcf.damage_filter",
        "bcftools",
        "damage_aware_filtering",
        "damage_filtering_metrics",
    )?;
    require_row_mapping(
        rows,
        "vcf.filter",
        "bcftools",
        "quality_control",
        "quality_control_metrics",
    )?;
    require_row_mapping(rows, "vcf.qc", "bcftools", "quality_control", "quality_control_metrics")?;
    require_row_mapping(
        rows,
        "vcf.gl_propagation",
        "bcftools",
        "likelihood_postprocess",
        "likelihood_postprocess_metrics",
    )?;
    require_row_mapping(
        rows,
        "vcf.postprocess",
        "bcftools",
        "normalization",
        "normalization_metrics",
    )?;
    require_row_mapping(
        rows,
        "vcf.prepare_reference_panel",
        "bcftools",
        "reference_panel_preparation",
        "reference_panel_readiness",
    )?;

    Ok(())
}

fn require_row_mapping(
    rows: &[VcfReportMapRow],
    stage_id: &str,
    tool_id: &str,
    section_id: &str,
    summary_table: &str,
) -> Result<()> {
    let row = rows
        .iter()
        .find(|row| row.stage_id == stage_id && row.tool_id == tool_id)
        .ok_or_else(|| anyhow!("VCF report map is missing `{stage_id}` / `{tool_id}`"))?;
    if row.section_id != section_id || row.summary_table != summary_table {
        return Err(anyhow!(
            "VCF report map row `{stage_id}` / `{tool_id}` drifted from `{section_id}` / `{summary_table}`"
        ));
    }
    Ok(())
}

fn render_vcf_report_map_tsv(rows: &[VcfReportMapRow]) -> String {
    let mut rendered = String::from(
        "stage_id\ttool_id\tsection_id\tsummary_table\tmetric_columns\tfailure_columns\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.section_id),
            sanitize_tsv(&row.summary_table),
            sanitize_tsv(&row.metric_columns.join(",")),
            sanitize_tsv(&row.failure_columns.join(",")),
        ));
    }
    rendered
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

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_vcf_report_map, DEFAULT_VCF_REPORT_MAP_PATH, VCF_REPORT_MAP_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_vcf_report_map_reports_expected_result_sections() {
        let root = repo_root();
        let report = render_vcf_report_map(&root, PathBuf::from(DEFAULT_VCF_REPORT_MAP_PATH))
            .expect("render VCF report map");

        assert_eq!(report.schema_version, VCF_REPORT_MAP_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_VCF_REPORT_MAP_PATH);
        assert_eq!(report.row_count, 14);
        assert_eq!(report.stage_count, 12);
        assert_eq!(report.tool_count, 4);
        assert_eq!(report.section_count, 7);
        assert_eq!(report.summary_table_count, 7);
        assert_eq!(report.section_counts.get("variant_calling"), Some(&4));
        assert_eq!(report.section_counts.get("quality_control"), Some(&5));
        assert_eq!(report.section_counts.get("normalization"), Some(&1));
        assert_eq!(report.section_counts.get("reference_panel_preparation"), Some(&1));

        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.call"
                && row.tool_id == "bcftools"
                && row.section_id == "variant_calling"
                && row.summary_table == "variant_calling_metrics"
                && row.metric_columns
                    == vec!["variant_count", "snp_count", "indel_count", "sample_count"]
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.gl_propagation"
                && row.section_id == "likelihood_postprocess"
                && row.summary_table == "likelihood_postprocess_metrics"
                && row.failure_columns.iter().any(|value| value == "observed_error")
        }));
    }
}
