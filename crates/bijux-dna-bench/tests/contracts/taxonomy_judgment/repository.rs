//! Existing repository taxonomy judgment oracles, registered once.
#![allow(clippy::expect_used)]

use std::path::PathBuf;

use super::{
    judge_edna_taxonomy_outputs, TaxonomyObservedReportArg as LocalTaxonomyObservedReportArg,
};
use bijux_dna_bench_model::BENCHMARK_TAXONOMY_OUTPUT_JUDGMENT_SCHEMA_VERSION as LOCAL_TAXONOMY_OUTPUT_JUDGMENT_SCHEMA_VERSION;

fn repo_root() -> PathBuf {
    crate::repo::resolve_repo_root().expect("repo root")
}

#[test]
fn taxonomy_output_judgment_matches_governed_expected_taxa_matrix() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let sample_a = temp.path().join("sample-a.json");
    let sample_b = temp.path().join("sample-b.json");
    bijux_dna_infra::write_payload(
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
    bijux_dna_infra::write_payload(
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
    assert!(report.samples.iter().all(|sample| {
        sample.valid
            && sample.false_positive_count == 0
            && sample.observed_unclassified_percent == 0.0
            && sample.observed_unclassified_read_count == 0
    }));
}

#[test]
fn taxonomy_output_judgment_rejects_missing_sample_report() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let sample_a = temp.path().join("sample-a.json");
    bijux_dna_infra::write_payload(
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

#[test]
fn taxonomy_output_judgment_reports_false_positive_taxa() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let sample_a = temp.path().join("sample-a.json");
    let sample_b = temp.path().join("sample-b.json");
    bijux_dna_infra::write_payload(
        &sample_a,
        serde_json::to_string_pretty(&serde_json::json!({
            "summary_entries": [
                {"label": "Escherichia coli", "percent": 50.0},
                {"label": "Salmonella enterica", "percent": 40.0},
                {"label": "Halobacterium salinarum", "percent": 10.0}
            ]
        }))
        .expect("encode sample a"),
    )
    .expect("write sample a");
    bijux_dna_infra::write_payload(
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

    let sample_a = report
        .samples
        .iter()
        .find(|sample| sample.sample_id == "mock_community_sample_a")
        .expect("sample a judgment");
    assert_eq!(sample_a.false_positive_count, 1);
    assert!(!sample_a.valid);
    assert_eq!(sample_a.unexpected_taxa.len(), 1);
    assert_eq!(sample_a.unexpected_taxa[0].name, "Halobacterium salinarum");
}
