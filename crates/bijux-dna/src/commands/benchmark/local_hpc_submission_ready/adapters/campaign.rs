use super::LocalReadinessAdapter;
use anyhow::Result;
use bijux_dna_bench_hpc::readiness::ports::{
    CampaignPlanObservation, FoundationPreparationObservation, LocalCampaignEvidence,
    MockSubmissionObservation, SubmittedScriptObservation,
};
use std::path::Path;
impl LocalCampaignEvidence for LocalReadinessAdapter {
    fn campaign_dry_run(
        &self,
        config_path: &Path,
        env_file_policy: Option<&Path>,
        user_policy_path: Option<&Path>,
    ) -> Result<CampaignPlanObservation> {
        let report =
            crate::commands::hpc::campaign_dry_run(config_path, env_file_policy, user_policy_path)?;
        Ok(CampaignPlanObservation {
            campaign_id: report.campaign_id,
            planned_job_count: report.planned_jobs.len(),
        })
    }
    fn prepare_foundation(
        &self,
        config_path: &Path,
        env_file_policy: Option<&Path>,
        user_policy_path: Option<&Path>,
        dry_run: bool,
    ) -> Result<FoundationPreparationObservation> {
        let report = crate::commands::hpc::prepare_foundation(
            config_path,
            env_file_policy,
            user_policy_path,
            dry_run,
        )?;
        Ok(FoundationPreparationObservation { action_count: report.actions.len() })
    }
    fn submit_mock_campaign(
        &self,
        config_path: &Path,
        env_file_policy: Option<&Path>,
        user_policy_path: Option<&Path>,
    ) -> Result<MockSubmissionObservation> {
        let report = crate::commands::hpc::submit_campaign(
            &crate::commands::cli::SlurmSubmitCampaignArgs {
                config: config_path.to_path_buf(),
                env_file: env_file_policy.map(Path::to_path_buf),
                user_policies: user_policy_path.map(Path::to_path_buf),
                mock_submit: true,
                json: false,
            },
        )?;
        Ok(MockSubmissionObservation {
            jobs: report
                .jobs
                .into_iter()
                .map(|job| SubmittedScriptObservation {
                    script_path: job.script_path,
                    code_path: job.code_path,
                })
                .collect(),
        })
    }
}
