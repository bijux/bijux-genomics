use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::contracts::stage_metrics_contract;
use bijux_dna_domain_vcf::{find_vcf_parser_fixture_inventory_row, VcfDomainStage};
use serde::Serialize;

use super::vcf_active_stage_tool_matrix::collect_vcf_active_stage_tool_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH: &str =
    "benchmarks/readiness/vcf/vcf-parser-fixture-coverage.tsv";
const VCF_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_parser_fixture_coverage.v1";
const VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION: &str = "bijux.fixture.vcf_raw_parser.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum VcfParserFixtureCoverageStatus {
    Covered,
    MissingFixtureInventory,
    MissingFixtureDirectory,
    MissingExpectedNormalizedJson,
    MissingRawFixtures,
    InvalidExpectedNormalizedJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfParserFixtureCoverageRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) schema_id: String,
    pub(crate) parser_fixture_parser_id: String,
    pub(crate) parser_fixture_schema_id: String,
    pub(crate) parser_fixture_root_path: String,
    pub(crate) expected_normalized_path: String,
    pub(crate) raw_fixture_count: usize,
    pub(crate) raw_fixture_paths: Vec<String>,
    pub(crate) coverage_status: VcfParserFixtureCoverageStatus,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfParserFixtureCoverageReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) row_count: usize,
    pub(crate) covered_row_count: usize,
    pub(crate) missing_row_count: usize,
    pub(crate) parser_fixture_coverage_percent: f64,
    pub(crate) coverage_status_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<VcfParserFixtureCoverageRow>,
}

pub(crate) fn run_render_vcf_parser_fixture_coverage(
    args: &parse::BenchReadinessRenderVcfParserFixtureCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_parser_fixture_coverage(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_parser_fixture_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfParserFixtureCoverageReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let (stage_count, tool_count, rows) = collect_vcf_parser_fixture_coverage_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_vcf_parser_fixture_coverage_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let covered_row_count = rows
        .iter()
        .filter(|row| row.coverage_status == VcfParserFixtureCoverageStatus::Covered)
        .count();
    let missing_row_count = rows.len().saturating_sub(covered_row_count);
    let parser_fixture_coverage_percent =
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
            .filter(|row| row.coverage_status != VcfParserFixtureCoverageStatus::Covered)
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
            "VCF parser fixture coverage must be complete for every active VCF row, missing coverage for: {}",
            missing_rows.join(", ")
        ));
    }

    Ok(VcfParserFixtureCoverageReport {
        schema_version: VCF_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        tool_count,
        row_count: rows.len(),
        covered_row_count,
        missing_row_count,
        parser_fixture_coverage_percent,
        coverage_status_counts,
        rows,
    })
}

pub(crate) fn collect_vcf_parser_fixture_coverage_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<VcfParserFixtureCoverageRow>)> {
    let active_rows = collect_vcf_active_stage_tool_matrix_rows(repo_root)?
        .into_iter()
        .filter(|row| row.scope_state == "active")
        .collect::<Vec<_>>();

    let stage_count =
        active_rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let tool_count =
        active_rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();

    let mut rows = Vec::with_capacity(active_rows.len());
    for active_row in active_rows {
        let stage = VcfDomainStage::try_from(active_row.stage_id.as_str())
            .map_err(|error| anyhow!("unknown VCF stage `{}`: {error}", active_row.stage_id))?;
        let parser_fixture_schema_id = stage_metrics_contract(stage).metrics_schema_id.to_string();
        let fixture_row = find_vcf_parser_fixture_inventory_row(&active_row.tool_id, stage);

        let mut parser_fixture_parser_id = String::new();
        let mut parser_fixture_root_path = String::new();
        let mut expected_normalized_path = String::new();
        let mut raw_fixture_count = 0usize;
        let mut raw_fixture_paths = Vec::new();

        let (coverage_status, reason) = match fixture_row {
            Some(fixture_row) => {
                parser_fixture_parser_id = fixture_row.parser_id.to_string();
                parser_fixture_root_path = fixture_row.fixture_path.to_string();
                expected_normalized_path =
                    format!("{}/expected.normalized.json", fixture_row.fixture_path);
                let fixture_root = repo_root.join(fixture_row.fixture_path);
                if !fixture_root.is_dir() {
                    (
                        VcfParserFixtureCoverageStatus::MissingFixtureDirectory,
                        format!(
                            "active row `{}` / `{}` expects parser fixture directory `{}` but it is missing",
                            active_row.stage_id, active_row.tool_id, fixture_row.fixture_path
                        ),
                    )
                } else {
                    raw_fixture_paths = collect_raw_fixture_paths(repo_root, &fixture_root)?;
                    raw_fixture_count = raw_fixture_paths.len();
                    if raw_fixture_count == 0 {
                        (
                            VcfParserFixtureCoverageStatus::MissingRawFixtures,
                            format!(
                                "active row `{}` / `{}` fixture directory `{}` does not contain governed `raw.*` files",
                                active_row.stage_id, active_row.tool_id, fixture_row.fixture_path
                            ),
                        )
                    } else {
                        let expected_path = fixture_root.join("expected.normalized.json");
                        if !expected_path.is_file() {
                            (
                                VcfParserFixtureCoverageStatus::MissingExpectedNormalizedJson,
                                format!(
                                    "active row `{}` / `{}` fixture directory `{}` is missing `expected.normalized.json`",
                                    active_row.stage_id, active_row.tool_id, fixture_row.fixture_path
                                ),
                            )
                        } else {
                            match validate_expected_normalized_fixture(
                                &expected_path,
                                &active_row.stage_id,
                                &active_row.tool_id,
                                fixture_row.parser_id,
                            ) {
                                Ok(()) => (
                                    VcfParserFixtureCoverageStatus::Covered,
                                    format!(
                                        "active row `{}` / `{}` keeps {} governed raw fixture file(s) under `{}` plus expected normalized JSON `{}`",
                                        active_row.stage_id,
                                        active_row.tool_id,
                                        raw_fixture_count,
                                        fixture_row.fixture_path,
                                        expected_normalized_path
                                    ),
                                ),
                                Err(error) => (
                                    VcfParserFixtureCoverageStatus::InvalidExpectedNormalizedJson,
                                    format!(
                                        "active row `{}` / `{}` has invalid expected normalized JSON `{}`: {}",
                                        active_row.stage_id,
                                        active_row.tool_id,
                                        expected_normalized_path,
                                        error
                                    ),
                                ),
                            }
                        }
                    }
                }
            }
            None => (
                VcfParserFixtureCoverageStatus::MissingFixtureInventory,
                format!(
                    "active row `{}` / `{}` is missing a governed VCF parser fixture inventory binding",
                    active_row.stage_id, active_row.tool_id
                ),
            ),
        };

        rows.push(VcfParserFixtureCoverageRow {
            stage_id: active_row.stage_id,
            tool_id: active_row.tool_id,
            corpus_id: active_row.corpus_id,
            asset_profile_id: active_row.asset_profile_id,
            adapter_id: active_row.adapter_id,
            parser_id: active_row.parser_id,
            schema_id: active_row.schema_id,
            parser_fixture_parser_id,
            parser_fixture_schema_id,
            parser_fixture_root_path,
            expected_normalized_path,
            raw_fixture_count,
            raw_fixture_paths,
            coverage_status,
            reason,
        });
    }

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    Ok((stage_count, tool_count, rows))
}

fn render_vcf_parser_fixture_coverage_tsv(rows: &[VcfParserFixtureCoverageRow]) -> String {
    let mut rendered = String::from(
        "stage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tparser_fixture_parser_id\tparser_fixture_schema_id\tparser_fixture_root_path\texpected_normalized_path\traw_fixture_count\traw_fixture_paths\tcoverage_status\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.corpus_id),
            sanitize_tsv(&row.asset_profile_id),
            sanitize_tsv(&row.adapter_id),
            sanitize_tsv(&row.parser_id),
            sanitize_tsv(&row.schema_id),
            sanitize_tsv(&row.parser_fixture_parser_id),
            sanitize_tsv(&row.parser_fixture_schema_id),
            sanitize_tsv(&row.parser_fixture_root_path),
            sanitize_tsv(&row.expected_normalized_path),
            row.raw_fixture_count,
            sanitize_tsv(&row.raw_fixture_paths.join(";")),
            sanitize_tsv(coverage_status_label(row.coverage_status)),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

pub(crate) fn coverage_status_label(status: VcfParserFixtureCoverageStatus) -> &'static str {
    match status {
        VcfParserFixtureCoverageStatus::Covered => "covered",
        VcfParserFixtureCoverageStatus::MissingFixtureInventory => "missing_fixture_inventory",
        VcfParserFixtureCoverageStatus::MissingFixtureDirectory => "missing_fixture_directory",
        VcfParserFixtureCoverageStatus::MissingExpectedNormalizedJson => {
            "missing_expected_normalized_json"
        }
        VcfParserFixtureCoverageStatus::MissingRawFixtures => "missing_raw_fixtures",
        VcfParserFixtureCoverageStatus::InvalidExpectedNormalizedJson => {
            "invalid_expected_normalized_json"
        }
    }
}

fn collect_raw_fixture_paths(repo_root: &Path, root: &Path) -> Result<Vec<String>> {
    let mut paths = Vec::new();
    collect_raw_fixture_paths_inner(repo_root, root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_raw_fixture_paths_inner(
    repo_root: &Path,
    root: &Path,
    paths: &mut Vec<String>,
) -> Result<()> {
    for entry in fs::read_dir(root).with_context(|| format!("read {}", root.display()))? {
        let entry = entry.with_context(|| format!("read entry under {}", root.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_raw_fixture_paths_inner(repo_root, &path, paths)?;
        } else if path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.starts_with("raw."))
        {
            paths.push(path_relative_to_repo(repo_root, &path));
        }
    }
    Ok(())
}

fn validate_expected_normalized_fixture(
    expected_path: &Path,
    stage_id: &str,
    tool_id: &str,
    parser_id: &str,
) -> Result<()> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    let parsed: serde_json::Value =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))?;
    let Some(object) = parsed.as_object() else {
        return Err(anyhow!("fixture root must be a JSON object"));
    };
    if object.get("schema_version").and_then(serde_json::Value::as_str)
        != Some(VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION)
    {
        return Err(anyhow!("schema_version must be `{VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION}`"));
    }
    if object.get("stage_id").and_then(serde_json::Value::as_str) != Some(stage_id) {
        return Err(anyhow!("stage_id must be `{stage_id}`"));
    }
    if object.get("tool_id").and_then(serde_json::Value::as_str) != Some(tool_id) {
        return Err(anyhow!("tool_id must be `{tool_id}`"));
    }
    if object.get("parser_id").and_then(serde_json::Value::as_str) != Some(parser_id) {
        return Err(anyhow!("parser_id must be `{parser_id}`"));
    }
    if !object.contains_key("normalized") {
        return Err(anyhow!("normalized payload is missing"));
    }
    Ok(())
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
        coverage_status_label, render_vcf_parser_fixture_coverage, VcfParserFixtureCoverageStatus,
        DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH, VCF_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_vcf_parser_fixture_coverage_reports_active_rows() {
        let root = repo_root();
        let report = render_vcf_parser_fixture_coverage(
            &root,
            PathBuf::from(DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH),
        )
        .expect("render VCF parser fixture coverage");

        assert_eq!(report.schema_version, VCF_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH);
        assert_eq!(report.stage_count, 17);
        assert_eq!(report.tool_count, 6);
        assert_eq!(report.row_count, 20);
        assert_eq!(report.covered_row_count, 20);
        assert_eq!(report.missing_row_count, 0);
        assert!((report.parser_fixture_coverage_percent - 100.0).abs() < f64::EPSILON);
        assert_eq!(report.coverage_status_counts.get("covered"), Some(&20));
        assert!(report.rows.iter().all(|row| {
            row.schema_id.starts_with("bijux.schemas.bench.vcf-normalized-metrics.")
                && row.parser_fixture_schema_id.starts_with("bijux.vcf.")
                && row.expected_normalized_path.ends_with("expected.normalized.json")
                && row.raw_fixture_count > 0
                && !row.raw_fixture_paths.is_empty()
                && coverage_status_label(row.coverage_status) == "covered"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.admixture"
                && row.tool_id == "plink2"
                && row.parser_fixture_parser_id == "parse_plink2_admixture_report"
                && row.parser_fixture_schema_id == "bijux.vcf.admixture.v1"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.call"
                && row.tool_id == "bcftools"
                && row.parser_fixture_parser_id == "parse_bcftools_call_metrics"
                && row.parser_fixture_schema_id == "bijux.vcf.call.v1"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.imputation_metrics"
                && row.tool_id == "beagle"
                && row.parser_fixture_parser_id == "parse_beagle_imputation_metrics"
                && row.expected_normalized_path.ends_with(
                    "benchmarks/tests/fixtures/bench/parsers/vcf/imputation/beagle/vcf.imputation_metrics/expected.normalized.json"
                )
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.postprocess"
                && row.tool_id == "bcftools"
                && row.parser_fixture_parser_id == "parse_bcftools_postprocess_metrics"
                && row
                    .raw_fixture_paths
                    .iter()
                    .any(|path| path.ends_with("raw.postprocess_summary.json"))
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.pca"
                && row.tool_id == "plink2"
                && row.parser_fixture_parser_id == "parse_plink2_pca_metrics"
                && row.raw_fixture_count >= 3
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.phasing"
                && row.tool_id == "shapeit5"
                && row.coverage_status == VcfParserFixtureCoverageStatus::Covered
                && row.reason.contains("expected normalized JSON")
        }));
    }

    #[test]
    fn coverage_status_labels_are_stable() {
        assert_eq!(coverage_status_label(VcfParserFixtureCoverageStatus::Covered), "covered");
        assert_eq!(
            coverage_status_label(VcfParserFixtureCoverageStatus::MissingFixtureInventory),
            "missing_fixture_inventory"
        );
        assert_eq!(
            coverage_status_label(VcfParserFixtureCoverageStatus::MissingFixtureDirectory),
            "missing_fixture_directory"
        );
        assert_eq!(
            coverage_status_label(VcfParserFixtureCoverageStatus::MissingExpectedNormalizedJson),
            "missing_expected_normalized_json"
        );
        assert_eq!(
            coverage_status_label(VcfParserFixtureCoverageStatus::MissingRawFixtures),
            "missing_raw_fixtures"
        );
        assert_eq!(
            coverage_status_label(VcfParserFixtureCoverageStatus::InvalidExpectedNormalizedJson),
            "invalid_expected_normalized_json"
        );
    }
}
