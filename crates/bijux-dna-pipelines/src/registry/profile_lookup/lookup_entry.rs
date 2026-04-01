use anyhow::Result;

use crate::{Domain, PipelineProfile};

/// # Errors
/// Returns an error if the requested profile id is unknown for the domain.
pub fn profile_by_id(domain: Domain, id: &str) -> Result<PipelineProfile> {
    match domain {
        Domain::Fastq => crate::fastq::fastq_profiles_by_id(id),
        Domain::Bam => crate::bam::bam_profiles_by_id(id),
        Domain::Cross => super::cross::profile_by_id(id),
        Domain::Vcf => super::vcf::profile_by_id(id),
    }
}
