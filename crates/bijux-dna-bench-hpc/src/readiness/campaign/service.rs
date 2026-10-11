use crate::readiness::assessment::fail_check;
use crate::readiness::assessment::ok_check;
use crate::readiness::campaign::profile::prepare_local_hpc_profile_support;
use crate::readiness::context::ReadinessContext;
use crate::readiness::declarations::DEFAULT_HPC_SUPPORT_ROOT;
use anyhow::Context;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;
use std::fs;

pub(in crate::readiness) fn evaluate_hpc_campaign_goals(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    let repo_root = context.repo_root;
    evaluate_campaign_stage_ids(context, checks);

    let support_root = repo_root.join(DEFAULT_HPC_SUPPORT_ROOT);
    match prepare_local_hpc_profile_support(&support_root) {
        Ok(support) => {
            let config_path =
                repo_root.join("benchmarks/configs/hpc/campaign/lunarc-fastq-bam-local-ready.toml");
            let dry_run = context.backend.campaign_dry_run(
                &config_path,
                Some(&support.env_file_path),
                Some(&support.policy_path),
            );
            let preparation = context.backend.prepare_foundation(
                &config_path,
                Some(&support.env_file_path),
                Some(&support.policy_path),
                false,
            );
            let submission = context.backend.submit_mock_campaign(&config_path, Some(&support.env_file_path), Some(&support.policy_path));

            match (dry_run, preparation, submission) {
                (Ok(dry_run), Ok(preparation), Ok(submission))
                    if dry_run.campaign_id == "adna-equus-caballus-local-ready"
                        && dry_run.planned_job_count == 9
                        && preparation.action_count == 3
                        && submission.jobs.len() == 9
                        && submission.jobs.iter().all(|job| {
                            repo_root.join(&job.script_path).is_file()
                                && repo_root.join(&job.code_path).is_file()
                        }) =>
                {
                    assess_prepared_profile_scripts(context, checks, &dry_run, &submission);
                }
                (dry_run, preparation, submission) => {
                    let mut details = Vec::new();
                    match dry_run {
                        Ok(report) => details.push(format!(
                            "dry-run planned {} jobs for `{}`",
                            report.planned_job_count,
                            report.campaign_id
                        )),
                        Err(err) => details.push(format!("campaign dry-run: {err:#}")),
                    }
                    match preparation {
                        Ok(report) => details.push(format!(
                            "prepare-foundation emitted {} actions",
                            report.action_count
                        )),
                        Err(err) => details.push(format!("prepare-foundation: {err:#}")),
                    }
                    match submission {
                        Ok(report) => details.push(format!(
                            "submit-campaign emitted {} jobs",
                            report.jobs.len()
                        )),
                        Err(err) => details.push(format!("submit-campaign: {err:#}")),
                    }
                    checks.push(fail_check(
                        99,
                        "slurm_dry_run",
                        "LUNARC local-ready profile dry-runs against prepared roots and prepared lock contracts",
                        Some(
                            "benchmarks/configs/hpc/campaign/lunarc-fastq-bam-local-ready.toml"
                                .to_string(),
                        ),
                        details.join("; "),
                    ));
                }
            }
        }
        Err(err) => checks.push(fail_check(
            99,
            "slurm_dry_run",
            "LUNARC local-ready profile dry-runs against prepared roots and prepared lock contracts",
            Some("benchmarks/configs/hpc/campaign/lunarc-fastq-bam-local-ready.toml".to_string()),
            format!("{err:#}"),
        )),
    }
}

pub(super) fn evaluate_campaign_stage_ids(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    let repo_root = context.repo_root;
    let cross_campaign_path = repo_root.join("benchmarks/configs/hpc/campaign/cross-mini.toml");
    let generic_campaign_path =
        repo_root.join("benchmarks/configs/hpc/campaign/generic-small.toml");
    match (
        context.backend.campaign_dry_run(&cross_campaign_path, None, None),
        context.backend.campaign_dry_run(&generic_campaign_path, None, None),
    ) {
        (Ok(cross_report), Ok(generic_report)) => checks.push(ok_check(
            91,
            "slurm_dry_run",
            "HPC campaign configs reference only real governed stage IDs",
            Some("benchmarks/configs/hpc/campaign/cross-mini.toml".to_string()),
            format!(
                "campaign stage validation passes for `{}` and `{}`",
                cross_report.campaign_id, generic_report.campaign_id
            ),
        )),
        (cross_result, generic_result) => {
            let mut details = Vec::new();
            if let Err(err) = cross_result {
                details.push(format!("cross-mini: {err:#}"));
            }
            if let Err(err) = generic_result {
                details.push(format!("generic-small: {err:#}"));
            }
            checks.push(fail_check(
                91,
                "slurm_dry_run",
                "HPC campaign configs reference only real governed stage IDs",
                Some("benchmarks/configs/hpc/campaign/cross-mini.toml".to_string()),
                details.join("; "),
            ));
        }
    }
}

pub(super) fn assess_prepared_profile_scripts(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    dry_run: &crate::readiness::ports::CampaignPlanObservation,
    submission: &crate::readiness::ports::MockSubmissionObservation,
) {
    let repo_root = context.repo_root;
    let first_script_path = repo_root.join(&submission.jobs[0].script_path);
    let script_body = fs::read_to_string(&first_script_path)
        .with_context(|| format!("read {}", first_script_path.display()));
    match script_body {
                        Ok(script_body)
                            if script_body.contains("BIJUX_CORPORA_PREPARE_LOCK=")
                                && script_body.contains("BIJUX_DATABASES_PREPARE_LOCK=")
                                && script_body.contains("BIJUX_IMAGES_PREPARE_LOCK=") =>
                        {
                            checks.push(ok_check(
                                99,
                                "slurm_dry_run",
                                "LUNARC local-ready profile dry-runs against prepared roots and prepared lock contracts",
                                Some(
                                    "benchmarks/configs/hpc/campaign/lunarc-fastq-bam-local-ready.toml"
                                        .to_string(),
                                ),
                                format!(
                                    "LUNARC profile dry-run planned {} jobs and mock submission wrote {} scripts",
                                    dry_run.planned_job_count,
                                    submission.jobs.len()
                                ),
                            ));
                        }
                        Ok(_) => checks.push(fail_check(
                            99,
                            "slurm_dry_run",
                            "LUNARC local-ready profile dry-runs against prepared roots and prepared lock contracts",
                            Some(
                                "benchmarks/configs/hpc/campaign/lunarc-fastq-bam-local-ready.toml"
                                    .to_string(),
                            ),
                            "mock submission scripts are missing prepared foundation lock exports"
                                .to_string(),
                        )),
                        Err(err) => checks.push(fail_check(
                            99,
                            "slurm_dry_run",
                            "LUNARC local-ready profile dry-runs against prepared roots and prepared lock contracts",
                            Some(
                                "benchmarks/configs/hpc/campaign/lunarc-fastq-bam-local-ready.toml"
                                    .to_string(),
                            ),
                            format!("{err:#}"),
                        )),
                    }
}
