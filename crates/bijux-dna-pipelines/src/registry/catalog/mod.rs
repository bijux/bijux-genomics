//! Pipeline registry catalog assembly.

mod queries;

use super::{bam_profiles, cross_profiles, fastq_profiles, vcf_profiles};
use crate::PipelineProfile;

#[derive(Debug, Clone)]
pub struct PipelineRegistry {
    profiles: Vec<PipelineProfile>,
}

impl PipelineRegistry {
    #[must_use]
    pub fn v1() -> Self {
        let mut profiles = Vec::new();
        profiles.extend(fastq_profiles());
        profiles.extend(bam_profiles());
        profiles.extend(cross_profiles());
        profiles.extend(vcf_profiles());
        profiles.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        Self { profiles }
    }
}
