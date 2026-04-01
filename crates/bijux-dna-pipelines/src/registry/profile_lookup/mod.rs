//! Pipeline registry lookups by domain and id.

use anyhow::Result;

use crate::{Domain, PipelineProfile};

mod cross;
mod vcf;

/// # Errors
/// Returns an error if the requested profile id is unknown for the domain.
pub fn profile_by_id(domain: Domain, id: &str) -> Result<PipelineProfile> {
    match domain {
        Domain::Fastq => crate::fastq::fastq_profiles_by_id(id),
        Domain::Bam => crate::bam::bam_profiles_by_id(id),
        Domain::Cross => cross::profile_by_id(id),
        Domain::Vcf => vcf::profile_by_id(id),
    }
}
