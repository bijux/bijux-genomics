use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_stage_tool_matrix::collect_all_domain_active_stage_tool_matrix_rows;
use super::bam_normalized_metrics_schema::collect_bam_normalized_metrics_schema_report_rows;
use super::bam_parser_coverage::collect_bam_parser_coverage_rows;
use super::bam_report_map::collect_bam_report_map_rows;
use super::fastq_normalized_metrics_schema::collect_fastq_normalized_metrics_schema_report_rows;
use super::fastq_parser_coverage::collect_fastq_parser_coverage_rows;
use super::fastq_report_map::collect_fastq_report_stage_metadata;
use super::vcf_normalized_metrics_schema::collect_vcf_normalized_metrics_schema_report_rows;
use super::vcf_parser_fixture_coverage::{
    collect_vcf_parser_fixture_coverage_rows, VcfParserFixtureCoverageStatus,
};
use super::vcf_report_map::collect_vcf_report_map_rows;
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_CATALOG_PATH: &str =
    "benchmarks/readiness/all-domains/active-stage-catalog.tsv";
const ALL_DOMAIN_ACTIVE_STAGE_CATALOG_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_active_stage_catalog.v1";
const NO_VALUES: &str = "none";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainActiveStageCatalogRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) active_tool_count: usize,
    pub(crate) benchmark_ready_tool_count: usize,
    pub(crate) parser_row_count: usize,
    pub(crate) parser_covered_row_count: usize,
    pub(crate) schema_present: bool,
    pub(crate) report_row_count: usize,
    pub(crate) benchmark_statuses: Vec<String>,
    pub(crate) active_tool_ids: Vec<String>,
    pub(crate) benchmark_ready_tool_ids: Vec<String>,
    pub(crate) report_section_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainActiveStageCatalogReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) stages_with_benchmark_ready_tools: usize,
    pub(crate) not_benchmark_ready_only_stage_count: usize,
    pub(crate) stages_with_parser_rows: usize,
    pub(crate) stages_with_schema: usize,
    pub(crate) stages_with_report_rows: usize,
    pub(crate) rows: Vec<AllDomainActiveStageCatalogRow>,
}

#[derive(Default)]
struct StageAccumulator {
    readiness_kind: Option<String>,
    active_tool_ids: BTreeSet<String>,
    benchmark_ready_tool_ids: BTreeSet<String>,
    benchmark_statuses: BTreeSet<String>,
    parser_row_count: usize,
    parser_covered_row_count: usize,
    schema_present: bool,
    report_section_ids: BTreeSet<String>,
    report_row_count: usize,
}

pub(crate) fn run_render_all_domain_active_stage_catalog(
    args: &parse::BenchReadinessRenderAllDomainActiveStageCatalogArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_active_stage_catalog(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_CATALOG_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_active_stage_catalog(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainActiveStageCatalogReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_all_domain_active_stage_catalog_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_all_domain_active_stage_catalog_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let row_count = rows.len();
    let stages_with_benchmark_ready_tools =
        rows.iter().filter(|row| row.benchmark_ready_tool_count > 0).count();
    let not_benchmark_ready_only_stage_count =
        rows.iter().filter(|row| row.benchmark_ready_tool_count == 0).count();
    let stages_with_parser_rows = rows.iter().filter(|row| row.parser_row_count > 0).count();
    let stages_with_schema = rows.iter().filter(|row| row.schema_present).count();
    let stages_with_report_rows = rows.iter().filter(|row| row.report_row_count > 0).count();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
    }

    Ok(AllDomainActiveStageCatalogReport {
        schema_version: ALL_DOMAIN_ACTIVE_STAGE_CATALOG_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count,
        domain_counts,
        stages_with_benchmark_ready_tools,
        not_benchmark_ready_only_stage_count,
        stages_with_parser_rows,
        stages_with_schema,
        stages_with_report_rows,
        rows,
    })
}

pub(crate) fn collect_all_domain_active_stage_catalog_rows(
    repo_root: &Path,
) -> Result<Vec<AllDomainActiveStageCatalogRow>> {
    let inventory_readiness_by_stage = collect_inventory_readiness_by_stage(repo_root)?;
    let stage_tool_rows = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?;
    let parser_stage_rows = collect_parser_stage_rows(repo_root)?;
    let schema_stage_rows = collect_schema_stage_rows()?;
    let report_stage_rows = collect_report_stage_rows(repo_root)?;

    let mut by_stage = BTreeMap::<(String, String), StageAccumulator>::new();
    for row in stage_tool_rows {
        let entry = by_stage.entry((row.domain.clone(), row.stage_id.clone())).or_default();
        entry.readiness_kind = Some(
            inventory_readiness_by_stage
                .get(&(row.domain.clone(), row.stage_id.clone()))
                .cloned()
                .ok_or_else(|| {
                    anyhow!(
                        "all-domain active stage catalog is missing config-backed readiness kind for `{}` / `{}`",
                        row.domain,
                        row.stage_id
                    )
                })?,
        );
        entry.active_tool_ids.insert(row.tool_id.clone());
        entry.benchmark_statuses.insert(row.status.clone());
        if row.status == "benchmark_ready" {
            entry.benchmark_ready_tool_ids.insert(row.tool_id);
        }
    }

    for parser_row in &parser_stage_rows {
        if let Some(entry) =
            by_stage.get_mut(&(parser_row.domain.clone(), parser_row.stage_id.clone()))
        {
            entry.parser_row_count += 1;
            if parser_row.covered {
                entry.parser_covered_row_count += 1;
            }
        }
    }

    for (domain, stage_id) in &schema_stage_rows {
        if let Some(entry) = by_stage.get_mut(&(domain.clone(), stage_id.clone())) {
            entry.schema_present = true;
        }
    }

    for report_row in &report_stage_rows {
        if let Some(entry) =
            by_stage.get_mut(&(report_row.domain.clone(), report_row.stage_id.clone()))
        {
            entry.report_row_count += 1;
            entry.report_section_ids.insert(report_row.report_section_id.clone());
        }
    }

    let rows = by_stage
        .into_iter()
        .map(|((domain, stage_id), acc)| {
            let readiness_kind = acc.readiness_kind.ok_or_else(|| {
                anyhow!(
                    "all-domain active stage catalog is missing config-backed readiness kind for `{}` / `{}`",
                    domain,
                    stage_id
                )
            })?;
            Ok(AllDomainActiveStageCatalogRow {
                domain,
                stage_id,
                readiness_kind,
                active_tool_count: acc.active_tool_ids.len(),
                benchmark_ready_tool_count: acc.benchmark_ready_tool_ids.len(),
                parser_row_count: acc.parser_row_count,
                parser_covered_row_count: acc.parser_covered_row_count,
                schema_present: acc.schema_present,
                report_row_count: acc.report_row_count,
                benchmark_statuses: acc.benchmark_statuses.into_iter().collect(),
                active_tool_ids: acc.active_tool_ids.into_iter().collect(),
                benchmark_ready_tool_ids: acc.benchmark_ready_tool_ids.into_iter().collect(),
                report_section_ids: acc.report_section_ids.into_iter().collect(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    ensure_all_domain_active_stage_catalog_contract(
        repo_root,
        &rows,
        &parser_stage_rows,
        &schema_stage_rows,
        &report_stage_rows,
    )?;
    Ok(rows)
}

fn collect_inventory_readiness_by_stage(
    repo_root: &Path,
) -> Result<BTreeMap<(String, String), String>> {
    let mut rows = BTreeMap::new();
    for domain in [BenchLocalDomain::Fastq, BenchLocalDomain::Bam, BenchLocalDomain::Vcf] {
        let inventory = load_local_stage_inventory(repo_root, domain)?;
        for stage in inventory.stages {
            rows.insert(
                (inventory.domain.to_string(), stage.stage_id),
                stage.readiness_kind.as_str().to_string(),
            );
        }
    }
    Ok(rows)
}

fn collect_parser_stage_rows(repo_root: &Path) -> Result<Vec<ParserStageRow>> {
    let (_, _, fastq_rows) = collect_fastq_parser_coverage_rows(repo_root)?;
    let (_, _, bam_rows, _) = collect_bam_parser_coverage_rows(repo_root)?;
    let (_, _, vcf_rows) = collect_vcf_parser_fixture_coverage_rows(repo_root)?;

    let mut rows = Vec::new();
    rows.extend(fastq_rows.into_iter().map(|row| ParserStageRow {
        domain: "fastq".to_string(),
        stage_id: row.stage_id,
        covered: row.parser_coverage
            == super::fastq_parser_coverage::FastqParserCoverageKind::Covered,
    }));
    rows.extend(bam_rows.into_iter().map(|row| ParserStageRow {
        domain: "bam".to_string(),
        stage_id: row.stage_id,
        covered: row.parser_coverage == super::bam_parser_coverage::BamParserCoverageKind::Covered,
    }));
    rows.extend(vcf_rows.into_iter().map(|row| ParserStageRow {
        domain: "vcf".to_string(),
        stage_id: row.stage_id,
        covered: row.coverage_status == VcfParserFixtureCoverageStatus::Covered,
    }));
    Ok(rows)
}

fn collect_schema_stage_rows() -> Result<BTreeSet<(String, String)>> {
    let mut rows = BTreeSet::new();
    for row in collect_fastq_normalized_metrics_schema_report_rows()? {
        rows.insert((String::from("fastq"), row.stage_id));
    }
    for row in collect_bam_normalized_metrics_schema_report_rows()? {
        rows.insert((String::from("bam"), row.stage_id));
    }
    for row in collect_vcf_normalized_metrics_schema_report_rows()? {
        rows.insert((String::from("vcf"), row.stage_id));
    }
    Ok(rows)
}

fn collect_report_stage_rows(repo_root: &Path) -> Result<Vec<ReportStageRow>> {
    let mut rows = Vec::new();
    rows.extend(collect_fastq_report_stage_metadata(repo_root)?.into_iter().map(
        |(stage_id, row)| ReportStageRow {
            domain: "fastq".to_string(),
            stage_id,
            report_section_id: row.report_section_id,
        },
    ));
    rows.extend(collect_bam_report_map_rows(repo_root)?.into_iter().map(|row| ReportStageRow {
        domain: "bam".to_string(),
        stage_id: row.stage_id,
        report_section_id: row.report_section_id,
    }));
    rows.extend(collect_vcf_report_map_rows(repo_root)?.into_iter().map(|row| ReportStageRow {
        domain: "vcf".to_string(),
        stage_id: row.stage_id,
        report_section_id: row.section_id,
    }));
    Ok(rows)
}

fn ensure_all_domain_active_stage_catalog_contract(
    repo_root: &Path,
    rows: &[AllDomainActiveStageCatalogRow],
    parser_stage_rows: &[ParserStageRow],
    schema_stage_rows: &BTreeSet<(String, String)>,
    report_stage_rows: &[ReportStageRow],
) -> Result<()> {
    let row_keys = rows
        .iter()
        .map(|row| (row.domain.as_str(), row.stage_id.as_str()))
        .collect::<BTreeSet<_>>();
    if row_keys.len() != rows.len() {
        return Err(anyhow!(
            "all-domain active stage catalog must keep exactly one row per domain/stage_id"
        ));
    }

    let stage_tool_keys = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?
        .into_iter()
        .map(|row| (row.domain, row.stage_id))
        .collect::<BTreeSet<_>>();
    let row_key_strings =
        row_keys.iter().map(|(d, s)| ((*d).to_string(), (*s).to_string())).collect::<BTreeSet<_>>();

    if row_key_strings != stage_tool_keys {
        return Err(anyhow!(
            "all-domain active stage catalog drifted from the governed active-scope stage-tool surface"
        ));
    }

    let parser_keys = parser_stage_rows
        .iter()
        .map(|row| (row.domain.clone(), row.stage_id.clone()))
        .collect::<BTreeSet<_>>();
    let report_keys = report_stage_rows
        .iter()
        .map(|row| (row.domain.clone(), row.stage_id.clone()))
        .collect::<BTreeSet<_>>();

    for row in rows {
        if row.readiness_kind.trim().is_empty()
            || row.active_tool_count == 0
            || row.active_tool_ids.is_empty()
            || row.benchmark_statuses.is_empty()
        {
            return Err(anyhow!(
                "all-domain active stage catalog row `{}` / `{}` is missing a required active-scope field",
                row.domain,
                row.stage_id
            ));
        }
        if row.active_tool_count != row.active_tool_ids.len()
            || row.benchmark_ready_tool_count != row.benchmark_ready_tool_ids.len()
        {
            return Err(anyhow!(
                "all-domain active stage catalog row `{}` / `{}` drifted from its owned tool sets",
                row.domain,
                row.stage_id
            ));
        }
        if row.benchmark_ready_tool_count > row.active_tool_count
            || row.parser_covered_row_count > row.parser_row_count
        {
            return Err(anyhow!(
                "all-domain active stage catalog row `{}` / `{}` overcounts benchmark-ready or parser-covered scope",
                row.domain,
                row.stage_id
            ));
        }
        let row_key = (row.domain.clone(), row.stage_id.clone());
        if row.parser_row_count > 0 && !parser_keys.contains(&row_key) {
            return Err(anyhow!(
                "all-domain active stage catalog row `{}` / `{}` drifted from the governed parser coverage surface",
                row.domain,
                row.stage_id
            ));
        }
        if row.schema_present && !schema_stage_rows.contains(&row_key) {
            return Err(anyhow!(
                "all-domain active stage catalog row `{}` / `{}` drifted from the governed normalized metrics surface",
                row.domain,
                row.stage_id
            ));
        }
        if row.report_row_count > 0 && !report_keys.contains(&row_key) {
            return Err(anyhow!(
                "all-domain active stage catalog row `{}` / `{}` drifted from the governed report-map surface",
                row.domain,
                row.stage_id
            ));
        }
    }

    Ok(())
}

fn render_all_domain_active_stage_catalog_tsv(rows: &[AllDomainActiveStageCatalogRow]) -> String {
    let mut rendered = String::from(
        "domain\tstage_id\treadiness_kind\tactive_tool_count\tbenchmark_ready_tool_count\tparser_row_count\tparser_covered_row_count\tschema_present\treport_row_count\tbenchmark_statuses\tactive_tool_ids\tbenchmark_ready_tool_ids\treport_section_ids\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.domain,
            row.stage_id,
            row.readiness_kind,
            row.active_tool_count,
            row.benchmark_ready_tool_count,
            row.parser_row_count,
            row.parser_covered_row_count,
            row.schema_present,
            row.report_row_count,
            row.benchmark_statuses.join(","),
            row.active_tool_ids.join(","),
            joined_values_or_none(&row.benchmark_ready_tool_ids),
            joined_values_or_none(&row.report_section_ids),
        ));
    }
    rendered
}

fn joined_values_or_none(values: &[String]) -> String {
    if values.is_empty() {
        NO_VALUES.to_string()
    } else {
        values.join(",")
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

struct ParserStageRow {
    domain: String,
    stage_id: String,
    covered: bool,
}

struct ReportStageRow {
    domain: String,
    stage_id: String,
    report_section_id: String,
}
