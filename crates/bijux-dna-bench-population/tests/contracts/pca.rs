//! Retained pca parser and service assertions.
use crate::pca::observations::records::parse_eigenvalues;
use crate::pca::observations::records::parse_pca_rows;
use crate::pca::workflow::run_local_vcf_pca_smoke;
use bijux_dna_bench_corpus::vcf::SampleMetadataRow;
use std::collections::BTreeMap;

#[test]
fn pca_rows_reject_duplicate_samples() {
    let temp = tempfile::tempdir().expect("tempdir");
    let table = temp.path().join("pca.tsv");
    bijux_dna_infra::write_payload(
        &table,
        "sample\tPC1\tPC2\nsample_a\t0.1\t0.2\nsample_a\t0.3\t0.4\n",
    )
    .expect("write pca table");
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "sample_a".to_string(),
        SampleMetadataRow {
            sample_id: "sample_a".to_string(),
            population_id: "cohort_alpha".to_string(),
            sex: "female".to_string(),
            role: "cohort".to_string(),
            description: "sample a".to_string(),
        },
    );
    let mut populations = BTreeMap::new();
    populations.insert("cohort_alpha".to_string(), "Cohort Alpha".to_string());
    let err = parse_pca_rows(&table, &metadata, &populations).expect_err("duplicate samples");
    assert!(err.to_string().contains("duplicated sample"));
}

#[test]
fn eigenvalues_parse_governed_tsv_rows() {
    let temp = tempfile::tempdir().expect("tempdir");
    let table = temp.path().join("eigenval.tsv");
    bijux_dna_infra::write_payload(&table, "component\teigenvalue\nPC1\t0.9\nPC2\t0.4\n")
        .expect("write eigenvalues");
    let eigenvalues = parse_eigenvalues(&table).expect("parse eigenvalues");
    assert_eq!(eigenvalues, vec![0.9, 0.4]);
}

#[test]
fn governed_vcf_pca_smoke_reports_complete_cohort_rows() {
    let fixture = crate::test_support::fixture();
    let repo_root = fixture.path().to_path_buf();
    let report = run_local_vcf_pca_smoke(&crate::test_support::inputs(), &repo_root, "plink2")
        .expect("run local pca smoke");
    assert_eq!(report.stage_id, "vcf.pca");
    assert_eq!(report.tool_id, "plink2");
    assert_eq!(report.corpus_id, "vcf_production_regression");
    assert_eq!(report.input_fixture_id, "vcf_mini_multisample_cohort");
    assert_eq!(report.variant_count, 2);
    assert_eq!(report.sample_count, 4);
    assert!(report.excluded_samples.is_empty());
    assert!(report.unexpected_samples.is_empty());
    assert_eq!(
        report.rows.iter().map(|row| row.sample_id.as_str()).collect::<Vec<_>>(),
        vec!["sample_a", "sample_b", "sample_c", "sample_d"]
    );
    assert!(report.eigenvalues.len() >= 2, "expected at least two governed PCA eigenvalues");
}

#[test]
fn governed_vcf_pca_smoke_supports_retained_eigensoft_binding() {
    let fixture = crate::test_support::fixture();
    let repo_root = fixture.path().to_path_buf();
    let report = run_local_vcf_pca_smoke(&crate::test_support::inputs(), &repo_root, "eigensoft")
        .expect("run eigensoft pca smoke");
    assert_eq!(report.stage_id, "vcf.pca");
    assert_eq!(report.tool_id, "eigensoft");
    assert_eq!(report.corpus_id, "vcf_production_regression");
    assert_eq!(report.input_fixture_id, "vcf_mini_multisample_cohort");
    assert_eq!(report.variant_count, 2);
    assert_eq!(report.sample_count, 4);
    assert!(report.excluded_samples.is_empty());
    assert!(report.unexpected_samples.is_empty());
    assert_eq!(
        report.rows.iter().map(|row| row.sample_id.as_str()).collect::<Vec<_>>(),
        vec!["sample_a", "sample_b", "sample_c", "sample_d"]
    );
    assert!(report.eigenvalues.len() >= 2, "expected at least two governed PCA eigenvalues");
}
