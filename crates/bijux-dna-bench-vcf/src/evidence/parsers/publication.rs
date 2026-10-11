use super::{coverage_status_label, VCF_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION};
use anyhow::{anyhow, Result};
use bijux_dna_bench_vcf_evidence::model::parsers::{
    VcfParserFixtureCoverageReport, VcfParserFixtureCoverageRow, VcfParserFixtureCoverageStatus,
};
use std::{
    collections::BTreeMap,
    fmt::Write,
    path::{Path, PathBuf},
};
/// Publish fixture coverage diagnostics, returning an error when coverage is incomplete.
///
/// # Errors
/// Returns admission/acquisition/publication failures or incomplete fixture coverage.
pub fn render_vcf_parser_fixture_coverage(
    inputs: &crate::inputs::VcfAdapterInputs,
    repo_root: &Path,
    retained: &[bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow],
    output_path: &Path,
) -> Result<VcfParserFixtureCoverageReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let (stage_count, tool_count, rows) =
        super::collect_vcf_parser_fixture_coverage_rows_from_retained(inputs, repo_root, retained)?;

    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_vcf_parser_fixture_coverage_tsv(&rows).as_bytes(),
    )?;

    let covered_row_count = rows
        .iter()
        .filter(|row| row.coverage_status == VcfParserFixtureCoverageStatus::Covered)
        .count();
    let missing_row_count = rows.len().saturating_sub(covered_row_count);
    let parser_fixture_coverage_percent = if rows.is_empty() {
        0.0
    } else {
        f64::from(u32::try_from(covered_row_count)?) * 100.0 / f64::from(u32::try_from(rows.len())?)
    };
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
fn render_vcf_parser_fixture_coverage_tsv(rows: &[VcfParserFixtureCoverageRow]) -> String {
    let mut rendered = String::from(
        "stage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tparser_fixture_parser_id\tparser_fixture_schema_id\tparser_fixture_root_path\texpected_normalized_path\traw_fixture_count\traw_fixture_paths\tcoverage_status\treason\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
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
        );
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
