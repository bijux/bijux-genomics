use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::evidence::{
    apptainer_recipe_identity_at_commit, container_build_evidence_matches_recipe,
    load_container_catalog, repository_commit_is_ancestor, repository_head_commit,
    BuildEvidenceStatus, ContainerArtifactDelivery, ContainerDetachedImage,
    ContainerImageCandidate, ContainerImageInventoryPublication, ContainerImageInventorySummary,
    ContainerImageInventoryTool, ContainerImageState, ContainerStagedImage,
    DetachedImageDisposition, ImageCandidateDisposition, ImageInventoryPublicationIdentity,
    CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V3,
};

use super::source::{
    evidence_root_identities, load_build_evidence, load_run_references, portable_evidence_path,
    scan_images, scan_staged_images, validate_evidence_roots, ImageInventoryDerivationRequest,
    ImageInventoryScan, ImageInventoryScannedImage, IndexedBuildEvidence, RAW_INVENTORY_SCHEMA_V2,
};

struct CandidateSelection {
    candidate: ContainerImageCandidate,
    referenced: bool,
    admitted_commits: BTreeSet<String>,
    current_recipe_commits: BTreeSet<String>,
}

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
    validate_evidence_roots(request)?;
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
    let scanned_digests =
        images.iter().map(|(_, image)| image.sha256.as_str()).collect::<BTreeSet<_>>();
    let build_evidence = load_build_evidence(request, &scanned_digests)?;
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
        schema_version: RAW_INVENTORY_SCHEMA_V2.to_string(),
        repository_commit: repository_commit.clone(),
        catalog_sha256: catalog_sha256.clone(),
        evidence_roots: evidence_root_identities(request)?,
        images: images.iter().map(|(_, identity)| identity.clone()).collect(),
        staged_images,
        build_evidence: build_evidence.identities,
        run_references,
    };
    let raw_bytes =
        serde_json::to_vec_pretty(&scan).context("serialize raw image inventory scan")?;
    crate::evidence::validate_image_inventory_scan(&scan)?;
    let publication = ContainerImageInventoryPublication {
        schema_version: CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V3.to_string(),
        catalog_sha256,
        repository_commit,
        publication: ImageInventoryPublicationIdentity {
            raw_schema_version: RAW_INVENTORY_SCHEMA_V2.to_string(),
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

/// Derive and immutably publish both the complete scan and admitted inventory.
///
/// # Errors
/// Returns an error when output identities overlap or already exist, derivation fails, or either
/// publication cannot be created atomically.
pub fn publish_image_inventory_evidence(
    request: &ImageInventoryDerivationRequest,
    scan_output: &Path,
    publication_output: &Path,
) -> Result<DerivedImageInventory> {
    if scan_output == publication_output {
        bail!("image-inventory scan and publication outputs must be distinct");
    }
    if scan_output.try_exists()? || publication_output.try_exists()? {
        bail!("refusing to replace existing image-inventory evidence output");
    }
    let evidence = derive_image_inventory_evidence(request)?;
    bijux_dna_infra::atomic_create_json(scan_output, &evidence.scan).with_context(|| {
        format!("publish immutable image-inventory scan {}", scan_output.display())
    })?;
    bijux_dna_infra::atomic_create_json(publication_output, &evidence.publication).with_context(
        || {
            format!(
                "publish immutable image-inventory publication {}",
                publication_output.display()
            )
        },
    )?;
    Ok(evidence)
}

fn derive_tool(
    request: &ImageInventoryDerivationRequest,
    repository_commit: &str,
    tool: &crate::evidence::ContainerCatalogTool,
    images: &[(std::path::PathBuf, ImageInventoryScannedImage)],
    evidence_by_digest: &BTreeMap<String, Vec<IndexedBuildEvidence>>,
    run_references: &BTreeMap<String, Vec<String>>,
) -> Result<ContainerImageInventoryTool> {
    if tool.artifact_delivery != ContainerArtifactDelivery::RedistributableImage
        || !tool.apptainer_supported
    {
        return Ok(ContainerImageInventoryTool {
            tool: tool.id.clone(),
            version: tool.version.clone(),
            product_support: tool.product_support,
            artifact_delivery: tool.artifact_delivery,
            redistribution_permitted: tool.redistribution_permitted,
            apptainer_supported: tool.apptainer_supported,
            state: ContainerImageState::NotApplicable,
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
    let (state, selected, retained) = select_candidates(&request.repository_root, candidates)?;
    verify_historical_references(&retained)?;
    Ok(ContainerImageInventoryTool {
        tool: tool.id.clone(),
        version: tool.version.clone(),
        product_support: tool.product_support,
        artifact_delivery: tool.artifact_delivery,
        redistribution_permitted: tool.redistribution_permitted,
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
) -> Result<Vec<CandidateSelection>> {
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
        let admitted_evidence = if immutable_name {
            matching
                .iter()
                .filter(|evidence| {
                    recipe_at_evidence_commit(request, tool, evidence).is_ok_and(|built_recipe| {
                        container_build_evidence_matches_recipe(&evidence.evidence, &built_recipe)
                    })
                })
                .copied()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let admitted_commits = admitted_evidence
            .iter()
            .map(|evidence| evidence.evidence.repository_commit.clone())
            .collect::<BTreeSet<_>>();
        let current_recipe_commits = admitted_evidence
            .iter()
            .filter(|evidence| evidence_matches_recipe_content(&evidence.evidence, recipe))
            .map(|evidence| evidence.evidence.repository_commit.clone())
            .collect::<BTreeSet<_>>();
        let admitted = !admitted_commits.is_empty();
        let current_recipe = !current_recipe_commits.is_empty();
        let is_referenced = run_references.contains_key(&identity.sha256);
        let references = run_references.get(&identity.sha256).cloned().unwrap_or_default();
        candidates.push(CandidateSelection {
            candidate: ContainerImageCandidate {
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
            referenced: is_referenced,
            admitted_commits,
            current_recipe_commits,
        });
    }
    Ok(candidates)
}

fn select_candidates(
    repository_root: &std::path::Path,
    mut candidates: Vec<CandidateSelection>,
) -> Result<(ContainerImageState, Option<String>, Vec<ContainerImageCandidate>)> {
    let current = candidates
        .iter()
        .filter(|candidate| candidate.candidate.current_recipe)
        .collect::<Vec<_>>();
    let admitted =
        candidates.iter().filter(|candidate| candidate.candidate.admitted).collect::<Vec<_>>();
    let selection_pool = if current.is_empty() { &admitted } else { &current };
    let unreferenced = selection_pool
        .iter()
        .copied()
        .filter(|candidate| !candidate.referenced)
        .collect::<Vec<_>>();
    let descendant =
        unique_descendant_candidate(repository_root, selection_pool, !current.is_empty())?;
    let selected_candidate = if selection_pool.len() == 1 {
        selection_pool.first().copied()
    } else if descendant.is_some() {
        descendant
    } else if unreferenced.len() == 1 {
        unreferenced.first().copied()
    } else {
        None
    };
    let selection_pool_is_empty = selection_pool.is_empty();
    let selected = selected_candidate.map(|candidate| candidate.candidate.sha256.clone());
    for selection in &mut candidates {
        let candidate = &mut selection.candidate;
        if Some(candidate.sha256.as_str()) == selected.as_deref() {
            candidate.disposition = ImageCandidateDisposition::Canonical;
        } else if candidate.real_run_references == 0 {
            candidate.disposition = ImageCandidateDisposition::UnreferencedObsolete;
        }
    }
    let retained = candidates.into_iter().map(|selection| selection.candidate).collect::<Vec<_>>();
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
    Ok((state, selected, retained))
}

fn unique_descendant_candidate<'a>(
    repository_root: &std::path::Path,
    candidates: &[&'a CandidateSelection],
    current_recipe_only: bool,
) -> Result<Option<&'a CandidateSelection>> {
    if candidates.len() < 2 {
        return Ok(None);
    }
    let commits = candidates
        .iter()
        .map(|candidate| {
            let commits = if current_recipe_only {
                &candidate.current_recipe_commits
            } else {
                &candidate.admitted_commits
            };
            commits.iter().map(String::as_str).collect::<BTreeSet<_>>()
        })
        .collect::<Vec<_>>();
    let mut descendants = Vec::new();
    for (candidate_index, candidate_commits) in commits.iter().enumerate() {
        let mut candidate_is_descendant = false;
        for candidate_commit in candidate_commits {
            let mut descends_from_all = true;
            for competing_commit in commits.iter().flatten() {
                if !repository_commit_is_ancestor(
                    repository_root,
                    competing_commit,
                    candidate_commit,
                )? {
                    descends_from_all = false;
                    break;
                }
            }
            if descends_from_all {
                candidate_is_descendant = true;
                break;
            }
        }
        if candidate_is_descendant {
            descendants.push(candidates[candidate_index]);
        }
    }
    Ok(match descendants.as_slice() {
        [candidate] => Some(*candidate),
        _ => None,
    })
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
        *states.entry(image_state_name(tool.state).to_string()).or_insert(0) += 1;
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

fn image_state_name(state: ContainerImageState) -> &'static str {
    match state {
        ContainerImageState::Current => "current",
        ContainerImageState::Historical => "historical",
        ContainerImageState::Ambiguous => "ambiguous",
        ContainerImageState::Missing => "missing",
        ContainerImageState::NotApplicable => "not_applicable",
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
