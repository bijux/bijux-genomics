//! Owner: image campaign contracts, admitted orchestration and current evidence verification.
mod admission;
mod execution;
mod model;
mod paths;
mod schema;
mod storage;
mod verification;
mod workflow;
pub use model::{
    ApptainerImageCampaignRequest, ImageCampaignAttemptRequest, ImageCampaignExecution,
    ImageCampaignJobCompletion, ImageCampaignPlan, ImageCampaignToolOutcome,
    ImageCampaignToolStatus, IMAGE_CAMPAIGN_EXECUTION_SCHEMA_V1,
    IMAGE_CAMPAIGN_JOB_COMPLETION_SCHEMA_V1, IMAGE_CAMPAIGN_PLAN_SCHEMA_V1,
};
pub use schema::{
    image_campaign_execution_schema, image_campaign_plan_schema,
    write_image_campaign_execution_schema, write_image_campaign_plan_schema,
};
pub use verification::verify_apptainer_image_campaign_receipt;
pub use workflow::{execute_apptainer_image_campaign, execute_apptainer_image_campaign_attempt};
