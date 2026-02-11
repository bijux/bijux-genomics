use bijux_dna_pipelines::vcf::{validate_vcf_profile, vcf_minimal_profile};

#[test]
fn vcf_minimal_profile_satisfies_invariants() {
    let profile = vcf_minimal_profile();
    let report = validate_vcf_profile(&profile);
    assert!(report.valid, "vcf profile violations: {:?}", report.violations);
}
