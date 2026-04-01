use crate::PipelineProfile;

#[derive(Debug, Clone)]
pub struct PipelineRegistry {
    profiles: Vec<PipelineProfile>,
}

impl PipelineRegistry {
    #[must_use]
    pub fn v1() -> Self {
        let mut profiles = Vec::new();
        profiles.extend(super::super::fastq_profiles());
        profiles.extend(super::super::bam_profiles());
        profiles.extend(super::super::cross_profiles());
        profiles.extend(super::super::vcf_profiles());
        profiles.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        Self { profiles }
    }
}
