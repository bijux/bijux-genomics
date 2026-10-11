use super::{path_from_arg, success_line, ContainerCommandOutcome, Result, Workspace};

const USAGE: &str = "Usage:\n  bijux-dna-dev containers run image-campaign -- plan <repository-root> <study-root> <executable> <apptainer> <attempt-id> <workers> <tools-csv> <account> <partition> <memory-mb> <walltime>\n  bijux-dna-dev containers run image-campaign -- prepare <campaign-spec> <submitted-at-utc>\n  bijux-dna-dev containers run image-campaign -- submit <campaign-spec>\n  bijux-dna-dev containers run image-campaign -- observe <campaign-spec> <observed-at-utc>\n  bijux-dna-dev containers run image-campaign -- collect <campaign-spec> <collected-at-utc>\n  bijux-dna-dev containers run image-campaign -- execute <repository-root> <study-root> <apptainer> <attempt-root> <attempt-id> <workers> <tools-csv> <completed-at-utc>\n  bijux-dna-dev containers run image-campaign -- verify <repository-root> <study-root> <apptainer> <attempt-root>";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let Some((operation, options)) = args.split_first() else {
        anyhow::bail!(USAGE);
    };
    if matches!(operation.as_str(), "--help" | "-h") {
        return success_line(USAGE);
    }
    match operation.as_str() {
        "plan" => plan(workspace, options),
        "prepare" => prepare(workspace, options),
        "submit" => submit(workspace, options),
        "observe" => observe(workspace, options),
        "collect" => collect(workspace, options),
        "execute" => execute(workspace, options),
        "verify" => verify(workspace, options),
        _ => anyhow::bail!(USAGE),
    }
}

fn plan(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [repository_root, study_root, executable, apptainer, attempt_id, workers, tools, account, partition, memory_mb, walltime] =
        args
    else {
        anyhow::bail!(USAGE);
    };
    let worker_count = workers
        .parse::<usize>()
        .map_err(|_| anyhow::anyhow!("image campaign worker count is invalid"))?;
    let cpus = u32::try_from(worker_count)
        .map_err(|_| anyhow::anyhow!("image campaign worker count exceeds CPU range"))?;
    let memory_mb = memory_mb
        .parse::<u64>()
        .map_err(|_| anyhow::anyhow!("image campaign memory is invalid"))?;
    let selected_tools = tools
        .split(',')
        .map(str::trim)
        .filter(|tool| !tool.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let spec = bijux_dna_runner::api::plan_image_campaign_slurm_spec(
        &bijux_dna_runner::api::ImageCampaignSlurmPlanRequest {
            repository_path: path_from_arg(workspace, repository_root),
            executable_path: path_from_arg(workspace, executable),
            study_root: path_from_arg(workspace, study_root),
            apptainer_path: path_from_arg(workspace, apptainer),
            attempt_id: attempt_id.clone(),
            selected_tools,
            worker_count,
            resources: bijux_dna_runner::api::ImageCampaignSlurmResources {
                account: account.clone(),
                partition: partition.clone(),
                cpus,
                memory_mb,
                walltime: walltime.clone(),
            },
        },
    )?;
    json_out(&serde_json::to_value(spec)?)
}

fn prepare(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [specification, submitted_at_utc] = args else {
        anyhow::bail!(USAGE);
    };
    let prepared = bijux_dna_runner::api::prepare_image_campaign_slurm_job(
        &load_spec(workspace, specification)?,
        submitted_at_utc,
    )?;
    json_out(&serde_json::to_value(prepared)?)
}

fn submit(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [specification] = args else {
        anyhow::bail!(USAGE);
    };
    let spec = load_spec(workspace, specification)?;
    let prepared = bijux_dna_runner::api::load_prepared_image_campaign_slurm_job(&spec)?;
    let handle = bijux_dna_runner::api::submit_prepared_image_campaign_job(
        &bijux_dna_runner::api::SlurmClientPaths::default(),
        &prepared,
    )?;
    json_out(&serde_json::to_value(handle)?)
}

fn observe(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [specification, observed_at_utc] = args else {
        anyhow::bail!(USAGE);
    };
    let observation = bijux_dna_runner::api::observe_image_campaign_slurm_job(
        &bijux_dna_runner::api::SlurmClientPaths::default(),
        &load_spec(workspace, specification)?,
        observed_at_utc,
    )?;
    json_out(&serde_json::to_value(observation)?)
}

fn collect(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [specification, collected_at_utc] = args else {
        anyhow::bail!(USAGE);
    };
    let collection = bijux_dna_runner::api::collect_image_campaign_slurm_job(
        &bijux_dna_runner::api::SlurmClientPaths::default(),
        &load_spec(workspace, specification)?,
        collected_at_utc,
    )?;
    json_out(&serde_json::to_value(collection)?)
}

fn execute(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [repository_root, study_root, apptainer, attempt_root, attempt_id, workers, tools, completed_at] =
        args
    else {
        anyhow::bail!(USAGE);
    };
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
    success_line(serde_json::to_string_pretty(&completion)?)
}

fn verify(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [repository_root, study_root, apptainer, attempt_root] = args else {
        anyhow::bail!(USAGE);
    };
    let execution = bijux_dna_containers::api::verify_apptainer_image_campaign_receipt(
        &path_from_arg(workspace, repository_root),
        &path_from_arg(workspace, study_root),
        &path_from_arg(workspace, apptainer),
        &path_from_arg(workspace, attempt_root),
    )?;
    success_line(serde_json::to_string_pretty(&execution)?)
}

fn load_spec(
    workspace: &Workspace,
    value: &str,
) -> Result<bijux_dna_runner::api::ImageCampaignSlurmSpec> {
    bijux_dna_runner::api::load_image_campaign_slurm_spec(&path_from_arg(workspace, value))
}

fn json_out(value: &serde_json::Value) -> Result<ContainerCommandOutcome> {
    Ok(ContainerCommandOutcome::success(format!("{}\n", serde_json::to_string_pretty(value)?)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_lists_non_waiting_lifecycle_and_rejects_incomplete_inputs() -> Result<()> {
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
        let help = run(&workspace, &["--help".into()])?;
        for operation in ["plan", "prepare", "submit", "observe", "collect", "execute", "verify"] {
            assert!(help.stdout.contains(operation));
        }
        Ok(())
    }
}
