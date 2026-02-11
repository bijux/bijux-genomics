use bijux_dna_pipelines::fastq::{
    fastq_adna_profile, fastq_default_profile, fastq_minimal_profile, validate_fastq_profile,
};

#[test]
fn fastq_profiles_pass_invariant_gate() {
    for profile in [
        fastq_default_profile(),
        fastq_adna_profile(),
        fastq_minimal_profile(),
    ] {
        let report = validate_fastq_profile(&profile);
        assert!(
            report.valid,
            "profile {} failed FASTQ invariants: {:?}",
            report.profile_id, report.violations
        );
    }
}
