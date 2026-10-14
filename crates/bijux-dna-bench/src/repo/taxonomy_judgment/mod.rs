//! Acquisition of benchmark taxonomy inputs against explicit corpus authority.
use anyhow::{anyhow, Result};
use bijux_dna_bench_corpus::edna::{
    load_edna_corpus_fixture_manifest_path, load_validated_edna_expected_taxa_rows,
    resolve_edna_expected_taxa_path, validate_edna_corpus_fixture_manifest_contract,
    EdnaCorpusFixtureManifest, EdnaExpectedTaxonRow,
};
use bijux_dna_bench_model::{
    evaluate_benchmark_taxonomy_outputs, BenchmarkTaxonomyJudgmentInput,
    BenchmarkTaxonomyOutputJudgmentReport,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
mod observations;
use observations::load_observed_taxonomy_report;
mod publication;
pub use publication::publish_edna_taxonomy_output_judgment;

#[derive(Debug, Clone)]
pub struct TaxonomyObservedReportArg {
    pub sample_id: String,
    pub report_path: PathBuf,
}

/// Acquire and judge observed classifier summaries against the governed corpus matrix.
///
/// # Errors
/// Rejects unreadable or invalid corpus declarations, expectation rows, report mappings,
/// summary records and invalid inferred counts.
pub fn judge_edna_taxonomy_outputs(
    repo_root: &Path,
    manifest_path: &Path,
    reports: &[TaxonomyObservedReportArg],
) -> Result<BenchmarkTaxonomyOutputJudgmentReport> {
    let manifest = load_edna_corpus_fixture_manifest_path(manifest_path)?;
    validate_edna_corpus_fixture_manifest_contract(&manifest)?;
    let expected_taxa_path = resolve_edna_expected_taxa_path(manifest_path, &manifest)?;
    let expected_rows = load_validated_edna_expected_taxa_rows(&manifest, &expected_taxa_path)?;
    judge_edna_taxonomy_outputs_with_expected_rows(
        repo_root,
        manifest_path,
        &manifest,
        &expected_taxa_path,
        &expected_rows,
        reports,
    )
}

/// Acquire observed reports using the caller's already admitted corpus and matrix rows.
///
/// # Errors
/// Rejects empty, foreign, duplicate or missing report mappings, unreadable/invalid
/// summaries and invalid inferred counts.
pub fn judge_edna_taxonomy_outputs_with_expected_rows(
    repo_root: &Path,
    manifest_path: &Path,
    manifest: &EdnaCorpusFixtureManifest,
    expected_taxa_path: &Path,
    expected_rows: &[EdnaExpectedTaxonRow],
    reports: &[TaxonomyObservedReportArg],
) -> Result<BenchmarkTaxonomyOutputJudgmentReport> {
    if reports.is_empty() {
        return Err(anyhow!("taxonomy output judgment requires at least one observed report"));
    }

    let manifest_samples =
        manifest.samples.iter().map(|sample| sample.sample_id.as_str()).collect::<BTreeSet<_>>();
    let mut observed_reports = BTreeMap::new();
    for report in reports {
        if report.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "taxonomy output judgment report mappings must declare a non-empty sample_id"
            ));
        }
        if !manifest_samples.contains(report.sample_id.as_str()) {
            return Err(anyhow!(
                "taxonomy output judgment sample_id `{}` is not declared by the fixture manifest",
                report.sample_id
            ));
        }
        let absolute_report_path = if report.report_path.is_absolute() {
            report.report_path.clone()
        } else {
            repo_root.join(&report.report_path)
        };
        if !absolute_report_path.is_file() {
            return Err(anyhow!(
                "taxonomy output judgment report path is missing: {}",
                absolute_report_path.display()
            ));
        }
        let observed = load_observed_taxonomy_report(&absolute_report_path)?;
        if observed_reports
            .insert(report.sample_id.clone(), (absolute_report_path, observed))
            .is_some()
        {
            return Err(anyhow!(
                "taxonomy output judgment repeats sample_id `{}`",
                report.sample_id
            ));
        }
    }

    for sample_id in &manifest_samples {
        if !observed_reports.contains_key(*sample_id) {
            return Err(anyhow!(
                "taxonomy output judgment is missing an observed report for sample_id `{sample_id}`"
            ));
        }
    }

    let manifest_label = path_relative_to_repo(repo_root, manifest_path);
    let expected_taxa_label = path_relative_to_repo(repo_root, expected_taxa_path);
    let observed_reports = observed_reports
        .into_iter()
        .map(|(sample_id, (path, observation))| {
            (sample_id, (path_relative_to_repo(repo_root, &path), observation))
        })
        .collect();
    evaluate_benchmark_taxonomy_outputs(BenchmarkTaxonomyJudgmentInput {
        manifest_path: &manifest_label,
        expected_taxa_path: &expected_taxa_label,
        manifest,
        expected_rows,
        observed_reports,
    })
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
#[path = "../../../tests/contracts/taxonomy_judgment/repository.rs"]
mod tests;
