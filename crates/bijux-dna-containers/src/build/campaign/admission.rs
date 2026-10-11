//! Owner: pure campaign identity, tool-scope and progress outcome admission.
use super::model::{
    ImageCampaignAttemptRequest, ImageCampaignExecution, ImageCampaignToolOutcome,
    ImageCampaignToolStatus, IMAGE_CAMPAIGN_EXECUTION_SCHEMA_V1,
};
use anyhow::{bail, Context, Result};
use chrono::DateTime;
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn validate_attempt_identity(request: &ImageCampaignAttemptRequest) -> Result<()> {
    if request.attempt_id.is_empty()
        || request.attempt_id.contains('/')
        || request.attempt_id.contains('\\')
        || request.attempt_id == "."
        || request.attempt_id == ".."
    {
        bail!("image campaign attempt identity is invalid");
    }
    if request.slurm_job_id.is_empty()
        || !request.slurm_job_id.bytes().all(|byte| byte.is_ascii_digit())
    {
        bail!("image campaign Slurm job identity is invalid");
    }
    DateTime::parse_from_rfc3339(&request.completed_at)
        .context("image campaign completion timestamp is invalid")?;
    Ok(())
}

pub(super) fn selected_receipt_tools<'a>(
    redistributable_tools: &[String],
    execution_tools: impl Iterator<Item = &'a String>,
) -> Result<Vec<String>> {
    let execution_tools = execution_tools.cloned().collect::<BTreeSet<_>>();
    let selected_tools = select_campaign_tools(redistributable_tools, Some(&execution_tools))?;
    if selected_tools.len() != execution_tools.len() {
        bail!("image campaign receipt tool roster is invalid");
    }
    Ok(selected_tools)
}

pub(super) fn select_campaign_tools(
    redistributable_tools: &[String],
    selected_tools: Option<&BTreeSet<String>>,
) -> Result<Vec<String>> {
    let Some(selected_tools) = selected_tools else {
        return Ok(redistributable_tools.to_vec());
    };
    if selected_tools.is_empty() {
        bail!("selected image campaign contains no tools");
    }
    let redistributable = redistributable_tools.iter().cloned().collect::<BTreeSet<_>>();
    let unknown = selected_tools.difference(&redistributable).cloned().collect::<Vec<_>>();
    if !unknown.is_empty() {
        bail!(
            "selected image campaign contains tools outside the redistributable-image roster: {}",
            unknown.join(",")
        );
    }
    Ok(redistributable_tools
        .iter()
        .filter(|tool| selected_tools.contains(*tool))
        .cloned()
        .collect())
}

pub(super) fn campaign_scope(tools: &[String], redistributable_count: usize) -> Option<String> {
    (tools.len() != redistributable_count).then(|| {
        format!("selected-{}", bijux_dna_infra::hash_bytes_sha256(tools.join("\n").as_bytes()))
    })
}

pub(super) fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(super) fn execution_from_tools(
    repository_commit: String,
    catalog_sha256: String,
    architecture: String,
    tools: BTreeMap<String, ImageCampaignToolOutcome>,
) -> ImageCampaignExecution {
    let passed =
        tools.values().filter(|item| item.status == ImageCampaignToolStatus::Passed).count();
    let failed =
        tools.values().filter(|item| item.status == ImageCampaignToolStatus::Failed).count();
    ImageCampaignExecution {
        schema_version: IMAGE_CAMPAIGN_EXECUTION_SCHEMA_V1.to_string(),
        repository_commit,
        catalog_sha256,
        architecture,
        tools,
        passed,
        failed,
    }
}

pub(super) fn validate_execution(
    execution: &ImageCampaignExecution,
    commit: &str,
    catalog_sha256: &str,
    architecture: &str,
) -> Result<()> {
    if execution.schema_version != IMAGE_CAMPAIGN_EXECUTION_SCHEMA_V1
        || execution.repository_commit != commit
        || execution.catalog_sha256 != catalog_sha256
        || execution.architecture != architecture
    {
        bail!("image campaign progress identity differs from current source");
    }
    let expected = execution_from_tools(
        commit.to_string(),
        catalog_sha256.to_string(),
        architecture.to_string(),
        execution.tools.clone(),
    );
    if execution.passed != expected.passed || execution.failed != expected.failed {
        bail!("image campaign progress counts differ from tool outcomes");
    }
    for (tool, outcome) in &execution.tools {
        match outcome.status {
            ImageCampaignToolStatus::Passed
                if outcome.publication.is_some() && outcome.error.is_none() => {}
            ImageCampaignToolStatus::Failed
                if outcome.publication.is_none()
                    && outcome.error.as_ref().is_some_and(|error| !error.trim().is_empty()) => {}
            _ => bail!("image campaign progress outcome is incoherent for {tool}"),
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::super::model::ApptainerImageCampaignRequest;
    use super::super::paths::{campaign_root, require_campaign_path};
    use super::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn selected_campaign_preserves_catalog_order_and_uses_scoped_root() -> Result<()> {
        let retained = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
        let selected = BTreeSet::from(["beta".to_string(), "gamma".to_string()]);

        let tools = select_campaign_tools(&retained, Some(&selected))?;
        assert_eq!(tools, ["beta", "gamma"]);
        let scope = campaign_scope(&tools, retained.len()).context("selected scope")?;
        assert!(scope.starts_with("selected-"));
        assert_eq!(
            campaign_root(Path::new("/study/campaigns"), "commit", Some(&scope)),
            Path::new("/study/campaigns").join("commit").join(scope)
        );
        assert!(require_campaign_path(
            Path::new("/study/campaigns/commit/selected-a/image.sif"),
            Path::new("/study/campaigns/commit/selected-a"),
            "image",
            "beta",
        )
        .is_ok());
        assert!(require_campaign_path(
            Path::new("/study/campaigns/commit/image.sif"),
            Path::new("/study/campaigns/commit/selected-a"),
            "image",
            "beta",
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn selected_campaign_rejects_empty_and_unknown_tool_sets() {
        let retained = vec!["alpha".to_string(), "beta".to_string()];
        assert!(select_campaign_tools(&retained, Some(&BTreeSet::new())).is_err());
        assert!(select_campaign_tools(&retained, Some(&BTreeSet::from(["unknown".to_string()])),)
            .is_err());
    }

    #[test]
    fn selected_receipt_accepts_exact_retained_subsets_only() -> Result<()> {
        let retained = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
        let selected = BTreeSet::from(["alpha".to_string(), "gamma".to_string()]);
        assert_eq!(selected_receipt_tools(&retained, selected.iter())?, ["alpha", "gamma"]);
        assert!(selected_receipt_tools(&retained, BTreeSet::<String>::new().iter()).is_err());
        let unknown = BTreeSet::from(["unknown".to_string()]);
        assert!(selected_receipt_tools(&retained, unknown.iter()).is_err());
        Ok(())
    }

    #[test]
    fn complete_campaign_preserves_historical_commit_root() -> Result<()> {
        let retained = vec!["alpha".to_string(), "beta".to_string()];
        let tools = select_campaign_tools(&retained, None)?;
        assert_eq!(campaign_scope(&tools, retained.len()), None);
        assert_eq!(
            campaign_root(Path::new("/study/campaigns"), "commit", None),
            Path::new("/study/campaigns/commit")
        );
        Ok(())
    }

    #[test]
    fn campaign_attempt_requires_stable_scheduler_identity() {
        let campaign = ApptainerImageCampaignRequest {
            repository_root: PathBuf::from("repository"),
            study_root: PathBuf::from("study"),
            apptainer_executable: PathBuf::from("apptainer"),
            worker_count: 1,
            selected_tools: None,
        };
        let valid = ImageCampaignAttemptRequest {
            campaign: campaign.clone(),
            attempt_root: PathBuf::from("attempt"),
            attempt_id: "attempt-001".to_string(),
            slurm_job_id: "3473000".to_string(),
            completed_at: "2026-08-09T07:10:00Z".to_string(),
        };
        assert!(validate_attempt_identity(&valid).is_ok());

        let mut invalid = valid.clone();
        invalid.attempt_id = "../attempt".to_string();
        assert!(validate_attempt_identity(&invalid).is_err());
        invalid = valid.clone();
        invalid.slurm_job_id = "job-3473000".to_string();
        assert!(validate_attempt_identity(&invalid).is_err());
        invalid = valid;
        invalid.completed_at = "not-a-timestamp".to_string();
        assert!(validate_attempt_identity(&invalid).is_err());
    }
    #[test]
    fn campaign_progress_rejects_stale_identity_counts_and_incoherent_outcomes() -> Result<()> {
        let outcomes = BTreeMap::from([(
            "alpha".to_string(),
            ImageCampaignToolOutcome {
                status: ImageCampaignToolStatus::Failed,
                publication: None,
                error: Some("build failed".into()),
            },
        )]);
        let execution = execution_from_tools(
            "commit".into(),
            "catalog".into(),
            "architecture".into(),
            outcomes,
        );
        validate_execution(&execution, "commit", "catalog", "architecture")?;
        assert!(validate_execution(&execution, "other", "catalog", "architecture").is_err());
        assert!(validate_execution(&execution, "commit", "other", "architecture").is_err());
        assert!(validate_execution(&execution, "commit", "catalog", "other").is_err());
        let mut changed = execution.clone();
        changed.schema_version = "unsupported".into();
        assert!(validate_execution(&changed, "commit", "catalog", "architecture").is_err());
        let mut changed = execution.clone();
        changed.failed = 0;
        assert!(validate_execution(&changed, "commit", "catalog", "architecture").is_err());
        let mut changed = execution.clone();
        changed.tools.get_mut("alpha").context("alpha outcome")?.error = None;
        assert!(validate_execution(&changed, "commit", "catalog", "architecture").is_err());
        let mut changed = execution;
        changed.tools.get_mut("alpha").context("alpha outcome")?.error = Some("  ".into());
        assert!(validate_execution(&changed, "commit", "catalog", "architecture").is_err());
        Ok(())
    }
}
