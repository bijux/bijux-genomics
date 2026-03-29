use anyhow::Result;
use bijux_dna_domain_bam::BamStage;
use bijux_dna_pipelines::bam::{
    bam_adna_capture_profile, bam_adna_shotgun_profile, bam_default_profile,
};
use bijux_dna_pipelines::PipelineProfile;

#[must_use]
pub fn adna_shotgun_profile() -> PipelineProfile {
    bam_adna_shotgun_profile()
}

#[must_use]
pub fn adna_capture_profile() -> PipelineProfile {
    bam_adna_capture_profile()
}

#[must_use]
pub fn profile_by_id(profile_id: &str) -> Option<PipelineProfile> {
    match profile_id {
        "bam-to-bam__default__v1" => Some(bam_default_profile()),
        "bam-to-bam__adna_shotgun__v1" => Some(bam_adna_shotgun_profile()),
        "bam-to-bam__adna_capture__v1" => Some(bam_adna_capture_profile()),
        _ => None,
    }
}

/// # Errors
/// Returns an error if the profile stage list contains an unknown BAM stage.
pub fn ordered_stages(profile: &PipelineProfile) -> Result<Vec<BamStage>> {
    profile
        .capabilities
        .required_stages
        .iter()
        .map(|stage_id| BamStage::try_from(stage_id.as_str()))
        .collect()
}

#[must_use]
pub fn pipeline_id_catalog(profile_id: &str) -> Vec<String> {
    let Some(profile) = profile_by_id(profile_id) else {
        return Vec::new();
    };
    ordered_stages(&profile)
        .unwrap_or_default()
        .iter()
        .map(|stage| stage.as_str().to_string())
        .collect()
}
