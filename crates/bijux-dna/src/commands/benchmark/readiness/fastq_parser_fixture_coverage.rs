use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_fastq::observer::{
    parse_cluster_otus_report, parse_correct_errors_report, parse_deplete_host_report,
    parse_deplete_reference_contaminants_report, parse_deplete_rrna_report,
    parse_detect_adapters_report, parse_detect_duplicates_premerge_report,
    parse_estimate_library_complexity_prealign_report, parse_extract_umis_report,
    parse_filter_low_complexity_report, parse_filter_reads_report, parse_index_reference_report,
    parse_infer_asvs_report, parse_merge_pairs_report, parse_normalize_abundance_report,
    parse_normalize_primers_report, parse_profile_overrepresented_report,
    parse_profile_read_lengths_report, parse_profile_reads_report, parse_remove_chimeras_report,
    parse_remove_duplicates_report, parse_report_qc_report, parse_screen_taxonomy_report,
    parse_terminal_damage_report, parse_trim_polyg_report, parse_trim_reads_report,
    parse_validation_report,
};
use bijux_dna_domain_fastq::{find_fastq_parser_fixture_binding, find_fastq_parser_fixture_case};
use serde::Serialize;

use super::fastq_active_stage_tool_matrix::collect_fastq_active_stage_tool_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FASTQ_PARSER_FIXTURE_COVERAGE_PATH: &str =
    "benchmarks/readiness/fastq/fastq-parser-fixture-coverage.tsv";
const FASTQ_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_parser_fixture_coverage.v1";
const FIXTURE_REFERENCE_KIND: &str = "fixture_case";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FastqParserFixtureCoverageStatus {
    Covered,
    MissingFixtureBinding,
    MissingFixtureCase,
    InvalidFixtureCase,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FastqParserFixtureCoverageRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) schema_id: String,
    pub(crate) parser_fixture_parser_id: String,
    pub(crate) parser_fixture_schema_id: String,
    pub(crate) parser_fixture_reference_kind: String,
    pub(crate) parser_fixture_reference: String,
    pub(crate) parser_fixture_surface: String,
    pub(crate) parser_fixture_canonical_tool_id: String,
    pub(crate) coverage_status: FastqParserFixtureCoverageStatus,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqParserFixtureCoverageReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) row_count: usize,
    pub(crate) covered_row_count: usize,
    pub(crate) missing_row_count: usize,
    pub(crate) parser_fixture_coverage_percent: f64,
    pub(crate) coverage_status_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<FastqParserFixtureCoverageRow>,
}

pub(crate) fn run_render_fastq_parser_fixture_coverage(
    args: &parse::BenchReadinessRenderFastqParserFixtureCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_parser_fixture_coverage(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_PARSER_FIXTURE_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_fastq_parser_fixture_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqParserFixtureCoverageReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let (stage_count, tool_count, rows) = collect_fastq_parser_fixture_coverage_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_fastq_parser_fixture_coverage_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let covered_row_count = rows
        .iter()
        .filter(|row| row.coverage_status == FastqParserFixtureCoverageStatus::Covered)
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
            .filter(|row| row.coverage_status != FastqParserFixtureCoverageStatus::Covered)
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
            "FASTQ parser fixture coverage must be complete for every active FASTQ row, missing coverage for: {}",
            missing_rows.join(", ")
        ));
    }

    Ok(FastqParserFixtureCoverageReport {
        schema_version: FASTQ_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION,
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

pub(crate) fn collect_fastq_parser_fixture_coverage_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<FastqParserFixtureCoverageRow>)> {
    let active_rows = collect_fastq_active_stage_tool_matrix_rows(repo_root)?.rows;
    let stage_count =
        active_rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let tool_count =
        active_rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();

    let mut rows = Vec::with_capacity(active_rows.len());
    for active_row in active_rows {
        let binding = find_fastq_parser_fixture_binding(&active_row.stage_id, &active_row.tool_id);
        let mut parser_fixture_parser_id = String::new();
        let mut parser_fixture_schema_id = String::new();
        let mut parser_fixture_reference = String::new();
        let mut parser_fixture_surface = String::new();
        let mut parser_fixture_canonical_tool_id = String::new();

        let (coverage_status, reason) = match binding {
            Some(binding) => {
                parser_fixture_parser_id = binding.parser_id.to_string();
                parser_fixture_schema_id = binding.parser_schema_id.to_string();
                parser_fixture_reference = binding.fixture_case_id.to_string();
                match find_fastq_parser_fixture_case(binding.fixture_case_id) {
                    Some(case) => {
                        parser_fixture_surface = case.semantic_surface.to_string();
                        parser_fixture_canonical_tool_id = case.canonical_tool_id.to_string();
                        match validate_fixture_case(binding.parser_id, binding.parser_schema_id, case) {
                            Ok(()) => (
                                FastqParserFixtureCoverageStatus::Covered,
                                format!(
                                    "active row `{}` / `{}` is governed by FASTQ parser fixture case `{}` using parser `{}` over canonical `{}` {} data",
                                    active_row.stage_id,
                                    active_row.tool_id,
                                    case.fixture_case_id,
                                    binding.parser_id,
                                    case.canonical_tool_id,
                                    case.semantic_surface
                                ),
                            ),
                            Err(error) => (
                                FastqParserFixtureCoverageStatus::InvalidFixtureCase,
                                format!(
                                    "active row `{}` / `{}` maps to invalid FASTQ parser fixture case `{}`: {}",
                                    active_row.stage_id, active_row.tool_id, case.fixture_case_id, error
                                ),
                            ),
                        }
                    }
                    None => (
                        FastqParserFixtureCoverageStatus::MissingFixtureCase,
                        format!(
                            "active row `{}` / `{}` is missing governed FASTQ parser fixture case `{}`",
                            active_row.stage_id, active_row.tool_id, binding.fixture_case_id
                        ),
                    ),
                }
            }
            None => (
                FastqParserFixtureCoverageStatus::MissingFixtureBinding,
                format!(
                    "active row `{}` / `{}` is missing a governed FASTQ parser fixture binding",
                    active_row.stage_id, active_row.tool_id
                ),
            ),
        };

        rows.push(FastqParserFixtureCoverageRow {
            stage_id: active_row.stage_id,
            tool_id: active_row.tool_id,
            corpus_id: active_row.corpus_id,
            asset_profile_id: active_row.asset_profile_id,
            adapter_id: active_row.adapter_id,
            parser_id: active_row.parser_id,
            schema_id: active_row.schema_id,
            parser_fixture_parser_id,
            parser_fixture_schema_id,
            parser_fixture_reference_kind: FIXTURE_REFERENCE_KIND.to_string(),
            parser_fixture_reference,
            parser_fixture_surface,
            parser_fixture_canonical_tool_id,
            coverage_status,
            reason,
        });
    }

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    Ok((stage_count, tool_count, rows))
}

fn validate_fixture_case(
    parser_id: &str,
    expected_schema_id: &str,
    case: bijux_dna_domain_fastq::FastqParserFixtureCase,
) -> Result<()> {
    let parsed = parse_fixture_case(parser_id, case.raw_fixture)?;
    let Some(object) = parsed.as_object() else {
        return Err(anyhow!("fixture parser output must be a JSON object"));
    };
    if object.get("schema_version").and_then(serde_json::Value::as_str) != Some(expected_schema_id)
    {
        return Err(anyhow!(
            "schema_version must be `{expected_schema_id}` but was {:?}",
            object.get("schema_version")
        ));
    }
    if object.get("stage_id").and_then(serde_json::Value::as_str) != Some(case.stage_id) {
        return Err(anyhow!("stage_id must be `{}`", case.stage_id));
    }
    if object.get("tool_id").and_then(serde_json::Value::as_str) != Some(case.canonical_tool_id) {
        return Err(anyhow!("tool_id must be `{}`", case.canonical_tool_id));
    }
    Ok(())
}

fn parse_fixture_case(parser_id: &str, raw_fixture: &str) -> Result<serde_json::Value> {
    match parser_id {
        "parse_cluster_otus_report" => {
            serde_json::to_value(parse_cluster_otus_report(raw_fixture)?)
                .context("serialize cluster otus parser output")
        }
        "parse_correct_errors_report" => {
            serde_json::to_value(parse_correct_errors_report(raw_fixture)?)
                .context("serialize correct-errors parser output")
        }
        "parse_deplete_host_report" => {
            serde_json::to_value(parse_deplete_host_report(raw_fixture)?)
                .context("serialize deplete-host parser output")
        }
        "parse_deplete_reference_contaminants_report" => {
            serde_json::to_value(parse_deplete_reference_contaminants_report(raw_fixture)?)
                .context("serialize deplete-reference-contaminants parser output")
        }
        "parse_deplete_rrna_report" => {
            serde_json::to_value(parse_deplete_rrna_report(raw_fixture)?)
                .context("serialize deplete-rrna parser output")
        }
        "parse_detect_adapters_report" => {
            serde_json::to_value(parse_detect_adapters_report(raw_fixture)?)
                .context("serialize detect-adapters parser output")
        }
        "parse_detect_duplicates_premerge_report" => {
            serde_json::to_value(parse_detect_duplicates_premerge_report(raw_fixture)?)
                .context("serialize detect-duplicates-premerge parser output")
        }
        "parse_estimate_library_complexity_prealign_report" => {
            serde_json::to_value(parse_estimate_library_complexity_prealign_report(raw_fixture)?)
                .context("serialize estimate-library-complexity-prealign parser output")
        }
        "parse_extract_umis_report" => {
            serde_json::to_value(parse_extract_umis_report(raw_fixture)?)
                .context("serialize extract-umis parser output")
        }
        "parse_filter_low_complexity_report" => {
            serde_json::to_value(parse_filter_low_complexity_report(raw_fixture)?)
                .context("serialize filter-low-complexity parser output")
        }
        "parse_filter_reads_report" => {
            serde_json::to_value(parse_filter_reads_report(raw_fixture)?)
                .context("serialize filter-reads parser output")
        }
        "parse_index_reference_report" => {
            serde_json::to_value(parse_index_reference_report(raw_fixture)?)
                .context("serialize index-reference parser output")
        }
        "parse_infer_asvs_report" => serde_json::to_value(parse_infer_asvs_report(raw_fixture)?)
            .context("serialize infer-asvs parser output"),
        "parse_merge_pairs_report" => serde_json::to_value(parse_merge_pairs_report(raw_fixture)?)
            .context("serialize merge-pairs parser output"),
        "parse_normalize_abundance_report" => {
            serde_json::to_value(parse_normalize_abundance_report(raw_fixture)?)
                .context("serialize normalize-abundance parser output")
        }
        "parse_normalize_primers_report" => {
            serde_json::to_value(parse_normalize_primers_report(raw_fixture)?)
                .context("serialize normalize-primers parser output")
        }
        "parse_profile_overrepresented_report" => {
            serde_json::to_value(parse_profile_overrepresented_report(raw_fixture)?)
                .context("serialize profile-overrepresented parser output")
        }
        "parse_profile_read_lengths_report" => {
            serde_json::to_value(parse_profile_read_lengths_report(raw_fixture)?)
                .context("serialize profile-read-lengths parser output")
        }
        "parse_profile_reads_report" => {
            serde_json::to_value(parse_profile_reads_report(raw_fixture)?)
                .context("serialize profile-reads parser output")
        }
        "parse_remove_chimeras_report" => {
            serde_json::to_value(parse_remove_chimeras_report(raw_fixture)?)
                .context("serialize remove-chimeras parser output")
        }
        "parse_remove_duplicates_report" => {
            serde_json::to_value(parse_remove_duplicates_report(raw_fixture)?)
                .context("serialize remove-duplicates parser output")
        }
        "parse_report_qc_report" => serde_json::to_value(parse_report_qc_report(raw_fixture)?)
            .context("serialize report-qc parser output"),
        "parse_screen_taxonomy_report" => {
            serde_json::to_value(parse_screen_taxonomy_report(raw_fixture)?)
                .context("serialize screen-taxonomy parser output")
        }
        "parse_terminal_damage_report" => {
            serde_json::to_value(parse_terminal_damage_report(raw_fixture)?)
                .context("serialize terminal-damage parser output")
        }
        "parse_trim_polyg_report" => serde_json::to_value(parse_trim_polyg_report(raw_fixture)?)
            .context("serialize trim-polyg parser output"),
        "parse_trim_reads_report" => serde_json::to_value(parse_trim_reads_report(raw_fixture)?)
            .context("serialize trim-reads parser output"),
        "parse_validation_report" => serde_json::to_value(parse_validation_report(raw_fixture)?)
            .context("serialize validation parser output"),
        _ => Err(anyhow!("unsupported FASTQ parser fixture parser `{parser_id}`")),
    }
}

pub(crate) fn coverage_status_label(status: FastqParserFixtureCoverageStatus) -> &'static str {
    match status {
        FastqParserFixtureCoverageStatus::Covered => "covered",
        FastqParserFixtureCoverageStatus::MissingFixtureBinding => "missing_fixture_binding",
        FastqParserFixtureCoverageStatus::MissingFixtureCase => "missing_fixture_case",
        FastqParserFixtureCoverageStatus::InvalidFixtureCase => "invalid_fixture_case",
    }
}

fn render_fastq_parser_fixture_coverage_tsv(rows: &[FastqParserFixtureCoverageRow]) -> String {
    let mut rendered = String::from(
        "stage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tparser_fixture_parser_id\tparser_fixture_schema_id\tparser_fixture_reference_kind\tparser_fixture_reference\tparser_fixture_surface\tparser_fixture_canonical_tool_id\tcoverage_status\treason\n",
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
            sanitize_tsv(&row.parser_fixture_reference_kind),
            sanitize_tsv(&row.parser_fixture_reference),
            sanitize_tsv(&row.parser_fixture_surface),
            sanitize_tsv(&row.parser_fixture_canonical_tool_id),
            sanitize_tsv(coverage_status_label(row.coverage_status)),
            sanitize_tsv(&row.reason),
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
        coverage_status_label, render_fastq_parser_fixture_coverage,
        FastqParserFixtureCoverageStatus, DEFAULT_FASTQ_PARSER_FIXTURE_COVERAGE_PATH,
        FASTQ_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_fastq_parser_fixture_coverage_reports_governed_rows() {
        let root = repo_root();
        let report = render_fastq_parser_fixture_coverage(
            &root,
            PathBuf::from(DEFAULT_FASTQ_PARSER_FIXTURE_COVERAGE_PATH),
        )
        .expect("render FASTQ parser fixture coverage");

        assert_eq!(report.schema_version, FASTQ_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_FASTQ_PARSER_FIXTURE_COVERAGE_PATH);
        assert_eq!(report.stage_count, 27);
        assert_eq!(report.tool_count, 44);
        assert_eq!(report.row_count, 69);
        assert_eq!(report.covered_row_count, 69);
        assert_eq!(report.missing_row_count, 0);
        assert_eq!(report.parser_fixture_coverage_percent, 100.0);
        assert_eq!(report.coverage_status_counts.get("covered"), Some(&69));
        assert_eq!(report.rows.len(), 69);

        assert!(report.rows.iter().all(|row| {
            row.coverage_status == FastqParserFixtureCoverageStatus::Covered
                && row.parser_fixture_reference_kind == "fixture_case"
                && !row.parser_fixture_reference.is_empty()
                && !row.parser_fixture_parser_id.is_empty()
                && !row.parser_fixture_schema_id.is_empty()
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.trim_reads"
                && row.tool_id == "trimmomatic"
                && row.parser_fixture_parser_id == "parse_trim_reads_report"
                && row.parser_fixture_reference == "fastq.trim_reads.report_json"
                && row.parser_fixture_canonical_tool_id == "fastp"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.detect_duplicates_premerge"
                && row.tool_id == "bijux_dna"
                && row.parser_fixture_parser_id == "parse_detect_duplicates_premerge_report"
                && row.parser_fixture_reference == "fastq.detect_duplicates_premerge.report_json"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.screen_taxonomy"
                && row.tool_id == "kraken2"
                && row.parser_fixture_parser_id == "parse_screen_taxonomy_report"
                && row.parser_fixture_schema_id == "bijux.fastq.screen_taxonomy.report.v2"
        }));
    }

    #[test]
    fn coverage_status_labels_are_stable() {
        assert_eq!(coverage_status_label(FastqParserFixtureCoverageStatus::Covered), "covered");
        assert_eq!(
            coverage_status_label(FastqParserFixtureCoverageStatus::MissingFixtureBinding),
            "missing_fixture_binding"
        );
        assert_eq!(
            coverage_status_label(FastqParserFixtureCoverageStatus::MissingFixtureCase),
            "missing_fixture_case"
        );
        assert_eq!(
            coverage_status_label(FastqParserFixtureCoverageStatus::InvalidFixtureCase),
            "invalid_fixture_case"
        );
    }
}
