use super::observations::{
    CampaignPlanObservation, FoundationPreparationObservation, MockSubmissionObservation,
};
use anyhow::Result;
use std::path::Path;
/// Supply typed local campaign evidence; provider failures remain explicit readiness diagnostics.
pub trait LocalCampaignEvidence {
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn campaign_dry_run(
        &self,
        config_path: &Path,
        env_file_policy: Option<&Path>,
        user_policy_path: Option<&Path>,
    ) -> Result<CampaignPlanObservation>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn prepare_foundation(
        &self,
        config_path: &Path,
        env_file_policy: Option<&Path>,
        user_policy_path: Option<&Path>,
        dry_run: bool,
    ) -> Result<FoundationPreparationObservation>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn submit_mock_campaign(
        &self,
        config_path: &Path,
        env_file_policy: Option<&Path>,
        user_policy_path: Option<&Path>,
    ) -> Result<MockSubmissionObservation>;
}
