#![cfg(test)]

use super::*;
use std::path::PathBuf;

#[test]
fn bam_kinship_summary_round_trips() {
    let summary = BamKinshipSummaryV1 {
        schema_version: BAM_KINSHIP_SUMMARY_SCHEMA_VERSION.to_string(),
        stage_id: "bam.kinship".to_string(),
        method: "king".to_string(),
        input_bam: PathBuf::from("input.sam"),
        reference_panel: "human_like_relatedness_panel".to_string(),
        reference_build: "grch38".to_string(),
        population_scope: "human_diploid_panel".to_string(),
        min_overlap_snps: 6,
        requires_cohort_context: true,
        sample_count: 2,
        observed_max_overlap_snps: 6,
        pair_count: 1,
        status: "ok".to_string(),
        insufficiency_reason: None,
        pairwise_results: vec![BamKinshipPairResultV1 {
            sample_a: "sample_a".to_string(),
            sample_b: "sample_b".to_string(),
            overlap_snps: 6,
            matching_sites: 5,
            mismatch_sites: 1,
            concordance: 0.833_333,
            kinship_coefficient: 0.416_667,
            relationship_label: "first_degree".to_string(),
        }],
    };
    let encoded = serde_json::to_string(&summary).expect("serialize kinship summary");
    let decoded: BamKinshipSummaryV1 =
        serde_json::from_str(&encoded).expect("deserialize kinship summary");
    assert_eq!(decoded, summary);
}

#[test]
fn acquired_kinship_projection_uses_missing_path_only_as_report_metadata() {
    let root = bijux_dna_testkit::tempdir_for("kinship-acquired-observation");
    let missing = root.path().join("not-acquired.sam");
    assert!(!missing.exists());
    let document = TinySamDocument {
        read_group_sample_map: HashMap::from([
            ("a".to_string(), "sample_a".to_string()),
            ("b".to_string(), "sample_b".to_string()),
        ]),
        records: ["a", "b"]
            .into_iter()
            .map(|read_group| TinySamRecord {
                qname: read_group.to_string(),
                flag: 0,
                rname: "chr1".to_string(),
                pos: 1,
                mapq: 60,
                cigar: "1M".to_string(),
                template_length: 0,
                seq: "A".to_string(),
                read_group_id: Some(read_group.to_string()),
            })
            .collect(),
        ..TinySamDocument::default()
    };
    let report = project_kinship(
        &document,
        KinshipContext {
            input_bam: &missing,
            method: "king",
            reference_panel: "declared-panel",
            reference_build: "declared-build",
            population_scope: "declared-scope",
            min_overlap_snps: 1,
            requires_cohort_context: true,
        },
    );
    assert_eq!(report.input_bam, missing);
    assert_eq!(report.pair_count, 1);
    assert_eq!(report.observed_max_overlap_snps, 1);
    assert_eq!(report.status, "ok");
    assert_eq!(report.pairwise_results[0].sample_a, "sample_a");
    assert_eq!(report.pairwise_results[0].sample_b, "sample_b");
    assert!(!report.input_bam.exists());
}
