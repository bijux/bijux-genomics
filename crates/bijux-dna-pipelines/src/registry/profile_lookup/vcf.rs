use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::id_catalog;

use crate::vcf::{vcf_minimal_profile, vcf_reference_basic_profile};
use crate::PipelineProfile;

pub(super) fn profile_by_id(id: &str) -> Result<PipelineProfile> {
    match id {
        id_catalog::PIPELINE_VCF_MINIMAL => Ok(vcf_minimal_profile()),
        id_catalog::PIPELINE_VCF_REFERENCE_BASIC => Ok(vcf_reference_basic_profile()),
        _ => Err(anyhow!("unknown VCF profile: {id}")),
    }
}
