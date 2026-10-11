//! Owner: bijux-dna-runner
//! Governed container image campaign Slurm lifecycle.

mod contracts;
mod lifecycle;
mod planning;
mod preparation;
mod validation;

pub use contracts::{
    ImageCampaignCollection, ImageCampaignEvidence, ImageCampaignEvidenceStatus,
    ImageCampaignSlurmPlanRequest, ImageCampaignSlurmResources, ImageCampaignSlurmSpec,
    PreparedImageCampaignSlurmJob, IMAGE_CAMPAIGN_EVIDENCE_SCHEMA_V1,
    IMAGE_CAMPAIGN_PREPARED_JOB_SCHEMA_V1, IMAGE_CAMPAIGN_SLURM_SPEC_SCHEMA_V1,
};
pub use lifecycle::{
    collect_image_campaign_slurm_job, load_image_campaign_slurm_job_handle,
    load_prepared_image_campaign_slurm_job, observe_image_campaign_slurm_job,
    submit_prepared_image_campaign_job,
};
pub use planning::plan_image_campaign_slurm_spec;
pub use preparation::prepare_image_campaign_slurm_job;
pub use validation::load_image_campaign_slurm_spec;
