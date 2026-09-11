#![cfg(test)]
//! Original scientific and binding behavior at its owning capability.
use crate::phasing::fixtures::inputs::write_governed_phasing_input_vcf;
use crate::phasing::observations::variants::summarize_phasing_genotypes;
#[test]
fn governed_phasing_fixture_tracks_unphased_input_counts() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input_path = dir.path().join("input.vcf");
    write_governed_phasing_input_vcf(&input_path).expect("write fixture");
    let summary = summarize_phasing_genotypes(&input_path).expect("summarize input");
    assert_eq!(summary.genotype_count, 8);
    assert_eq!(summary.phased_genotypes, 0);
    assert_eq!(summary.unphased_genotypes, 8);
    assert_eq!(summary.phase_set_count, 0);
}
