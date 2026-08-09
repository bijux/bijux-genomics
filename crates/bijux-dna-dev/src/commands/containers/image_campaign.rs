use super::{path_from_arg, success_line, ContainerCommandOutcome, Result, Workspace};

const USAGE: &str = "Usage:\n  bijux-dna-dev containers run image-campaign -- execute <repository-root> <study-root> <apptainer> <attempt-root> <attempt-id> <workers> <tools-csv> <completed-at-utc>\n  bijux-dna-dev containers run image-campaign -- verify <repository-root> <study-root> <apptainer> <attempt-root>";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if matches!(args, [help] if matches!(help.as_str(), "--help" | "-h")) {
        return success_line(USAGE);
    }
    if let [operation, repository_root, study_root, apptainer, attempt_root, attempt_id, workers, tools, completed_at] =
        args
    {
        if operation != "execute" {
            anyhow::bail!(USAGE);
        }
        let worker_count = workers
            .parse::<usize>()
            .map_err(|_| anyhow::anyhow!("image campaign worker count is invalid"))?;
        let selected_tools = tools
            .split(',')
            .map(str::trim)
            .filter(|tool| !tool.is_empty())
            .map(str::to_string)
            .collect::<std::collections::BTreeSet<_>>();
        let completion = bijux_dna_containers::api::execute_apptainer_image_campaign_attempt(
            &bijux_dna_containers::api::ImageCampaignAttemptRequest {
                campaign: bijux_dna_containers::api::ApptainerImageCampaignRequest {
                    repository_root: path_from_arg(workspace, repository_root),
                    study_root: path_from_arg(workspace, study_root),
                    apptainer_executable: path_from_arg(workspace, apptainer),
                    worker_count,
                    selected_tools: Some(selected_tools),
                },
                attempt_root: path_from_arg(workspace, attempt_root),
                attempt_id: attempt_id.clone(),
                slurm_job_id: std::env::var("SLURM_JOB_ID")
                    .map_err(|_| anyhow::anyhow!("SLURM_JOB_ID is required"))?,
                completed_at: completed_at.clone(),
            },
        )?;
        return success_line(serde_json::to_string_pretty(&completion)?);
    }
    let [operation, repository_root, study_root, apptainer, attempt_root] = args else {
        anyhow::bail!(USAGE);
    };
    if operation != "verify" {
        anyhow::bail!(USAGE);
    }
    let execution = bijux_dna_containers::api::verify_apptainer_image_campaign_receipt(
        &path_from_arg(workspace, repository_root),
        &path_from_arg(workspace, study_root),
        &path_from_arg(workspace, apptainer),
        &path_from_arg(workspace, attempt_root),
    )?;
    success_line(serde_json::to_string_pretty(&execution)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_requires_exact_verification_inputs() {
        let workspace = Workspace { root: "/workspace".into() };
        assert!(run(&workspace, &[]).is_err());
        assert!(run(&workspace, &["verify".into()]).is_err());
        assert!(run(
            &workspace,
            &[
                "admit".into(),
                "repository".into(),
                "study".into(),
                "apptainer".into(),
                "attempt".into(),
            ],
        )
        .is_err());
        assert!(run(
            &workspace,
            &[
                "execute".into(),
                "repository".into(),
                "study".into(),
                "apptainer".into(),
                "attempt".into(),
                "attempt-001".into(),
                "2".into(),
                "bijux_dna,picard".into(),
                "2026-08-09T07:10:00Z".into(),
            ],
        )
        .is_err());
        assert!(run(&workspace, &["--help".into()]).is_ok());
    }
}
