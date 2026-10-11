//! Original scientific and binding behavior at its owning capability.
use crate::imputation::fixtures::inputs::write_governed_impute_input_vcf;
use crate::imputation::fixtures::inputs::write_governed_impute_truth_vcf;
use crate::imputation::observations::variants::summarize_imputed_output;
use bijux_dna_domain_vcf::artifacts::genotypes_equivalent_unphased;
use bijux_dna_stages_vcf::pipeline::ImputeBackend;

#[test]
fn governed_impute_fixture_tracks_masked_input_and_truth() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input_path = dir.path().join("input.vcf");
    let truth_path = dir.path().join("truth.vcf");
    write_governed_impute_input_vcf(&input_path, ImputeBackend::Beagle).expect("write input");
    write_governed_impute_truth_vcf(&truth_path).expect("write truth");

    let input_summary = summarize_imputed_output(&input_path).expect("summarize input");
    let truth_summary = summarize_imputed_output(&truth_path).expect("summarize truth");
    assert_eq!(input_summary.variant_count, 2);
    assert_eq!(input_summary.masked_sample_gt, "./.");
    assert_eq!(truth_summary.masked_sample_gt, "0/1");
    assert_eq!(truth_summary.donor_sample_gt, "0/1");
}

#[test]
fn governed_glimpse_fixture_carries_likelihood_evidence() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input_path = dir.path().join("glimpse.vcf");
    write_governed_impute_input_vcf(&input_path, ImputeBackend::Glimpse)
        .expect("write GLIMPSE input");
    let payload = std::fs::read_to_string(input_path).expect("read GLIMPSE input");
    assert!(payload.contains("##FORMAT=<ID=GL,"));
    assert!(payload.contains("\tGT:GL\t"));
}

#[test]
fn unphased_truth_accepts_equivalent_phased_backend_genotypes() {
    assert!(genotypes_equivalent_unphased("0/1", "0|1"));
    assert!(genotypes_equivalent_unphased("0/1", "1|0"));
    assert!(!genotypes_equivalent_unphased("0/1", "0|0"));
    assert!(!genotypes_equivalent_unphased("0/1", ".|."));
}
