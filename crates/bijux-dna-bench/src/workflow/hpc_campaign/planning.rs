//! Owner: benchmark local campaign planning.
//! Campaign planning owned by the workflow boundary.
use crate::workflow::hpc_campaign::configuration::resolve_campaign_config;
use crate::workflow::hpc_campaign::resources::apply_corpus_size_scaling;
use crate::workflow::hpc_campaign::templates::fill_template;
use crate::workflow::hpc_campaign::CampaignContext;
use anyhow::{anyhow, Result};
use bijux_dna_bench_model::resolve_hpc_campaign_resource_template as resolve_job_resource_template;
use bijux_dna_bench_model::{
    CampaignDryRunReport, CampaignJob, PlannedJob, PlannedLayout, PlannedOutputs, PlannedSecurity,
    CAMPAIGN_SCHEMA_VERSION,
};
use std::collections::BTreeMap;
use std::path::Path;

pub(crate) fn normalize_job_name(value: &str) -> String {
    value
        .chars()
        .map(
            |ch| {
                if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.') {
                    ch
                } else {
                    '_'
                }
            },
        )
        .collect::<String>()
}

pub(crate) fn campaign_job_name(job: &CampaignJob, index: usize) -> String {
    job.name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(normalize_job_name)
        .unwrap_or_else(|| {
            normalize_job_name(&format!("{}-{}-{}-{}", job.stage, job.tool, job.sample, index + 1))
        })
}

pub fn campaign_dry_run(
    context: &CampaignContext,
    config_path: &Path,
    env_file_policy: Option<&Path>,
    user_policy_path: Option<&Path>,
) -> Result<CampaignDryRunReport> {
    let (config, resolved_slurm, metadata) =
        resolve_campaign_config(context, config_path, env_file_policy, user_policy_path)?;
    let timestamp = context.timestamp.clone();
    let mut planned_jobs = Vec::new();

    for (index, job) in config.jobs.iter().enumerate() {
        let job_id = format!("dryrun-{:04}", index + 1);
        let job_name = campaign_job_name(job, index);
        let template_name = resolve_job_resource_template(
            job,
            &config.resources,
            &resolved_slurm.default_resource_template,
        );

        let base_resources = config.resources.templates.get(&template_name).ok_or_else(|| {
            anyhow!("job `{}` references unknown resource template `{template_name}`", job.stage)
        })?;
        let corpus_size_class =
            job.corpus_size_class.as_deref().map(str::trim).filter(|value| !value.is_empty());
        let scaling_policy = match corpus_size_class {
            Some(class_name) => {
                Some(config.resources.corpus_size_scaling.get(class_name).ok_or_else(|| {
                    anyhow!(
                        "job `{}` references unknown corpus_size_class `{class_name}`",
                        job.stage
                    )
                })?)
            }
            None => None,
        };
        let resources = apply_corpus_size_scaling(base_resources, scaling_policy)?;

        let mut values = BTreeMap::new();
        values.insert("job_id".to_string(), job_id.clone());
        values.insert("timestamp".to_string(), timestamp.clone());
        values.insert("campaign".to_string(), config.campaign.id.clone());
        values.insert("domain".to_string(), config.campaign.domain.clone());
        values.insert("stage".to_string(), job.stage.clone());
        values.insert("tool".to_string(), job.tool.clone());
        values.insert("sample".to_string(), job.sample.clone());
        values.insert("array_task".to_string(), job.array_task.unwrap_or(0).to_string());

        let outputs = PlannedOutputs {
            scratch_dir: config
                .layout
                .scratch_root
                .join(format!(
                    "{}/{}/{}/{}/{}/{}-{}",
                    config.campaign.id,
                    config.campaign.domain,
                    job.stage,
                    job.tool,
                    job.sample,
                    job_id,
                    timestamp
                ))
                .display()
                .to_string(),
            log: config
                .layout
                .logs_root
                .join(fill_template(&config.output_templates.log, &values)?)
                .display()
                .to_string(),
            out: config
                .layout
                .logs_root
                .join(fill_template(&config.output_templates.out, &values)?)
                .display()
                .to_string(),
            err: config
                .layout
                .logs_root
                .join(fill_template(&config.output_templates.err, &values)?)
                .display()
                .to_string(),
            results: config
                .layout
                .encrypted_results_root
                .join(fill_template(&config.output_templates.results, &values)?)
                .display()
                .to_string(),
            code: config
                .layout
                .encrypted_code_root
                .join(fill_template(&config.output_templates.code, &values)?)
                .display()
                .to_string(),
        };

        planned_jobs.push(PlannedJob {
            job_id,
            job_name,
            stage: job.stage.clone(),
            tool: job.tool.clone(),
            sample: job.sample.clone(),
            corpus_size_class: corpus_size_class.map(ToOwned::to_owned),
            array_task: job.array_task,
            depends_on: job.depends_on.clone(),
            resource_template: template_name,
            resources,
            outputs,
        });
    }

    Ok(CampaignDryRunReport {
        schema_version: CAMPAIGN_SCHEMA_VERSION,
        config_path: config_path.display().to_string(),
        env_file_path: metadata.env_file_path.display().to_string(),
        user_policy_path: metadata.user_policy_path.display().to_string(),
        user_policies_applied: metadata.user_policies_applied,
        campaign_id: config.campaign.id,
        domain: config.campaign.domain,
        layout: PlannedLayout {
            corpora_root: config.layout.corpora_root.display().to_string(),
            databases_root: config.layout.databases_root.display().to_string(),
            images_root: config.layout.images_root.display().to_string(),
            scratch_root: config.layout.scratch_root.display().to_string(),
            logs_root: config.layout.logs_root.display().to_string(),
            encrypted_results_root: config.layout.encrypted_results_root.display().to_string(),
            encrypted_code_root: config.layout.encrypted_code_root.display().to_string(),
            appraiser_imports_root: config.layout.appraiser_imports_root.display().to_string(),
            baselines_root: config.layout.baselines_root.display().to_string(),
        },
        resolved_slurm,
        security: PlannedSecurity {
            encryption_backend: config.security.encryption_backend,
            encryption_recipients: config.security.encryption_recipients,
            encryption_identity_files: config
                .security
                .encryption_identity_files
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            encrypt_operator_outputs: config.security.encrypt_operator_outputs,
            redacted_env_keys: config.security.redacted_env_keys,
        },
        planned_jobs,
    })
}
