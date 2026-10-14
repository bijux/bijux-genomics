use std::fs;
use std::path::PathBuf;

use bijux_dna_bench_corpus::bam::{
    validate_bam_corpus_fixture_manifest_path, BAM_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
    DEFAULT_CORPUS_01_ADNA_BAM_MINI_MANIFEST_PATH, DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH,
    DEFAULT_CORPUS_01_GENOTYPING_MINI_MANIFEST_PATH, DEFAULT_CORPUS_01_KINSHIP_MINI_MANIFEST_PATH,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn corpus_01_bam_mini_fixture_manifest_validates_expected_contigs_and_sample_ids() {
    let root = repo_root();
    let report = validate_bam_corpus_fixture_manifest_path(
        &root,
        &root.join(DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH),
    )
    .expect("validate corpus-01 bam mini fixture manifest");

    assert_eq!(report.schema_version, BAM_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION);
    assert_eq!(report.corpus_id, "corpus-01-bam-mini");
    assert_eq!(report.sample_count, 23);
    assert_eq!(
        report.reference_contigs,
        vec![
            "chr1".to_string(),
            "chr2".to_string(),
            "chrX".to_string(),
            "chrY".to_string(),
            "chranc".to_string(),
            "chrgc".to_string(),
        ]
    );
    assert!(report.valid);
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_duplicate_flagged_multicontig"
            && sample.observed_contigs == vec!["chr1".to_string(), "chr2".to_string()]
            && sample.observed_header_sample_ids
                == vec!["human_like_duplicate_flagged_multicontig".to_string()]
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_gc_window_ladder"
            && sample.observed_contigs == vec!["chrgc".to_string()]
            && sample.observed_header_sample_ids
                == vec!["human_like_gc_window_ladder".to_string()]
            && sample.observed_read_group_ids
                == vec!["rg-gc-bias-human-like".to_string()]
            && sample.source_paths
                == vec![
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam"
                        .to_string(),
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta"
                        .to_string(),
                ]
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_partial_mapping"
            && sample.observed_contigs == vec!["chr1".to_string()]
            && sample.observed_header_sample_ids == vec!["human_like_partial_mapping".to_string()]
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_endogenous_partial_mapping"
            && sample.observed_contigs == vec!["chr1".to_string()]
            && sample.observed_header_sample_ids
                == vec!["human_like_endogenous_partial_mapping".to_string()]
            && sample.observed_read_group_ids
                == vec!["rg-endogenous-content-human-like".to_string()]
            && sample.observed_record_count == 5
            && sample.source_paths
                == vec![
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_endogenous_partial_mapping.sam"
                        .to_string(),
                ]
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_contamination_panel_screen"
            && sample.observed_contigs == vec!["chr1".to_string()]
            && sample.observed_header_sample_ids
                == vec!["human_like_contamination_panel_screen".to_string()]
            && sample.observed_read_group_ids
                == vec!["rg-contamination-human-like".to_string()]
            && sample.observed_record_count == 3
            && sample.source_paths
                == vec![
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_contamination_panel_screen.sam"
                        .to_string(),
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_contamination_panel.dat"
                        .to_string(),
                ]
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_paired_overlap_control"
            && sample.observed_contigs == vec!["chr1".to_string()]
            && sample.observed_header_sample_ids
                == vec!["human_like_paired_overlap_control".to_string()]
            && sample.observed_read_group_ids
                == vec!["rg-overlap-correction-human-like".to_string()]
            && sample.observed_record_count == 4
            && sample.source_paths
                == vec![
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_paired_overlap_control.sam"
                        .to_string(),
                ]
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_mapq_threshold_ladder"
            && sample.observed_contigs == vec!["chr1".to_string()]
            && sample.observed_header_sample_ids
                == vec!["human_like_mapq_threshold_ladder".to_string()]
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_length_threshold_ladder"
            && sample.observed_contigs == vec!["chr1".to_string()]
            && sample.observed_header_sample_ids
                == vec!["human_like_length_threshold_ladder".to_string()]
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_complexity_projection"
            && sample.observed_contigs == vec!["chr1".to_string()]
            && sample.observed_header_sample_ids
                == vec!["human_like_complexity_projection".to_string()]
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_target_window_coverage"
            && sample.observed_contigs == vec!["chr1".to_string(), "chr2".to_string()]
            && sample.observed_header_sample_ids
                == vec!["human_like_target_window_coverage".to_string()]
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_insert_size_triplet"
            && sample.observed_contigs == vec!["chr1".to_string()]
            && sample.observed_header_sample_ids
                == vec!["human_like_insert_size_triplet".to_string()]
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_xy_autosome_coverage"
            && sample.observed_contigs
                == vec!["chr1".to_string(), "chrX".to_string(), "chrY".to_string()]
            && sample.observed_header_sample_ids
                == vec!["human_like_xy_autosome_coverage".to_string()]
            && sample.observed_read_group_ids
                == vec!["rg-sex-human-like".to_string()]
            && sample.observed_record_count == 5
            && sample.source_paths
                == vec![
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_xy_autosome_coverage.sam"
                        .to_string(),
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
                        .to_string(),
                ]
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_mixed_filter_constraints"
            && sample.observed_contigs == vec!["chr1".to_string()]
            && sample.observed_header_sample_ids
                == vec!["human_like_mixed_filter_constraints".to_string()]
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_duplicate_cluster"
            && sample.observed_contigs == vec!["chr1".to_string()]
            && sample.observed_header_sample_ids == vec!["human_like_duplicate_cluster".to_string()]
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "adna_like_damage"
            && sample.observed_contigs == vec!["chranc".to_string()]
            && sample.observed_header_sample_ids == vec!["adna_like_damage".to_string()]
    }));
}

#[test]
fn corpus_01_genotyping_mini_fixture_manifest_validates_governed_candidate_site_sample() {
    let root = repo_root();
    let report = validate_bam_corpus_fixture_manifest_path(
        &root,
        &root.join(DEFAULT_CORPUS_01_GENOTYPING_MINI_MANIFEST_PATH),
    )
    .expect("validate corpus-01 genotyping mini fixture manifest");

    assert_eq!(report.schema_version, BAM_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION);
    assert_eq!(report.corpus_id, "corpus-01-genotyping-mini");
    assert_eq!(report.sample_count, 1);
    assert_eq!(
        report.reference_contigs,
        vec![
            "chr1".to_string(),
            "chr2".to_string(),
            "chrX".to_string(),
            "chrY".to_string(),
            "chranc".to_string(),
            "chrgc".to_string(),
        ]
    );
    assert!(report.valid);
    assert_eq!(
        report.samples.first().map(|sample| sample.sample_id.as_str()),
        Some("human_like_genotyping_candidate_panel")
    );
    assert_eq!(
        report.samples.first().map(|sample| sample.observed_read_group_ids.clone()),
        Some(vec!["rg-genotyping-human-like".to_string()])
    );
    let contract =
        report.genotyping_contract.as_ref().expect("genotyping contract must be present");
    assert_eq!(contract.sample_id, "human_like_genotyping_candidate_panel");
    assert_eq!(
        contract.sites_vcf,
        "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_genotyping_candidate_sites.vcf"
    );
    assert_eq!(
        contract.regions,
        "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_genotyping_target_regions.txt"
    );
    assert_eq!(contract.min_posterior, 0.9);
    assert_eq!(contract.min_call_rate, 0.5);
    assert_eq!(
        contract.expected_outputs,
        vec![
            "genotyping_bcf".to_string(),
            "genotyping_vcf".to_string(),
            "genotyping_vcf_tbi".to_string(),
            "genotyping_gl".to_string(),
            "summary".to_string(),
            "stage_metrics".to_string(),
        ]
    );
}

#[test]
fn corpus_01_adna_bam_mini_fixture_manifest_validates_governed_adna_support_samples() {
    let root = repo_root();
    let report = validate_bam_corpus_fixture_manifest_path(
        &root,
        &root.join(DEFAULT_CORPUS_01_ADNA_BAM_MINI_MANIFEST_PATH),
    )
    .expect("validate corpus-01 adna bam mini fixture manifest");

    assert_eq!(report.schema_version, BAM_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION);
    assert_eq!(report.corpus_id, "corpus-01-adna-bam-mini");
    assert_eq!(report.sample_count, 4);
    assert_eq!(report.udg_model.as_deref(), Some("non_udg"));
    assert_eq!(report.damage_signal.as_deref(), Some("moderate"));
    assert_eq!(report.expected_terminal_pattern_class.as_deref(), Some("ct5p_dominant"));
    assert!(report.valid);
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "adna_contamination_panel_screen"
            && sample.observed_contigs == vec!["chr1".to_string()]
            && sample.observed_header_sample_ids
                == vec!["adna_contamination_panel_screen".to_string()]
            && sample.observed_read_group_ids == vec!["rg-contamination-adna".to_string()]
            && sample.observed_record_count == 3
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "adna_xy_autosome_coverage"
            && sample.observed_contigs
                == vec!["chr1".to_string(), "chrX".to_string(), "chrY".to_string()]
            && sample.observed_header_sample_ids == vec!["adna_xy_autosome_coverage".to_string()]
            && sample.observed_read_group_ids == vec!["rg-sex-adna".to_string()]
            && sample.observed_record_count == 5
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "adna_y_haplogroup_panel"
            && sample.observed_contigs == vec!["chrY".to_string()]
            && sample.observed_header_sample_ids == vec!["adna_y_haplogroup_panel".to_string()]
            && sample.observed_read_group_ids == vec!["rg-haplogroups-adna".to_string()]
            && sample.observed_record_count == 4
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "adna_y_haplogroup_partial_panel"
            && sample.observed_contigs == vec!["chrY".to_string()]
            && sample.observed_header_sample_ids
                == vec!["adna_y_haplogroup_partial_panel".to_string()]
            && sample.observed_read_group_ids == vec!["rg-haplogroups-adna-partial".to_string()]
            && sample.observed_record_count == 2
    }));
}

#[test]
fn corpus_01_kinship_mini_fixture_manifest_validates_governed_pair_samples() {
    let root = repo_root();
    let report = validate_bam_corpus_fixture_manifest_path(
        &root,
        &root.join(DEFAULT_CORPUS_01_KINSHIP_MINI_MANIFEST_PATH),
    )
    .expect("validate corpus-01 kinship mini fixture manifest");

    assert_eq!(report.schema_version, BAM_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION);
    assert_eq!(report.corpus_id, "corpus-01-kinship-mini");
    assert_eq!(report.sample_count, 2);
    assert!(report.valid);
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_kinship_low_overlap_pair"
            && sample.observed_header_sample_ids
                == vec!["sample_a".to_string(), "sample_b".to_string()]
            && sample.observed_read_group_ids
                == vec![
                    "rg-kinship-low-overlap-a".to_string(),
                    "rg-kinship-low-overlap-b".to_string(),
                ]
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_kinship_related_pair"
            && sample.observed_header_sample_ids
                == vec!["sample_a".to_string(), "sample_b".to_string()]
            && sample.observed_read_group_ids
                == vec!["rg-kinship-related-a".to_string(), "rg-kinship-related-b".to_string()]
    }));
    let contract = report.kinship_contract.as_ref().expect("kinship contract must be present");
    assert_eq!(contract.reference_panel, "human_like_relatedness_panel");
    assert_eq!(
        contract.reference_panel_path,
        "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_relatedness_panel.tsv"
    );
    assert_eq!(contract.reference_build, "grch38");
    assert_eq!(contract.population_scope, "human_diploid_panel");
    assert_eq!(
        contract.expected_outputs,
        vec![
            "kinship_report".to_string(),
            "summary".to_string(),
            "kinship_segments".to_string(),
            "stage_metrics".to_string(),
        ]
    );
    assert!(contract.cases.iter().any(|case| {
        case.sample_id == "human_like_kinship_low_overlap_pair"
            && case.min_overlap_snps == 5
            && case.expected_status == "insufficient"
            && case.expected_observed_max_overlap_snps == 4
            && case.expected_relationship_labels.is_empty()
            && case.skip_behavior == "stop_without_pairwise_results"
    }));
    assert!(contract.cases.iter().any(|case| {
        case.sample_id == "human_like_kinship_related_pair"
            && case.min_overlap_snps == 6
            && case.expected_status == "ok"
            && case.expected_observed_max_overlap_snps == 6
            && case.expected_relationship_labels == vec!["first_degree".to_string()]
            && case.skip_behavior == "emit_pairwise_results"
    }));
}

#[test]
fn corpus_01_bam_mini_fixture_validation_refuses_header_sample_id_drift() {
    let root = repo_root();
    let fixture_root = root.join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini");
    let temp = tempfile::Builder::new()
        .prefix("broken-corpus-01-bam-mini-")
        .suffix(".toml")
        .tempfile_in(&fixture_root)
        .expect("temp manifest");
    let manifest_path = temp.path().to_path_buf();
    let broken = fs::read_to_string(root.join(DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH))
        .expect("read governed corpus-01 bam mini manifest")
        .replacen(
            "expected_header_sample_ids = [\"core-v1-pass\"]",
            "expected_header_sample_ids = [\"unexpected_sample\"]",
            1,
        );
    fs::write(&manifest_path, broken).expect("write broken manifest");

    let error = validate_bam_corpus_fixture_manifest_path(&root, &manifest_path)
        .expect_err("manifest validation should reject header sample drift");
    assert!(
        error.to_string().contains("expected header sample IDs"),
        "validation error should explain sample-id drift: {error:#}"
    );
}

#[path = "bam/alignment_truth.rs"]
mod alignment_truth;
