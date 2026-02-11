//! VCF stage specs and metrics parser bindings.

pub mod metrics;
pub mod stage_specs;

#[must_use]
pub fn implemented_stages() -> Vec<bijux_dna_domain_vcf::VcfStage> {
    bijux_dna_domain_vcf::VcfStage::all().to_vec()
}
