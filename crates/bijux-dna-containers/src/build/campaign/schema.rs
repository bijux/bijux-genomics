//! Owner: canonical image campaign schema projection and schema publication.
use super::model::{ImageCampaignExecution, ImageCampaignPlan};
use anyhow::Result;
use schemars::schema_for;
use serde_json::Value;
use std::path::Path;
#[must_use]
pub fn image_campaign_plan_schema() -> Value {
    serde_json::to_value(schema_for!(ImageCampaignPlan)).unwrap_or_else(|_| serde_json::json!({}))
}

#[must_use]
pub fn image_campaign_execution_schema() -> Value {
    serde_json::to_value(schema_for!(ImageCampaignExecution))
        .unwrap_or_else(|_| serde_json::json!({}))
}

/// Write the retained-image campaign plan schema atomically.
///
/// # Errors
/// Returns an error when the destination cannot be written.
pub fn write_image_campaign_plan_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &image_campaign_plan_schema())?;
    Ok(())
}

/// Write the retained-image campaign execution schema atomically.
///
/// # Errors
/// Returns an error when the destination cannot be written.
pub fn write_image_campaign_execution_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &image_campaign_execution_schema())?;
    Ok(())
}
