use bijux_dna_pipelines::vcf::{validate_vcf_profile, vcf_minimal_profile};
use bijux_dna_pipelines::StabilityTier;

#[test]
fn vcf_minimal_profile_satisfies_invariants() {
    let profile = vcf_minimal_profile();
    let report = validate_vcf_profile(&profile);
    assert!(
        report.valid,
        "vcf profile violations: {:?}",
        report.violations
    );
}

#[test]
fn vcf_invariants_reject_missing_required_artifacts() {
    let mut profile = vcf_minimal_profile();
    profile
        .capabilities
        .required_artifacts
        .retain(|artifact| *artifact != "invariants_report.json");
    let report = validate_vcf_profile(&profile);
    assert!(!report.valid);
    assert!(report
        .violations
        .iter()
        .any(|v| v.code == "required_artifact_missing"));
}

#[test]
fn vcf_invariants_require_sample_and_reference_for_production() {
    let mut profile = vcf_minimal_profile();
    profile.stability = StabilityTier::Stable;
    let report = validate_vcf_profile(&profile);
    assert!(!report.valid);
    assert!(report
        .violations
        .iter()
        .any(|v| v.code == "reference_required"));
}
