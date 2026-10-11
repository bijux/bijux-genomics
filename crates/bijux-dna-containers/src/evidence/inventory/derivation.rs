use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use anyhow::{bail, Context, Result};

use crate::evidence::{
    apptainer_recipe_identity_at_commit, container_build_evidence_matches_recipe,
    load_container_catalog, repository_head_commit, BuildEvidenceStatus, ContainerDetachedImage,
    ContainerImageCandidate, ContainerImageInventoryPublication, ContainerImageInventorySummary,
    ContainerImageInventoryTool, ContainerImageState, ContainerStagedImage,
    ContainerSupportDisposition, DetachedImageDisposition, ImageCandidateDisposition,
    ImageInventoryPublicationIdentity, CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V2,
};

use super::source::{
    load_build_evidence, load_run_references, portable_evidence_path, scan_images,
    scan_staged_images, ImageInventoryDerivationRequest, ImageInventoryScan,
    ImageInventoryScannedImage, IndexedBuildEvidence, RAW_INVENTORY_SCHEMA_V1,
};

type CandidateWithReference = (ContainerImageCandidate, bool);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedImageInventory {
    pub scan: ImageInventoryScan,
    pub publication: ContainerImageInventoryPublication,
}

/// Derive a portable inventory publication from immutable images, build evidence, and run locks.
///
/// # Errors
/// Returns an error when source evidence is malformed, ambiguous, outside governed roots, or does
/// not close over the exact catalog and committed Apptainer recipe identities.
pub fn derive_image_inventory_publication(
    request: &ImageInventoryDerivationRequest,
) -> Result<ContainerImageInventoryPublication> {
    Ok(derive_image_inventory_evidence(request)?.publication)
}

/// Derive both the complete raw scan and its portable admitted publication.
///
/// # Errors
/// Returns an error under the same fail-closed conditions as inventory publication derivation.
pub fn derive_image_inventory_evidence(
    request: &ImageInventoryDerivationRequest,
) -> Result<DerivedImageInventory> {
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
    let staged_images = scan_staged_images(request)?;
    let build_evidence = load_build_evidence(request)?;
    let scanned_digests =
        images.iter().map(|(_, image)| image.sha256.as_str()).collect::<BTreeSet<_>>();
    let run_references = load_run_references(request)?
        .into_iter()
        .filter(|(digest, _)| scanned_digests.contains(digest.as_str()))
        .collect::<BTreeMap<_, _>>();

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
    let (historical_images, unknown_images) =
        classify_detached_images(&catalog.tools, &images, &tools, &run_references)?;
    let staged_publication = staged_images
        .iter()
        .map(|image| ContainerStagedImage {
            path: image.path.clone(),
            sha256: image.sha256.clone(),
            size_bytes: image.size_bytes,
        })
        .collect::<Vec<_>>();
    let summary =
        inventory_summary(&tools, &historical_images, &unknown_images, &staged_publication);

    let scan = ImageInventoryScan {
        schema_version: RAW_INVENTORY_SCHEMA_V1.to_string(),
        repository_commit: repository_commit.clone(),
        catalog_sha256: catalog_sha256.clone(),
        images: images.iter().map(|(_, identity)| identity.clone()).collect(),
        staged_images,
        build_evidence: build_evidence.identities,
        run_references,
    };
    let raw_bytes =
        serde_json::to_vec_pretty(&scan).context("serialize raw image inventory scan")?;
    crate::evidence::validate_image_inventory_scan(&scan)?;
    let publication = ContainerImageInventoryPublication {
        schema_version: CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V2.to_string(),
        catalog_sha256,
        repository_commit,
        publication: ImageInventoryPublicationIdentity {
            raw_schema_version: RAW_INVENTORY_SCHEMA_V1.to_string(),
            raw_sha256: bijux_dna_infra::hash_bytes_sha256(&raw_bytes),
        },
        summary,
        tools,
        historical_images,
        unknown_images,
        staged_images: staged_publication,
    };
    crate::evidence::validate_image_inventory_evidence(&scan, &publication)?;
    Ok(DerivedImageInventory { scan, publication })
}

fn derive_tool(
    request: &ImageInventoryDerivationRequest,
    repository_commit: &str,
    tool: &crate::evidence::ContainerCatalogTool,
    images: &[(std::path::PathBuf, ImageInventoryScannedImage)],
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
    let (state, selected, retained) = select_candidates(candidates);
    verify_historical_references(&retained)?;
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
    images: &[(std::path::PathBuf, ImageInventoryScannedImage)],
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
        let references = run_references.get(&identity.sha256).cloned().unwrap_or_default();
        candidates.push((
            ContainerImageCandidate {
                admitted,
                current_recipe,
                disposition: ImageCandidateDisposition::HistoricalRunDependency,
                path: portable_evidence_path(image_path, request)?,
                sha256: identity.sha256.clone(),
                size_bytes: identity.size_bytes,
                real_run_references: references.len() as u64,
                run_references: references,
                build_evidence: matching.into_iter().map(|item| item.evidence.clone()).collect(),
            },
            is_referenced,
        ));
    }
    Ok(candidates)
}

fn select_candidates(
    mut candidates: Vec<CandidateWithReference>,
) -> (ContainerImageState, Option<String>, Vec<ContainerImageCandidate>) {
    let current =
        candidates.iter().filter(|(candidate, _)| candidate.current_recipe).collect::<Vec<_>>();
    let admitted =
        candidates.iter().filter(|(candidate, _)| candidate.admitted).collect::<Vec<_>>();
    let selection_pool = if current.is_empty() { &admitted } else { &current };
    let unreferenced =
        selection_pool.iter().copied().filter(|(_, referenced)| !referenced).collect::<Vec<_>>();
    let selected_candidate = if selection_pool.len() == 1 {
        selection_pool.first().copied()
    } else if unreferenced.len() == 1 {
        unreferenced.first().copied()
    } else {
        None
    };
    let selection_pool_is_empty = selection_pool.is_empty();
    let selected = selected_candidate.map(|(candidate, _)| candidate.sha256.clone());
    for (candidate, _) in &mut candidates {
        if Some(candidate.sha256.as_str()) == selected.as_deref() {
            candidate.disposition = ImageCandidateDisposition::Canonical;
        } else if candidate.real_run_references == 0 {
            candidate.disposition = ImageCandidateDisposition::UnreferencedObsolete;
        }
    }
    let retained = candidates.into_iter().map(|(candidate, _)| candidate).collect::<Vec<_>>();
    let state = match selected.as_deref() {
        Some(digest)
            if retained
                .iter()
                .any(|candidate| candidate.sha256 == digest && candidate.current_recipe) =>
        {
            ContainerImageState::Current
        }
        Some(_) => ContainerImageState::Historical,
        None if selection_pool_is_empty => ContainerImageState::Missing,
        None => ContainerImageState::Ambiguous,
    };
    (state, selected, retained)
}

fn verify_historical_references(candidates: &[ContainerImageCandidate]) -> Result<()> {
    for candidate in candidates {
        if matches!(candidate.disposition, ImageCandidateDisposition::HistoricalRunDependency)
            && candidate.run_references.is_empty()
        {
            bail!("unselected image {} has no immutable run reference", candidate.sha256);
        }
    }
    Ok(())
}

fn classify_detached_images(
    catalog: &[crate::evidence::ContainerCatalogTool],
    images: &[(std::path::PathBuf, ImageInventoryScannedImage)],
    tools: &[ContainerImageInventoryTool],
    run_references: &BTreeMap<String, Vec<String>>,
) -> Result<(Vec<ContainerDetachedImage>, Vec<ContainerDetachedImage>)> {
    let consumed = tools
        .iter()
        .flat_map(|tool| tool.candidates.iter().map(|candidate| candidate.sha256.as_str()))
        .collect::<BTreeSet<_>>();
    let mut catalog_ids = catalog.iter().map(|tool| tool.id.as_str()).collect::<Vec<_>>();
    catalog_ids.sort_by_key(|tool_id| std::cmp::Reverse(tool_id.len()));
    let mut historical = Vec::new();
    let mut unknown = Vec::new();
    for (path, image) in
        images.iter().filter(|(_, image)| !consumed.contains(image.sha256.as_str()))
    {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .context("detached container image filename is not UTF-8")?;
        let digest_suffix = format!("-linux-amd64-{:.12}.sif", image.sha256);
        let catalog_tool = catalog_ids
            .iter()
            .find(|tool_id| {
                name.starts_with(&format!("{tool_id}-")) && name.ends_with(&digest_suffix)
            })
            .map(|value| (*value).to_string());
        let references = run_references.get(&image.sha256).cloned().unwrap_or_default();
        let disposition = match (&catalog_tool, references.is_empty()) {
            (Some(_), false) => DetachedImageDisposition::HistoricalRunDependency,
            (Some(_), true) => DetachedImageDisposition::UnreferencedObsolete,
            (None, _) => DetachedImageDisposition::Unclassified,
        };
        let record = ContainerDetachedImage {
            catalog_tool,
            disposition,
            path: image.path.clone(),
            sha256: image.sha256.clone(),
            size_bytes: image.size_bytes,
            real_run_references: references.len() as u64,
            run_references: references,
        };
        if matches!(disposition, DetachedImageDisposition::HistoricalRunDependency) {
            historical.push(record);
        } else {
            unknown.push(record);
        }
    }
    Ok((historical, unknown))
}

fn inventory_summary(
    tools: &[ContainerImageInventoryTool],
    historical_images: &[ContainerDetachedImage],
    unknown_images: &[ContainerDetachedImage],
    staged_images: &[ContainerStagedImage],
) -> ContainerImageInventorySummary {
    let mut states = BTreeMap::new();
    for tool in tools {
        *states.entry(image_state_name(&tool.state).to_string()).or_insert(0) += 1;
    }
    let historical_candidates = tools
        .iter()
        .flat_map(|tool| &tool.candidates)
        .filter(|candidate| {
            matches!(candidate.disposition, ImageCandidateDisposition::HistoricalRunDependency)
        })
        .count();
    let obsolete_candidates = tools
        .iter()
        .flat_map(|tool| &tool.candidates)
        .filter(|candidate| {
            matches!(candidate.disposition, ImageCandidateDisposition::UnreferencedObsolete)
        })
        .count();
    let obsolete_detached = unknown_images
        .iter()
        .filter(|image| matches!(image.disposition, DetachedImageDisposition::UnreferencedObsolete))
        .count();
    ContainerImageInventorySummary {
        tool_count: tools.len() as u64,
        states,
        historical_dependency_count: (historical_candidates + historical_images.len()) as u64,
        unknown_image_count: unknown_images.len() as u64,
        staged_image_count: staged_images.len() as u64,
        unreferenced_obsolete_count: (obsolete_candidates + obsolete_detached) as u64,
    }
}

fn image_state_name(state: &ContainerImageState) -> &'static str {
    match state {
        ContainerImageState::Current => "current",
        ContainerImageState::Historical => "historical",
        ContainerImageState::Ambiguous => "ambiguous",
        ContainerImageState::Missing => "missing",
        ContainerImageState::Unsupported => "unsupported",
    }
}

fn evidence_matches_image(
    evidence: &IndexedBuildEvidence,
    tool: &crate::evidence::ContainerCatalogTool,
    filename: &str,
    image: &ImageInventoryScannedImage,
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
