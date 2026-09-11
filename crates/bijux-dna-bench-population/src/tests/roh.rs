#![cfg(test)]
//! Retained roh parser and service assertions.
use crate::roh::admission::binding::resolve_governed_vcf_roh_smoke_contract;
use crate::roh::observations::records::parse_roh_per_sample_summary;
use crate::roh::observations::records::parse_roh_segments;
use crate::roh::workflow::run_local_vcf_roh_smoke;

#[test]
fn governed_vcf_roh_smoke_contract_matches_stage_matrix() {
    let contract =
        resolve_governed_vcf_roh_smoke_contract(&crate::test_support::inputs(), "plink2")
            .expect("resolve contract");
    assert_eq!(contract.stage, "vcf.roh");
    assert_eq!(contract.tool, "plink2");
    assert_eq!(contract.corpus, "vcf_production_regression");
    assert_eq!(contract.input_fixture, "vcf_mini_multisample_cohort");
}

#[test]
fn roh_segments_parser_rejects_unexpected_samples() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("segments.tsv");
    bijux_dna_infra::write_payload(
        &path,
        "sample\tcontig\tstart\tend\tlength_bp\tn_sites\npanel_ref_1\tchr1\t10\t20\t10\t2\n",
    )
    .expect("write segments");
    let err = parse_roh_segments(&path, &["sample_a".to_string()].into_iter().collect())
        .expect_err("unexpected sample");
    assert!(err.to_string().contains("unexpected sample"));
}

#[test]
fn roh_per_sample_parser_rejects_duplicate_samples() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("per-sample.tsv");
    bijux_dna_infra::write_payload(
        &path,
        "sample\tsegment_count\ttotal_length_bp\tmean_length_bp\nsample_a\t1\t20\t20.0\nsample_a\t0\t0\t0.0\n",
    )
    .expect("write per-sample");
    let err = parse_roh_per_sample_summary(&path, &["sample_a".to_string()])
        .expect_err("duplicate sample");
    assert!(err.to_string().contains("duplicated sample"));
}

#[test]
fn governed_vcf_roh_smoke_reports_segments_and_per_sample_summary() {
    let fixture = crate::test_support::fixture();
    let repo_root = fixture.path().to_path_buf();
    let report = run_local_vcf_roh_smoke(&crate::test_support::inputs(), &repo_root, "plink2")
        .expect("run local roh smoke");
    assert_eq!(report.stage_id, "vcf.roh");
    assert_eq!(report.tool_id, "plink2");
    assert_eq!(report.corpus_id, "vcf_production_regression");
    assert_eq!(report.input_fixture_id, "vcf_mini_multisample_cohort");
    assert_eq!(report.status, "complete");
    assert_eq!(report.sample_count, 4);
    assert_eq!(report.per_sample_summary.len(), 4);
    assert_eq!(
        report.per_sample_summary.iter().map(|row| row.sample_id.as_str()).collect::<Vec<_>>(),
        vec!["sample_a", "sample_b", "sample_c", "sample_d"]
    );
    assert_eq!(
        report.per_sample_summary.iter().map(|row| row.segment_count).sum::<u64>(),
        report.segment_count
    );
    assert_eq!(
        report.per_sample_summary.iter().map(|row| row.total_length).sum::<u64>(),
        report.total_length
    );
    assert!(report
        .segments
        .iter()
        .all(|segment| !segment.sample_id.is_empty() && segment.variant_count > 0));
}
