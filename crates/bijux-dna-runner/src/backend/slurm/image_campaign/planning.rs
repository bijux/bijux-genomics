//! Source-locked planning for container image campaign scheduler specifications.

use std::collections::BTreeSet;

use anyhow::{bail, Context, Result};

use super::contracts::{
    ImageCampaignSlurmPlanRequest, ImageCampaignSlurmSpec, IMAGE_CAMPAIGN_SLURM_SPEC_SCHEMA_V1,
};
use super::preparation::ensure_attempt_root;
use super::validation::{
    ensure_json_identity, validate_specification, validate_submission_authorities,
};

/// Derive and persist one immutable image-campaign scheduler specification.
///
/// The repository commit, executable identities, governed attempt root, and canonical catalog tool
/// order are derived rather than accepted as caller assertions.
///
/// # Errors
/// Returns an error when source or executable authorities drift, tools are unknown or duplicated,
/// resources are invalid, or immutable attempt evidence conflicts.
pub fn plan_image_campaign_slurm_spec(
    request: &ImageCampaignSlurmPlanRequest,
) -> Result<ImageCampaignSlurmSpec> {
    let repository_path =
        request.repository_path.canonicalize().context("canonicalize image campaign repository")?;
    let repository_commit =
        bijux_dna_containers::api::clean_repository_head_commit(&repository_path)
            .context("require a clean image campaign repository")?;
    let executable_path =
        request.executable_path.canonicalize().context("canonicalize image campaign executable")?;
    let study_root = request.study_root.canonicalize().context("canonicalize STUDY root")?;
    let apptainer_path =
        request.apptainer_path.canonicalize().context("canonicalize Apptainer executable")?;
    let selected_tools = canonical_selected_tools(&repository_path, &request.selected_tools)?;
    let attempt_root = study_root
        .join("containers/evidence/image-campaign-submissions")
        .join(&repository_commit)
        .join("attempts")
        .join(&request.attempt_id);
    let spec = ImageCampaignSlurmSpec {
        schema_version: IMAGE_CAMPAIGN_SLURM_SPEC_SCHEMA_V1.to_string(),
        run_id: "run-1000".to_string(),
        attempt_id: request.attempt_id.clone(),
        repository_path,
        repository_commit,
        executable_sha256: bijux_dna_infra::hash_file_sha256(&executable_path)?,
        executable_path,
        study_root,
        apptainer_sha256: bijux_dna_infra::hash_file_sha256(&apptainer_path)?,
        apptainer_path,
        attempt_root,
        selected_tools,
        worker_count: request.worker_count,
        resources: request.resources.clone(),
    };
    validate_specification(&spec)?;
    validate_submission_authorities(&spec)?;
    ensure_attempt_root(&spec)?;
    ensure_json_identity(&spec.attempt_root.join("image-campaign-spec.json"), &spec)?;
    Ok(spec)
}

fn canonical_selected_tools(
    repository: &std::path::Path,
    requested: &[String],
) -> Result<Vec<String>> {
    let requested_set = requested.iter().cloned().collect::<BTreeSet<_>>();
    if requested_set.len() != requested.len() {
        bail!("image campaign selected tools are duplicated");
    }
    let catalog = bijux_dna_containers::api::load_container_catalog(repository)?;
    let retained = bijux_dna_containers::api::retained_apptainer_tool_ids(
        repository,
        &catalog.apptainer_architecture,
    )?;
    let selected =
        retained.into_iter().filter(|tool| requested_set.contains(tool)).collect::<Vec<_>>();
    if selected.len() != requested_set.len() {
        bail!("image campaign selected tools include an unknown or non-retained tool");
    }
    Ok(selected)
}
