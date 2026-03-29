//! BAM pipeline profiles and default params.

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::id_catalog;

use crate::PipelineProfile;

pub use super::adna_profiles::{
    bam_adna_capture_profile, bam_adna_profile, bam_adna_shotgun_profile,
    bam_reference_adna_profile,
};
pub use super::baseline_profiles::bam_default_profile;

/// # Errors
/// Returns an error if the requested profile id is unknown.
pub fn bam_profiles_by_id(id: &str) -> Result<PipelineProfile> {
    match id {
        id_catalog::PIPELINE_BAM_DEFAULT => Ok(super::baseline_profiles::bam_default_profile()),
        id_catalog::PIPELINE_BAM_ADNA_SHOTGUN => Ok(super::adna_profiles::bam_adna_shotgun_profile()),
        id_catalog::PIPELINE_BAM_ADNA_CAPTURE => Ok(super::adna_profiles::bam_adna_capture_profile()),
        id_catalog::PIPELINE_BAM_REFERENCE_ADNA => {
            Ok(super::adna_profiles::bam_reference_adna_profile())
        }
        _ => Err(anyhow!("unknown BAM profile: {id}")),
    }
}
