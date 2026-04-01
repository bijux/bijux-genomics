use crate::vcf::{vcf_minimal_profile, vcf_reference_basic_profile};
use crate::PipelineProfile;

#[must_use]
pub fn vcf_profiles() -> Vec<PipelineProfile> {
    vec![vcf_minimal_profile(), vcf_reference_basic_profile()]
}
