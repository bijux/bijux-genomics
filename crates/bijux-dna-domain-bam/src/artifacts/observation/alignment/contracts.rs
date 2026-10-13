#![cfg(test)]

use super::*;

#[test]
fn inspect_tiny_alignment_accepts_governed_binary_bam_fixture() {
    let repo_root = bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"));
    let bam = repo_root.join("assets/toy/core-v1/bam/validation_pass.bam");

    let inspection = inspect_tiny_alignment(&bam).expect("inspect BAM fixture");
    assert_eq!(inspection.sort_order.as_deref(), Some("coordinate"));
    assert_eq!(inspection.header_contigs, vec!["chr1".to_string()]);
    assert_eq!(inspection.header_sample_ids, vec!["core-v1-pass".to_string()]);
    assert_eq!(inspection.read_group_ids.len(), 1);
    assert_eq!(inspection.mapped_record_contigs, vec!["chr1".to_string()]);
    assert_eq!(inspection.record_count, 2);
}

#[test]
fn summarize_tiny_bam_alignment_truth_accepts_governed_binary_bam_fixture() {
    let repo_root = bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"));
    let bam = repo_root.join("assets/toy/core-v1/bam/validation_pass.bam");

    let summary = summarize_tiny_bam_alignment_truth(&bam).expect("summarize BAM fixture");
    assert_eq!(summary.schema_version, BAM_ALIGNMENT_TRUTH_SUMMARY_SCHEMA_VERSION);
    assert_eq!(summary.total_reads, 2);
    assert_eq!(summary.mapped_reads, 2);
    assert_eq!(summary.unmapped_reads, 0);
    assert_eq!(summary.mapped_contigs, vec!["chr1".to_string()]);
    assert_eq!(
        summary.mapped_positions,
        vec![
            BamAlignmentTruthPositionV1 {
                reference_name: "chr1".to_string(),
                position: 1,
                read_count: 1,
            },
            BamAlignmentTruthPositionV1 {
                reference_name: "chr1".to_string(),
                position: 7,
                read_count: 1,
            },
        ]
    );
    assert_eq!(
        summary.mapped_mapq_classes,
        vec![BamAlignmentTruthMapqClassV1 { mapq: 60, read_count: 2 }]
    );
    assert_eq!(
        summary.cigar_classes,
        vec![BamAlignmentTruthCigarClassV1 { cigar: "6M".to_string(), read_count: 2 }]
    );
}

#[test]
fn summarize_tiny_bam_alignment_truth_tracks_mapped_unmapped_positions_and_classes() {
    let repo_root = bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"));
    let sam = repo_root.join("assets/toy/core-v1/bam/filter_mixed_constraints.sam");

    let summary = summarize_tiny_bam_alignment_truth(&sam).expect("summarize SAM fixture");
    assert_eq!(summary.schema_version, BAM_ALIGNMENT_TRUTH_SUMMARY_SCHEMA_VERSION);
    assert_eq!(summary.total_reads, 5);
    assert_eq!(summary.mapped_reads, 4);
    assert_eq!(summary.unmapped_reads, 1);
    assert_eq!(summary.mapped_contigs, vec!["chr1".to_string()]);
    assert_eq!(
        summary.mapped_positions,
        vec![
            BamAlignmentTruthPositionV1 {
                reference_name: "chr1".to_string(),
                position: 1,
                read_count: 1,
            },
            BamAlignmentTruthPositionV1 {
                reference_name: "chr1".to_string(),
                position: 10,
                read_count: 1,
            },
            BamAlignmentTruthPositionV1 {
                reference_name: "chr1".to_string(),
                position: 20,
                read_count: 1,
            },
            BamAlignmentTruthPositionV1 {
                reference_name: "chr1".to_string(),
                position: 30,
                read_count: 1,
            },
        ]
    );
    assert_eq!(
        summary.mapped_mapq_classes,
        vec![
            BamAlignmentTruthMapqClassV1 { mapq: 10, read_count: 1 },
            BamAlignmentTruthMapqClassV1 { mapq: 60, read_count: 3 },
        ]
    );
    assert_eq!(
        summary.cigar_classes,
        vec![
            BamAlignmentTruthCigarClassV1 { cigar: "*".to_string(), read_count: 1 },
            BamAlignmentTruthCigarClassV1 { cigar: "6M".to_string(), read_count: 1 },
            BamAlignmentTruthCigarClassV1 { cigar: "8M".to_string(), read_count: 3 },
        ]
    );
}

#[test]
fn inspect_tiny_alignment_accepts_text_sam_fixture() {
    let scratch = bijux_dna_testkit::tempdir_for("inspect-tiny-alignment-sam");
    let temp = scratch.path();
    let sam = temp.join("fixture.sam");
    std::fs::write(
        &sam,
        "@HD\tVN:1.6\tSO:coordinate\n\
@SQ\tSN:chranc\tLN:64\n\
@RG\tID:rg-adna\tSM:adna_like_damage\n\
read1\t99\tchranc\t5\t42\t8M\t=\t13\t8\tCCTTCCAA\tFFFFFFFF\tRG:Z:rg-adna\n\
read1\t147\tchranc\t13\t42\t8M\t=\t5\t-8\tTTCCAAGG\tFFFFFFFF\tRG:Z:rg-adna\n",
    )
    .expect("write SAM fixture");

    let inspection = inspect_tiny_alignment(&sam).expect("inspect SAM fixture");
    assert_eq!(inspection.sort_order.as_deref(), Some("coordinate"));
    assert_eq!(inspection.header_contigs, vec!["chranc".to_string()]);
    assert_eq!(inspection.header_sample_ids, vec!["adna_like_damage".to_string()]);
    assert_eq!(inspection.read_group_ids, vec!["rg-adna".to_string()]);
    assert_eq!(inspection.mapped_record_contigs, vec!["chranc".to_string()]);
    assert_eq!(inspection.record_count, 2);
}
