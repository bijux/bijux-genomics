use std::collections::BTreeMap;
use std::fs;

use anyhow::{bail, Context, Result};

use crate::evidence::{
    apptainer_recipe_identity_at_commit, container_build_evidence_matches_recipe,
    load_container_catalog, repository_head_commit, BuildEvidenceStatus, ContainerImageCandidate,
    ContainerImageInventoryPublication, ContainerImageInventoryTool, ContainerImageState,
    ContainerSupportDisposition, ImageCandidateDisposition, ImageInventoryPublicationIdentity,
    CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V1,
};

use super::source::{
    load_build_evidence, load_run_references, portable_study_path, referenced_digests, scan_images,
    ImageInventoryDerivationRequest, IndexedBuildEvidence, RawImageIdentity, RawInventoryIdentity,
    RAW_INVENTORY_SCHEMA_V1,
};

type CandidateWithReference = (ContainerImageCandidate, bool);

/// Derive a portable inventory publication from immutable images, build evidence, and run locks.
///
/// # Errors
/// Returns an error when source evidence is malformed, ambiguous, outside governed roots, or does
/// not close over the exact catalog and committed Apptainer recipe identities.
pub fn derive_image_inventory_publication(
    request: &ImageInventoryDerivationRequest,
) -> Result<ContainerImageInventoryPublication> {
    let repository_commit = repository_head_commit(&request.repository_root)?;
    let catalog_path = request.repository_root.join("containers/contracts/catalog.toml");
    let catalog_bytes = fs::read(&catalog_path)
        .with_context(|| format!("read container catalog {}", catalog_path.display()))?;
    let catalog_sha256 = bijux_dna_infra::hash_bytes_sha256(&catalog_bytes);
    let committed_catalog_sha256 = crate::evidence::tool_contract::catalog_identity_at_commit(
        &request.repository_root,
        &repository_commit,
    )?;
    if catalog_sha256 != committed_catalog_sha256 {
        bail!("working container catalog differs from repository commit {repository_commit}");
    }
    let catalog = load_container_catalog(&request.repository_root)?;
    let images = scan_images(request)?;
    let build_evidence = load_build_evidence(request)?;
    let run_references = load_run_references(request)?;

    let tools = catalog
        .tools
        .iter()
        .map(|tool| {
            derive_tool(
                request,
                &repository_commit,
                tool,
                &images,
                &build_evidence.by_digest,
                &run_references,
            )
        })
        .collect::<Result<Vec<_>>>()?;

    let raw_identity = RawInventoryIdentity {
        schema_version: RAW_INVENTORY_SCHEMA_V1,
        repository_commit: repository_commit.clone(),
        catalog_sha256: catalog_sha256.clone(),
        images: images.iter().map(|(_, identity)| identity.clone()).collect(),
        build_evidence: build_evidence.identities,
        run_references,
    };
    let raw_bytes = serde_json::to_vec(&raw_identity).context("serialize raw image inventory")?;
    let publication = ContainerImageInventoryPublication {
        schema_version: CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V1.to_string(),
        catalog_sha256,
        repository_commit,
        publication: ImageInventoryPublicationIdentity {
            raw_schema_version: RAW_INVENTORY_SCHEMA_V1.to_string(),
            raw_sha256: bijux_dna_infra::hash_bytes_sha256(&raw_bytes),
        },
        tools,
    };
    crate::evidence::validate_image_inventory_publication(&publication)?;
    Ok(publication)
}

fn derive_tool(
    request: &ImageInventoryDerivationRequest,
    repository_commit: &str,
    tool: &crate::evidence::ContainerCatalogTool,
    images: &[(std::path::PathBuf, RawImageIdentity)],
    evidence_by_digest: &BTreeMap<String, Vec<IndexedBuildEvidence>>,
    run_references: &BTreeMap<String, Vec<String>>,
) -> Result<ContainerImageInventoryTool> {
    let support_disposition = support_disposition(&tool.support_disposition)?;
    if !tool.apptainer_supported {
        return Ok(ContainerImageInventoryTool {
            tool: tool.id.clone(),
            version: tool.version.clone(),
            support_disposition,
            apptainer_supported: false,
            state: ContainerImageState::Unsupported,
            selected_image_sha256: None,
            candidates: Vec::new(),
        });
    }

    let recipe = apptainer_recipe_identity_at_commit(
        &request.repository_root,
        repository_commit,
        &tool.apptainer_definition,
    )?;
    let candidates =
        derive_candidates(request, tool, &recipe, images, evidence_by_digest, run_references)?;
    let (state, selected, retained) =
        select_candidates(&tool.id, &support_disposition, candidates)?;
    verify_historical_references(&retained, run_references)?;
    Ok(ContainerImageInventoryTool {
        tool: tool.id.clone(),
        version: tool.version.clone(),
        support_disposition,
        apptainer_supported: true,
        state,
        selected_image_sha256: selected,
        candidates: retained,
    })
}

fn derive_candidates(
    request: &ImageInventoryDerivationRequest,
    tool: &crate::evidence::ContainerCatalogTool,
    recipe: &crate::evidence::ApptainerRecipeIdentity,
    images: &[(std::path::PathBuf, RawImageIdentity)],
    evidence_by_digest: &BTreeMap<String, Vec<IndexedBuildEvidence>>,
    run_references: &BTreeMap<String, Vec<String>>,
) -> Result<Vec<CandidateWithReference>> {
    let prefix = format!("{}-{}-linux-amd64-", tool.id, tool.version);
    let mut candidates = Vec::new();
    for (image_path, identity) in images.iter().filter(|(path, _)| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(&prefix))
    }) {
        let filename = image_path
            .file_name()
            .and_then(|name| name.to_str())
            .context("container image filename is not UTF-8")?;
        let immutable_name = filename == format!("{}{:.12}.sif", prefix, identity.sha256);
        let matching = evidence_by_digest
            .get(&identity.sha256)
            .into_iter()
            .flatten()
            .filter(|evidence| evidence_matches_image(evidence, tool, filename, identity))
            .collect::<Vec<_>>();
        let admitted = immutable_name
            && matching.iter().any(|evidence| {
                recipe_at_evidence_commit(request, tool, evidence).is_ok_and(|built_recipe| {
                    container_build_evidence_matches_recipe(&evidence.evidence, &built_recipe)
                })
            });
        let current_recipe = admitted
            && matching
                .iter()
                .any(|evidence| evidence_matches_recipe_content(&evidence.evidence, recipe));
        let is_referenced = run_references.contains_key(&identity.sha256);
        candidates.push((
            ContainerImageCandidate {
                admitted,
                current_recipe,
                disposition: ImageCandidateDisposition::HistoricalRunDependency,
                path: portable_study_path(image_path, &request.study_root)?,
                sha256: identity.sha256.clone(),
                build_evidence: matching.into_iter().map(|item| item.evidence.clone()).collect(),
            },
            is_referenced,
        ));
    }
    Ok(candidates)
}

fn select_candidates(
    tool_id: &str,
    support_disposition: &ContainerSupportDisposition,
    mut candidates: Vec<CandidateWithReference>,
) -> Result<(ContainerImageState, Option<String>, Vec<ContainerImageCandidate>)> {
    let current = candidates.iter().filter(|(candidate, _)| candidate.current_recipe).count();
    if current > 1 {
        bail!("tool {tool_id} has {current} admitted images for the current recipe");
    }
    let admitted = candidates.iter().filter(|(candidate, _)| candidate.admitted).count();
    if current == 0 && admitted > 1 {
        bail!("tool {tool_id} has {admitted} admitted historical images and no unique selection");
    }
    let selected = candidates
        .iter()
        .find(|(candidate, _)| candidate.current_recipe)
        .or_else(|| candidates.iter().find(|(candidate, _)| candidate.admitted))
        .map(|(candidate, _)| candidate.sha256.clone());
    if matches!(support_disposition, ContainerSupportDisposition::Retained) && selected.is_none() {
        bail!("retained tool {tool_id} has no admitted immutable image");
    }
    for (candidate, _) in &mut candidates {
        if Some(candidate.sha256.as_str()) == selected.as_deref() {
            candidate.disposition = ImageCandidateDisposition::Canonical;
        }
    }
    let retained = candidates
        .into_iter()
        .filter(|(candidate, referenced)| {
            Some(candidate.sha256.as_str()) == selected.as_deref() || *referenced
        })
        .map(|(candidate, _)| candidate)
        .collect::<Vec<_>>();
    let state = match selected.as_deref() {
        Some(digest)
            if retained
                .iter()
                .any(|candidate| candidate.sha256 == digest && candidate.current_recipe) =>
        {
            ContainerImageState::Current
        }
        Some(_) => ContainerImageState::Historical,
        None => ContainerImageState::Missing,
    };
    Ok((state, selected, retained))
}

fn verify_historical_references(
    candidates: &[ContainerImageCandidate],
    run_references: &BTreeMap<String, Vec<String>>,
) -> Result<()> {
    let referenced = referenced_digests(run_references);
    for candidate in candidates {
        if matches!(candidate.disposition, ImageCandidateDisposition::HistoricalRunDependency)
            && !referenced.contains(candidate.sha256.as_str())
        {
            bail!("unselected image {} has no immutable run reference", candidate.sha256);
        }
    }
    Ok(())
}

fn evidence_matches_image(
    evidence: &IndexedBuildEvidence,
    tool: &crate::evidence::ContainerCatalogTool,
    filename: &str,
    image: &RawImageIdentity,
) -> bool {
    evidence.tool == tool.id
        && evidence.version == tool.version
        && evidence.definition_path.as_deref().is_none_or(|path| path == tool.apptainer_definition)
        && evidence.image_name.as_deref().is_none_or(|name| name == filename)
        && evidence.image_size_bytes == image.size_bytes
        && matches!(
            evidence.evidence.status,
            BuildEvidenceStatus::Built | BuildEvidenceStatus::Passed
        )
}

fn evidence_matches_recipe_content(
    evidence: &crate::evidence::ContainerBuildEvidence,
    recipe: &crate::evidence::ApptainerRecipeIdentity,
) -> bool {
    matches!(evidence.status, BuildEvidenceStatus::Built | BuildEvidenceStatus::Passed)
        && evidence.definition_sha256 == recipe.definition_sha256
        && evidence.build_context.as_deref().unwrap_or_default() == recipe.build_context
}

fn recipe_at_evidence_commit(
    request: &ImageInventoryDerivationRequest,
    tool: &crate::evidence::ContainerCatalogTool,
    evidence: &IndexedBuildEvidence,
) -> Result<crate::evidence::ApptainerRecipeIdentity> {
    apptainer_recipe_identity_at_commit(
        &request.repository_root,
        &evidence.evidence.repository_commit,
        &tool.apptainer_definition,
    )
}

fn support_disposition(value: &str) -> Result<ContainerSupportDisposition> {
    match value {
        "retained" => Ok(ContainerSupportDisposition::Retained),
        "unsupported" => Ok(ContainerSupportDisposition::Unsupported),
        "retired" => Ok(ContainerSupportDisposition::Retired),
        "excluded" => Ok(ContainerSupportDisposition::Excluded),
        _ => bail!("unsupported catalog support disposition: {value}"),
    }
}
