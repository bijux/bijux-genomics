//! Pure campaign resource selection and preflight decisions over explicit observations.
use crate::{
    is_supported_encryption_backend, CampaignCheck, CampaignJob, CampaignPreflightInputs,
    CampaignPreflightReport, ResourceTemplates, CAMPAIGN_SCHEMA_VERSION,
};

#[must_use]
pub fn resolve_hpc_campaign_resource_template(
    job: &CampaignJob,
    resources: &ResourceTemplates,
    fallback_default: &str,
) -> String {
    if let Some(explicit) =
        job.resource_template.as_deref().map(str::trim).filter(|value| !value.is_empty())
    {
        return explicit.to_string();
    }
    let mut best_match: Option<(&str, &str)> = None;
    for (stage_prefix, template_name) in &resources.stage_defaults {
        if (job.stage == *stage_prefix || job.stage.starts_with(&format!("{stage_prefix}.")))
            && best_match.is_none_or(|(best_prefix, _)| stage_prefix.len() > best_prefix.len())
        {
            best_match = Some((stage_prefix.as_str(), template_name.as_str()));
        }
    }
    best_match.map_or_else(
        || fallback_default.to_string(),
        |(_, template_name)| template_name.to_string(),
    )
}

fn looks_like_age_recipient(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("age1") else {
        return false;
    };
    rest.len() >= 10 && rest.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
}

/// Evaluate local readiness without reading files, the environment or executable search paths.
///
/// # Errors
/// Returns an invalid observation when layout facts do not match the configured roots.
pub fn evaluate_campaign_preflight(
    inputs: CampaignPreflightInputs<'_>,
) -> Result<CampaignPreflightReport, crate::BenchError> {
    let CampaignPreflightInputs {
        config,
        config_path,
        env_file_path,
        user_policy_path,
        user_policies_applied,
        resolved_slurm,
        observations,
    } = inputs;
    validate_layout_observations(config, &observations)?;
    let mut checks = vec![
        CampaignCheck {
            name: "schema_version".to_string(),
            ok: config.schema_version == CAMPAIGN_SCHEMA_VERSION,
            detail: config.schema_version.clone(),
        },
        CampaignCheck {
            name: "campaign_id_non_empty".to_string(),
            ok: !config.campaign.id.trim().is_empty(),
            detail: config.campaign.id.clone(),
        },
        CampaignCheck {
            name: "jobs_non_empty".to_string(),
            ok: !config.jobs.is_empty(),
            detail: format!("jobs={}", config.jobs.len()),
        },
    ];
    append_scheduler_checks(&resolved_slurm, &mut checks);
    append_security_checks(config, &observations, env_file_path, &mut checks);
    append_layout_checks(&observations, &mut checks);
    append_resource_checks(config, &resolved_slurm, &mut checks);
    let ok = checks.iter().all(|check| check.ok);
    Ok(CampaignPreflightReport {
        schema_version: CAMPAIGN_SCHEMA_VERSION,
        config_path: config_path.display().to_string(),
        env_file_path: env_file_path.display().to_string(),
        user_policy_path: user_policy_path.display().to_string(),
        user_policies_applied,
        checks,
        resolved_slurm,
        ok,
    })
}

fn validate_layout_observations(
    config: &crate::CampaignConfig,
    observations: &crate::CampaignPreflightObservations,
) -> Result<(), crate::BenchError> {
    let expected = [
        ("corpora_root", &config.layout.corpora_root),
        ("databases_root", &config.layout.databases_root),
        ("images_root", &config.layout.images_root),
        ("scratch_root", &config.layout.scratch_root),
        ("logs_root", &config.layout.logs_root),
        ("encrypted_results_root", &config.layout.encrypted_results_root),
        ("encrypted_code_root", &config.layout.encrypted_code_root),
        ("appraiser_imports_root", &config.layout.appraiser_imports_root),
        ("baselines_root", &config.layout.baselines_root),
    ];
    let by_name = observations
        .layout
        .iter()
        .map(|row| (row.name.as_str(), &row.path))
        .collect::<std::collections::BTreeMap<_, _>>();
    if observations.layout.len() != expected.len()
        || by_name.len() != expected.len()
        || expected.into_iter().any(|(name, path)| by_name.get(name) != Some(&path))
    {
        return Err(crate::BenchError::InvalidObservation { reason:"campaign preflight observations must cover each configured layout root exactly once".to_string() });
    }
    Ok(())
}

fn append_scheduler_checks(resolved_slurm: &crate::ResolvedSlurm, checks: &mut Vec<CampaignCheck>) {
    checks.push(CampaignCheck {
        name: "slurm_account_resolved".to_string(),
        ok: resolved_slurm.account_redacted != "<missing>",
        detail: resolved_slurm.account_redacted.clone(),
    });
    checks.push(CampaignCheck {
        name: "slurm_project_resolved".to_string(),
        ok: resolved_slurm.project_redacted != "<missing>",
        detail: resolved_slurm.project_redacted.clone(),
    });

    checks.push(CampaignCheck {
        name: "slurm_partition_resolved".to_string(),
        ok: resolved_slurm.partition != "<missing>",
        detail: resolved_slurm.partition.clone(),
    });

    checks.push(CampaignCheck {
        name: "slurm_qos_resolved".to_string(),
        ok: resolved_slurm.qos != "<missing>",
        detail: resolved_slurm.qos.clone(),
    });
    checks.push(CampaignCheck {
        name: "slurm_retry_attempts_valid".to_string(),
        ok: resolved_slurm.retry_attempts >= 1,
        detail: resolved_slurm.retry_attempts.to_string(),
    });
    checks.push(CampaignCheck {
        name: "slurm_retry_backoff_valid".to_string(),
        ok: resolved_slurm.retry_backoff_seconds >= 1,
        detail: resolved_slurm.retry_backoff_seconds.to_string(),
    });
}

fn append_security_checks(
    config: &crate::CampaignConfig,
    observations: &crate::CampaignPreflightObservations,
    env_file_path: &std::path::Path,
    checks: &mut Vec<CampaignCheck>,
) {
    checks.push(CampaignCheck {
        name: "encryption_recipients_present".to_string(),
        ok: !config.security.encryption_recipients.is_empty(),
        detail: format!("count={}", config.security.encryption_recipients.len()),
    });
    checks.push(CampaignCheck {
        name: "encryption_backend_supported".to_string(),
        ok: is_supported_encryption_backend(&config.security.encryption_backend),
        detail: config.security.encryption_backend.clone(),
    });
    if config.security.encryption_backend == "age-cli" {
        checks.push(CampaignCheck {
            name: "age_cli_available".to_string(),
            ok: observations.commands.get("age").copied().unwrap_or(false),
            detail: "required for age-cli backend".to_string(),
        });
        checks.push(CampaignCheck {
            name: "age_recipients_valid".to_string(),
            ok: config
                .security
                .encryption_recipients
                .iter()
                .all(|recipient| looks_like_age_recipient(recipient)),
            detail: format!("count={}", config.security.encryption_recipients.len()),
        });
        checks.push(CampaignCheck {
            name: "age_identity_files_configured".to_string(),
            ok: !config.security.encryption_identity_files.is_empty(),
            detail: format!("count={}", config.security.encryption_identity_files.len()),
        });
    }
    checks.push(CampaignCheck {
        name: "env_file_private".to_string(),
        ok: observations.env_file_private.unwrap_or(true),
        detail: env_file_path.display().to_string(),
    });
    for command in ["apptainer", "sbatch", "squeue", "sacct"] {
        checks.push(CampaignCheck {
            name: format!("command_available:{command}"),
            ok: observations.commands.get(command).copied().unwrap_or(false),
            detail: command.to_string(),
        });
    }
}

fn append_layout_checks(
    observations: &crate::CampaignPreflightObservations,
    checks: &mut Vec<CampaignCheck>,
) {
    for observation in &observations.layout {
        let name = &observation.name;
        let path = &observation.path;
        checks.push(CampaignCheck {
            name: format!("layout_{name}_absolute"),
            ok: path.is_absolute(),
            detail: path.display().to_string(),
        });
        checks.push(CampaignCheck {
            name: format!("layout_{name}_exists"),
            ok: observation.exists,
            detail: path.display().to_string(),
        });
        checks.push(CampaignCheck {
            name: format!("layout_{name}_writable"),
            ok: observation.writable,
            detail: path.display().to_string(),
        });
    }
}

fn append_resource_checks(
    config: &crate::CampaignConfig,
    resolved_slurm: &crate::ResolvedSlurm,
    checks: &mut Vec<CampaignCheck>,
) {
    let default_template_exists =
        config.resources.templates.contains_key(&resolved_slurm.default_resource_template);
    checks.push(CampaignCheck {
        name: "default_resource_template_exists".to_string(),
        ok: default_template_exists,
        detail: resolved_slurm.default_resource_template.clone(),
    });

    for job in &config.jobs {
        let template_name = resolve_hpc_campaign_resource_template(
            job,
            &config.resources,
            &resolved_slurm.default_resource_template,
        );
        checks.push(CampaignCheck {
            name: format!("job_template_present:{}:{}", job.stage, job.sample),
            ok: config.resources.templates.contains_key(&template_name),
            detail: template_name,
        });
        if let Some(class_name) =
            job.corpus_size_class.as_deref().map(str::trim).filter(|value| !value.is_empty())
        {
            checks.push(CampaignCheck {
                name: format!("job_corpus_size_class_present:{}:{}", job.stage, job.sample),
                ok: config.resources.corpus_size_scaling.contains_key(class_name),
                detail: class_name.to_string(),
            });
        }
    }

    for (class_name, policy) in &config.resources.corpus_size_scaling {
        let ok = policy.cpu_percent > 0
            && policy.mem_percent > 0
            && policy.walltime_percent > 0
            && policy.scratch_percent > 0;
        checks.push(CampaignCheck {
            name: format!("corpus_size_policy_nonzero:{class_name}"),
            ok,
            detail: format!(
                "cpu={} mem={} walltime={} scratch={}",
                policy.cpu_percent,
                policy.mem_percent,
                policy.walltime_percent,
                policy.scratch_percent
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn campaign_preflight_rejects_missing_duplicate_and_misbound_layout_facts() -> anyhow::Result<()>
    {
        let names = [
            "corpora_root",
            "databases_root",
            "images_root",
            "scratch_root",
            "logs_root",
            "encrypted_results_root",
            "encrypted_code_root",
            "appraiser_imports_root",
            "baselines_root",
        ];
        let layout = names
            .iter()
            .map(|name| (name.to_string(), serde_json::json!(name)))
            .collect::<serde_json::Map<_, _>>();
        let config: crate::CampaignConfig = serde_json::from_value(
            serde_json::json!({"campaign":{"id":"local","domain":"fastq"},"layout":layout,"jobs":[]}),
        )?;
        let mut observed = crate::CampaignPreflightObservations {
            env_file_private: None,
            commands: std::collections::BTreeMap::new(),
            layout: names
                .iter()
                .map(|name| crate::CampaignLayoutObservation {
                    name: name.to_string(),
                    path: std::path::PathBuf::from(name),
                    exists: true,
                    writable: true,
                })
                .collect(),
        };
        validate_layout_observations(&config, &observed)?;
        observed.layout[0].path = "unrelated".into();
        assert!(validate_layout_observations(&config, &observed).is_err());
        observed.layout[0] = observed.layout[1].clone();
        assert!(validate_layout_observations(&config, &observed).is_err());
        observed.layout.clear();
        assert!(validate_layout_observations(&config, &observed).is_err());
        Ok(())
    }
}
