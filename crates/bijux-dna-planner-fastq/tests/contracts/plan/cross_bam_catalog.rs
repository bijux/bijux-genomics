#[test]
fn cross_fastq_to_bam_catalog_includes_mapping_summary_before_coverage() {
    for profile_id in ["fastq-to-bam__default__v1", "fastq-to-bam__adna_shotgun__v1"] {
        let stages = bijux_dna_planner_fastq::cross_fastq_to_bam_id_catalog(profile_id);
        let mapping_idx = stages
            .iter()
            .position(|stage| stage == "bam.mapping_summary")
            .unwrap_or_else(|| panic!("{profile_id} missing bam.mapping_summary"));
        let coverage_idx = stages
            .iter()
            .position(|stage| stage == "bam.coverage")
            .unwrap_or_else(|| panic!("{profile_id} missing bam.coverage"));
        assert!(
            mapping_idx < coverage_idx,
            "{profile_id} must route bam.mapping_summary before bam.coverage"
        );
    }
}
