use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_corpus_fixture::edna::{
    load_edna_corpus_fixture_manifest_path, load_validated_edna_expected_taxa_rows,
    resolve_edna_expected_taxa_path, validate_edna_corpus_fixture_manifest_contract,
    EdnaExpectedPresence, EdnaExpectedTaxonRow,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const LOCAL_TAXONOMY_OUTPUT_JUDGMENT_SCHEMA_VERSION: &str =
    "bijux.bench.local_taxonomy_output_judgment.v1";
pub(crate) const DEFAULT_TAXONOMY_OUTPUT_JUDGMENT_PATH: &str =
    "benchmarks/readiness/local-ready/corpus-02-edna-taxonomy-judgment.json";

#[derive(Debug, Clone)]
pub(crate) struct LocalTaxonomyObservedReportArg {
    pub(crate) sample_id: String,
    pub(crate) report_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalTaxonomyObservedTaxon {
    pub(crate) name: String,
    pub(crate) percent: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalTaxonomyExpectationRowJudgment {
    pub(crate) sample_id: String,
    pub(crate) taxon_id: u64,
    pub(crate) name: String,
    pub(crate) rank: String,
    pub(crate) expected_presence: EdnaExpectedPresence,
    pub(crate) observed_presence: bool,
    pub(crate) matched: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalTaxonomySampleJudgment {
    pub(crate) sample_id: String,
    pub(crate) report_path: String,
    pub(crate) observed_taxa: Vec<LocalTaxonomyObservedTaxon>,
    pub(crate) expectation_count: usize,
    pub(crate) matched_expectation_count: usize,
    pub(crate) mismatched_expectation_count: usize,
    pub(crate) valid: bool,
    pub(crate) rows: Vec<LocalTaxonomyExpectationRowJudgment>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalTaxonomyOutputJudgmentReport {
    pub(crate) schema_version: &'static str,
    pub(crate) manifest_path: String,
    pub(crate) expected_taxa_path: String,
    pub(crate) sample_count: usize,
    pub(crate) expectation_count: usize,
    pub(crate) matched_expectation_count: usize,
    pub(crate) mismatched_expectation_count: usize,
    pub(crate) valid: bool,
    pub(crate) samples: Vec<LocalTaxonomySampleJudgment>,
}

#[derive(Debug, Clone)]
struct ObservedTaxonomyReport {
    taxa: Vec<LocalTaxonomyObservedTaxon>,
}

pub(crate) fn judge_edna_taxonomy_outputs(
    repo_root: &Path,
    manifest_path: &Path,
    reports: &[LocalTaxonomyObservedReportArg],
) -> Result<LocalTaxonomyOutputJudgmentReport> {
    let manifest = load_edna_corpus_fixture_manifest_path(manifest_path)?;
    validate_edna_corpus_fixture_manifest_contract(&manifest)?;
    let expected_taxa_path = resolve_edna_expected_taxa_path(manifest_path, &manifest)?;
    let expected_rows = load_validated_edna_expected_taxa_rows(&manifest, &expected_taxa_path)?;

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

    let mut matched_expectation_count = 0usize;
    let mut mismatched_expectation_count = 0usize;
    let mut samples = Vec::with_capacity(manifest.samples.len());

    for sample in &manifest.samples {
        let (report_path, observed) =
            observed_reports.remove(&sample.sample_id).ok_or_else(|| {
                anyhow!("missing observed report for sample_id `{}`", sample.sample_id)
            })?;
        let observed_presence = observed
            .taxa
            .iter()
            .filter(|entry| entry.percent > 0.0)
            .map(|entry| entry.name.as_str())
            .collect::<BTreeSet<_>>();
        let rows = expected_rows
            .iter()
            .filter(|row| row.sample_id == sample.sample_id)
            .map(|row| row_judgment(row, &observed_presence))
            .collect::<Vec<_>>();
        let sample_matched = rows.iter().filter(|row| row.matched).count();
        let sample_mismatched = rows.len().saturating_sub(sample_matched);
        matched_expectation_count += sample_matched;
        mismatched_expectation_count += sample_mismatched;
        samples.push(LocalTaxonomySampleJudgment {
            sample_id: sample.sample_id.clone(),
            report_path: path_relative_to_repo(repo_root, &report_path),
            observed_taxa: observed.taxa,
            expectation_count: rows.len(),
            matched_expectation_count: sample_matched,
            mismatched_expectation_count: sample_mismatched,
            valid: sample_mismatched == 0,
            rows,
        });
    }

    Ok(LocalTaxonomyOutputJudgmentReport {
        schema_version: LOCAL_TAXONOMY_OUTPUT_JUDGMENT_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_taxa_path: path_relative_to_repo(repo_root, &expected_taxa_path),
        sample_count: samples.len(),
        expectation_count: expected_rows.len(),
        matched_expectation_count,
        mismatched_expectation_count,
        valid: mismatched_expectation_count == 0,
        samples,
    })
}

pub(crate) fn run_judge_taxonomy_output(
    args: &parse::BenchLocalJudgeTaxonomyOutputArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let manifest_path = if args.manifest.is_absolute() {
        args.manifest.clone()
    } else {
        repo_root.join(&args.manifest)
    };
    let output_path =
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_TAXONOMY_OUTPUT_JUDGMENT_PATH));
    let report = render_edna_taxonomy_output_judgment(
        &repo_root,
        manifest_path,
        parse_report_args(&args.reports)?,
        output_path.clone(),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        let absolute_output_path =
            if output_path.is_absolute() { output_path } else { repo_root.join(output_path) };
        println!("{}", path_relative_to_repo(&repo_root, &absolute_output_path));
    }
    Ok(())
}

pub(crate) fn render_edna_taxonomy_output_judgment(
    repo_root: &Path,
    manifest_path: PathBuf,
    reports: Vec<LocalTaxonomyObservedReportArg>,
    output_path: PathBuf,
) -> Result<LocalTaxonomyOutputJudgmentReport> {
    let absolute_output_path =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let report = judge_edna_taxonomy_outputs(repo_root, &manifest_path, &reports)?;
    fs::write(
        &absolute_output_path,
        serde_json::to_string_pretty(&report).context("serialize taxonomy output judgment")?,
    )
    .with_context(|| format!("write {}", absolute_output_path.display()))?;
    Ok(report)
}

fn row_judgment(
    row: &EdnaExpectedTaxonRow,
    observed_presence: &BTreeSet<&str>,
) -> LocalTaxonomyExpectationRowJudgment {
    let observed = observed_presence.contains(row.name.as_str());
    let matched = match row.expected_presence {
        EdnaExpectedPresence::Present => observed,
        EdnaExpectedPresence::Absent => !observed,
    };
    LocalTaxonomyExpectationRowJudgment {
        sample_id: row.sample_id.clone(),
        taxon_id: row.taxon_id,
        name: row.name.clone(),
        rank: row.rank.clone(),
        expected_presence: row.expected_presence,
        observed_presence: observed,
        matched,
    }
}

fn load_observed_taxonomy_report(report_path: &Path) -> Result<ObservedTaxonomyReport> {
    let raw = fs::read_to_string(report_path)
        .with_context(|| format!("read {}", report_path.display()))?;
    let payload: serde_json::Value =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", report_path.display()))?;
    let entries = payload
        .get("summary_entries")
        .or_else(|| payload.get("top_taxa"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            anyhow!(
                "taxonomy output judgment report {} must declare a `summary_entries` or `top_taxa` array",
                report_path.display()
            )
        })?;

    let mut seen_names = BTreeSet::new();
    let mut taxa = entries
        .iter()
        .map(|entry| {
            let name = entry
                .get("label")
                .or_else(|| entry.get("name"))
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .ok_or_else(|| {
                    anyhow!(
                        "taxonomy output judgment report {} contains an entry without a string `label` or `name`",
                        report_path.display()
                    )
                })?;
            if name.is_empty() {
                return Err(anyhow!(
                    "taxonomy output judgment report {} contains an empty taxon label",
                    report_path.display()
                ));
            }
            let percent = entry
                .get("percent")
                .and_then(serde_json::Value::as_f64)
                .ok_or_else(|| {
                    anyhow!(
                        "taxonomy output judgment report {} taxon `{}` is missing numeric `percent`",
                        report_path.display(),
                        name
                    )
                })?;
            if !percent.is_finite() || percent.is_sign_negative() {
                return Err(anyhow!(
                    "taxonomy output judgment report {} taxon `{}` has invalid percent {}",
                    report_path.display(),
                    name,
                    percent
                ));
            }
            if !seen_names.insert(name.to_string()) {
                return Err(anyhow!(
                    "taxonomy output judgment report {} repeats taxon `{}`",
                    report_path.display(),
                    name
                ));
            }
            Ok(LocalTaxonomyObservedTaxon { name: name.to_string(), percent })
        })
        .collect::<Result<Vec<_>>>()?;
    taxa.retain(|entry| !entry.name.eq_ignore_ascii_case("unclassified"));
    taxa.sort_by(|left, right| {
        right.percent.total_cmp(&left.percent).then_with(|| left.name.cmp(&right.name))
    });

    Ok(ObservedTaxonomyReport { taxa })
}

fn parse_report_args(values: &[String]) -> Result<Vec<LocalTaxonomyObservedReportArg>> {
    if values.is_empty() {
        return Err(anyhow!(
            "taxonomy output judgment requires at least one `--report sample_id=path` mapping"
        ));
    }
    values
        .iter()
        .map(|value| {
            let (sample_id, report_path) = value.split_once('=').ok_or_else(|| {
                anyhow!(
                    "taxonomy output judgment report mapping must use `sample_id=path`, found `{value}`"
                )
            })?;
            if sample_id.trim().is_empty() || report_path.trim().is_empty() {
                return Err(anyhow!(
                    "taxonomy output judgment report mapping must use non-empty `sample_id=path`, found `{value}`"
                ));
            }
            Ok(LocalTaxonomyObservedReportArg {
                sample_id: sample_id.trim().to_string(),
                report_path: PathBuf::from(report_path.trim()),
            })
        })
        .collect()
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        judge_edna_taxonomy_outputs, LocalTaxonomyObservedReportArg,
        LOCAL_TAXONOMY_OUTPUT_JUDGMENT_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn taxonomy_output_judgment_matches_governed_expected_taxa_matrix() {
        let root = repo_root();
        let temp = tempfile::tempdir().expect("tempdir");
        let sample_a = temp.path().join("sample-a.json");
        let sample_b = temp.path().join("sample-b.json");
        std::fs::write(
            &sample_a,
            serde_json::to_string_pretty(&serde_json::json!({
                "summary_entries": [
                    {"label": "Escherichia coli", "percent": 60.0},
                    {"label": "Salmonella enterica", "percent": 40.0},
                    {"label": "unclassified", "percent": 0.0}
                ]
            }))
            .expect("encode sample a"),
        )
        .expect("write sample a");
        std::fs::write(
            &sample_b,
            serde_json::to_string_pretty(&serde_json::json!({
                "summary_entries": [
                    {"label": "Halobacterium salinarum", "percent": 100.0}
                ]
            }))
            .expect("encode sample b"),
        )
        .expect("write sample b");

        let report = judge_edna_taxonomy_outputs(
            &root,
            &root.join("benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/manifest.toml"),
            &[
                LocalTaxonomyObservedReportArg {
                    sample_id: "mock_community_sample_a".to_string(),
                    report_path: sample_a,
                },
                LocalTaxonomyObservedReportArg {
                    sample_id: "mock_community_sample_b".to_string(),
                    report_path: sample_b,
                },
            ],
        )
        .expect("judge observed taxonomy outputs");

        assert_eq!(report.schema_version, LOCAL_TAXONOMY_OUTPUT_JUDGMENT_SCHEMA_VERSION);
        assert_eq!(report.sample_count, 2);
        assert_eq!(report.expectation_count, 6);
        assert_eq!(report.matched_expectation_count, 6);
        assert_eq!(report.mismatched_expectation_count, 0);
        assert!(report.valid);
        assert!(report.samples.iter().all(|sample| sample.valid));
    }

    #[test]
    fn taxonomy_output_judgment_rejects_missing_sample_report() {
        let root = repo_root();
        let temp = tempfile::tempdir().expect("tempdir");
        let sample_a = temp.path().join("sample-a.json");
        std::fs::write(
            &sample_a,
            serde_json::to_string_pretty(&serde_json::json!({
                "summary_entries": [
                    {"label": "Escherichia coli", "percent": 100.0}
                ]
            }))
            .expect("encode sample a"),
        )
        .expect("write sample a");

        let error = judge_edna_taxonomy_outputs(
            &root,
            &root.join("benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/manifest.toml"),
            &[LocalTaxonomyObservedReportArg {
                sample_id: "mock_community_sample_a".to_string(),
                report_path: sample_a,
            }],
        )
        .expect_err("missing sample report should fail");

        assert!(
            error.to_string().contains("taxonomy output judgment is missing an observed report"),
            "validation error should explain the missing sample report: {error:#}"
        );
    }
}
