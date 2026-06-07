use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::contracts::stage_metrics_contract;
use bijux_dna_domain_vcf::{find_vcf_parser_fixture_inventory_row, VcfDomainStage};
use serde::Serialize;

use super::vcf_tool_serving_map::collect_vcf_tool_serving_map_rows;
use crate::commands::benchmark::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_PARSER_COVERAGE_PATH: &str =
    "benchmarks/readiness/vcf-parser-coverage.tsv";
const VCF_PARSER_COVERAGE_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_parser_coverage.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum VcfParserCoverageStatus {
    Covered,
    MissingFixture,
    MissingSchema,
    MissingInventory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfParserCoverageRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) parser_id: String,
    pub(crate) fixture_path: String,
    pub(crate) schema_id: String,
    pub(crate) coverage_status: VcfParserCoverageStatus,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfParserCoverageReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) row_count: usize,
    pub(crate) covered_row_count: usize,
    pub(crate) missing_row_count: usize,
    pub(crate) parser_coverage_percent: f64,
    pub(crate) coverage_status_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<VcfParserCoverageRow>,
}

pub(crate) fn run_render_vcf_parser_coverage(
    args: &parse::BenchReadinessRenderVcfParserCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_parser_coverage(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_PARSER_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_parser_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfParserCoverageReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let (stage_count, tool_count, rows) = collect_vcf_parser_coverage_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_vcf_parser_coverage_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let covered_row_count =
        rows.iter().filter(|row| row.coverage_status == VcfParserCoverageStatus::Covered).count();
    let missing_row_count = rows.len().saturating_sub(covered_row_count);
    let parser_coverage_percent =
        if rows.is_empty() { 0.0 } else { covered_row_count as f64 * 100.0 / rows.len() as f64 };
    let mut coverage_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *coverage_status_counts
            .entry(coverage_status_label(row.coverage_status).to_string())
            .or_default() += 1;
    }

    if missing_row_count != 0 {
        let missing_rows = rows
            .iter()
            .filter(|row| row.coverage_status != VcfParserCoverageStatus::Covered)
            .map(|row| {
                format!(
                    "{}:{}:{}",
                    row.stage_id,
                    row.tool_id,
                    coverage_status_label(row.coverage_status)
                )
            })
            .collect::<Vec<_>>();
        return Err(anyhow!(
            "VCF parser coverage must be complete for every benchmark-ready row, missing coverage for: {}",
            missing_rows.join(", ")
        ));
    }

    Ok(VcfParserCoverageReport {
        schema_version: VCF_PARSER_COVERAGE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        tool_count,
        row_count: rows.len(),
        covered_row_count,
        missing_row_count,
        parser_coverage_percent,
        coverage_status_counts,
        rows,
    })
}

pub(crate) fn collect_vcf_parser_coverage_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<VcfParserCoverageRow>)> {
    let benchmark_ready_rows = collect_vcf_tool_serving_map_rows()?
        .into_iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .collect::<Vec<_>>();
    let matrix_rows = build_vcf_stage_matrix_rows()?
        .into_iter()
        .map(|row| ((row.stage_id.clone(), row.tool_id.clone()), row))
        .collect::<BTreeMap<_, _>>();

    let stage_count =
        benchmark_ready_rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let tool_count =
        benchmark_ready_rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();

    let mut rows = Vec::with_capacity(benchmark_ready_rows.len());
    for serving_row in benchmark_ready_rows {
        let pair_key = (serving_row.stage_id.clone(), serving_row.tool_id.clone());
        let _matrix_row = matrix_rows.get(&pair_key).ok_or_else(|| {
            anyhow!(
                "VCF parser coverage is missing stage-matrix coverage for `{}` / `{}`",
                serving_row.stage_id,
                serving_row.tool_id
            )
        })?;
        let stage = VcfDomainStage::try_from(serving_row.stage_id.as_str())
            .map_err(|error| anyhow!("unknown VCF stage `{}`: {error}", serving_row.stage_id))?;
        let schema_id = stage_metrics_contract(stage).metrics_schema_id.to_string();
        let fixture_row = find_vcf_parser_fixture_inventory_row(&serving_row.tool_id, stage);

        let (parser_id, fixture_path, coverage_status) = match fixture_row {
            Some(fixture_row) => {
                let fixture_exists = repo_root.join(fixture_row.fixture_path).exists();
                let status = if !fixture_exists {
                    VcfParserCoverageStatus::MissingFixture
                } else if schema_id.trim().is_empty() {
                    VcfParserCoverageStatus::MissingSchema
                } else {
                    VcfParserCoverageStatus::Covered
                };
                (fixture_row.parser_id.to_string(), fixture_row.fixture_path.to_string(), status)
            }
            None => (String::new(), String::new(), VcfParserCoverageStatus::MissingInventory),
        };

        rows.push(VcfParserCoverageRow {
            stage_id: serving_row.stage_id,
            tool_id: serving_row.tool_id,
            parser_id,
            fixture_path,
            schema_id,
            coverage_status,
        });
    }

    rows.sort_by(|left, right| {
        left.stage_id
            .cmp(&right.stage_id)
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.parser_id.cmp(&right.parser_id))
    });
    Ok((stage_count, tool_count, rows))
}

fn render_vcf_parser_coverage_tsv(rows: &[VcfParserCoverageRow]) -> String {
    let mut rendered =
        String::from("stage_id\ttool_id\tparser_id\tfixture_path\tschema_id\tcoverage_status\n");
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.parser_id),
            sanitize_tsv(&row.fixture_path),
            sanitize_tsv(&row.schema_id),
            sanitize_tsv(coverage_status_label(row.coverage_status)),
        ));
    }
    rendered
}

fn coverage_status_label(status: VcfParserCoverageStatus) -> &'static str {
    match status {
        VcfParserCoverageStatus::Covered => "covered",
        VcfParserCoverageStatus::MissingFixture => "missing_fixture",
        VcfParserCoverageStatus::MissingSchema => "missing_schema",
        VcfParserCoverageStatus::MissingInventory => "missing_inventory",
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

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        coverage_status_label, render_vcf_parser_coverage, VcfParserCoverageStatus,
        DEFAULT_VCF_PARSER_COVERAGE_PATH, VCF_PARSER_COVERAGE_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_vcf_parser_coverage_reports_benchmark_ready_rows() {
        let root = repo_root();
        let report =
            render_vcf_parser_coverage(&root, PathBuf::from(DEFAULT_VCF_PARSER_COVERAGE_PATH))
                .expect("render VCF parser coverage");

        assert_eq!(report.schema_version, VCF_PARSER_COVERAGE_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_VCF_PARSER_COVERAGE_PATH);
        assert_eq!(report.stage_count, 9);
        assert_eq!(report.tool_count, 1);
        assert_eq!(report.row_count, 9);
        assert_eq!(report.covered_row_count, 9);
        assert_eq!(report.missing_row_count, 0);
        assert_eq!(report.parser_coverage_percent, 100.0);
        assert_eq!(report.coverage_status_counts.get("covered"), Some(&9));
        assert!(report.rows.iter().all(|row| {
            row.tool_id == "bcftools"
                && row
                    .fixture_path
                    .starts_with("benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/")
                && row.schema_id.starts_with("bijux.vcf.")
                && coverage_status_label(row.coverage_status) == "covered"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.call"
                && row.parser_id == "parse_bcftools_call_metrics"
                && row.schema_id == "bijux.vcf.call.v1"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.call_gl"
                && row.parser_id == "parse_bcftools_call_gl_metrics"
                && row.schema_id == "bijux.vcf.call_gl.v1"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.postprocess"
                && row.parser_id == "parse_bcftools_postprocess_metrics"
                && row.schema_id == "bijux.vcf.postprocess.v1"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.stats"
                && row.parser_id == "parse_bcftools_stats_metrics"
                && row.schema_id == "bijux.vcf.stats.v1"
        }));
    }

    #[test]
    fn coverage_status_labels_are_stable() {
        assert_eq!(coverage_status_label(VcfParserCoverageStatus::Covered), "covered");
        assert_eq!(
            coverage_status_label(VcfParserCoverageStatus::MissingFixture),
            "missing_fixture"
        );
        assert_eq!(coverage_status_label(VcfParserCoverageStatus::MissingSchema), "missing_schema");
        assert_eq!(
            coverage_status_label(VcfParserCoverageStatus::MissingInventory),
            "missing_inventory"
        );
    }
}
