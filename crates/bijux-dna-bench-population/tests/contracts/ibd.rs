//! Retained ibd parser and service assertions.
use crate::ibd::admission::binding::resolve_governed_vcf_ibd_smoke_contract;
use crate::ibd::fixtures::failure_probe::write_probe_sparse_overlap_vcf;
use crate::ibd::observations::records::compute_pair_overlap_marker_counts;
use crate::ibd::workflow::run_local_vcf_ibd_smoke;

#[test]
fn governed_vcf_ibd_smoke_contract_matches_stage_matrix() {
    let fixture = crate::test_support::fixture();
    let repo_root = fixture.path().to_path_buf();
    let contract = resolve_governed_vcf_ibd_smoke_contract(
        &crate::test_support::inputs(),
        &repo_root,
        "germline",
    )
    .expect("resolve contract");
    assert_eq!(contract.stage, "vcf.ibd");
    assert_eq!(contract.tool, "germline");
    assert_eq!(contract.corpus, "vcf_production_regression");
    assert_eq!(contract.input_fixture, "vcf_mini_multisample_cohort");

    let ibdseq = resolve_governed_vcf_ibd_smoke_contract(
        &crate::test_support::inputs(),
        &repo_root,
        "ibdseq",
    )
    .expect("resolve IBDseq contract");
    assert_eq!(ibdseq.stage, "vcf.ibd");
    assert_eq!(ibdseq.tool, "ibdseq");
    assert_eq!(ibdseq.corpus, "vcf_production_regression");
}

#[test]
fn pair_overlap_counts_track_called_markers_only() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("probe.vcf");
    write_probe_sparse_overlap_vcf(&path, "sample_a", "sample_b").expect("write probe");
    let counts = compute_pair_overlap_marker_counts(
        &path,
        &["sample_a".to_string(), "sample_b".to_string()],
    )
    .expect("pair overlap counts");
    assert_eq!(counts.get(&("sample_a".to_string(), "sample_b".to_string())).copied(), Some(1));
}

#[test]
fn governed_vcf_ibd_smoke_reports_pair_rows_and_localized_probe() {
    let fixture = crate::test_support::fixture();
    let repo_root = fixture.path().to_path_buf();
    let report = run_local_vcf_ibd_smoke(&crate::test_support::inputs(), &repo_root, "germline")
        .expect("run local ibd smoke");
    assert_eq!(report.stage_id, "vcf.ibd");
    assert_eq!(report.tool_id, "germline");
    assert_eq!(report.corpus_id, "vcf_production_regression");
    assert_eq!(report.input_fixture_id, "vcf_mini_multisample_cohort");
    assert_eq!(report.status, "complete");
    assert!(!report.rows.is_empty(), "expected at least one IBD pair row");
    assert!(report.rows.iter().all(|row| row.segment_count > 0
        && row.overlap_marker_count > 0
        && row.status == "complete"));
    assert_eq!(report.insufficient_overlap_probe.ibd_status, "insufficient_marker_overlap");
    assert_eq!(report.insufficient_overlap_probe.unrelated_stage_id, "vcf.roh");
    assert_eq!(report.insufficient_overlap_probe.unrelated_stage_status, "complete");
    assert!(
        report.insufficient_overlap_probe.unrelated_stage_segment_count > 0,
        "expected sparse-overlap probe to keep ROH productive"
    );
}
