use std::collections::BTreeSet;
use std::path::{Component, Path};

use anyhow::{bail, Result};

use super::inventory::{ImageInventoryScan, RAW_INVENTORY_SCHEMA_V1};
use super::model::{
    ContainerImageInventoryPublication, ContainerImageState, ContainerSupportDisposition,
    ImageCandidateDisposition, RuntimeVerificationPublication,
    CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V1,
    CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V1,
};

/// Validate complete scan identities before publication or replay.
///
/// # Errors
/// Returns an error when schema, source, image, path, or run-reference identity is malformed.
pub fn validate_image_inventory_scan(scan: &ImageInventoryScan) -> Result<()> {
    if scan.schema_version != RAW_INVENTORY_SCHEMA_V1 {
        bail!("unsupported image-inventory scan schema: {}", scan.schema_version);
    }
    validate_git_commit(&scan.repository_commit, "inventory scan repository")?;
    validate_sha256(&scan.catalog_sha256, "inventory scan catalog")?;
    if scan.images.is_empty() || scan.build_evidence.is_empty() {
        bail!("image-inventory scan requires images and build-evidence sources");
    }
    let mut image_paths = BTreeSet::new();
    let mut image_digests = BTreeSet::new();
    for image in &scan.images {
        validate_portable_path(&image.path, "inventory scan image")?;
        validate_sha256(&image.sha256, "inventory scan image")?;
        if image.size_bytes == 0
            || !image_paths.insert(image.path.as_str())
            || !image_digests.insert(image.sha256.as_str())
        {
            bail!("image-inventory scan contains empty or duplicate image identity");
        }
    }
    let mut evidence_paths = BTreeSet::new();
    for evidence in &scan.build_evidence {
        validate_portable_path(&evidence.path, "inventory build evidence")?;
        validate_sha256(&evidence.sha256, "inventory build evidence")?;
        if !evidence_paths.insert(evidence.path.as_str()) {
            bail!("image-inventory scan duplicates a build-evidence source");
        }
    }
    for (digest, paths) in &scan.run_references {
        validate_sha256(digest, "inventory run image")?;
        if paths.is_empty() {
            bail!("image-inventory scan has an empty run-reference set");
        }
        let mut unique = BTreeSet::new();
        for path in paths {
            validate_portable_path(path, "inventory run evidence")?;
            if !unique.insert(path.as_str()) {
                bail!("image-inventory scan duplicates a run-evidence reference");
            }
        }
    }
    Ok(())
}

/// Validate exact identity closure between a complete scan and its admitted publication.
///
/// # Errors
/// Returns an error when either document is invalid or their schema, commit, catalog, or serialized
/// scan identities differ.
pub fn validate_image_inventory_evidence(
    scan: &ImageInventoryScan,
    publication: &ContainerImageInventoryPublication,
) -> Result<()> {
    validate_image_inventory_scan(scan)?;
    validate_image_inventory_publication(publication)?;
    let scan_bytes =
        serde_json::to_vec_pretty(scan).map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if publication.publication.raw_schema_version != scan.schema_version
        || publication.publication.raw_sha256 != bijux_dna_infra::hash_bytes_sha256(&scan_bytes)
        || publication.repository_commit != scan.repository_commit
        || publication.catalog_sha256 != scan.catalog_sha256
    {
        bail!("image-inventory publication does not identify the exact admitted scan");
    }
    Ok(())
}

/// Validate immutable identities and selected-image closure in an inventory publication.
///
/// # Errors
/// Returns an error when schema identity, evidence identity, or tool closure is invalid.
pub fn validate_image_inventory_publication(
    publication: &ContainerImageInventoryPublication,
) -> Result<()> {
    if publication.schema_version != CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V1 {
        bail!("unsupported image-inventory publication schema: {}", publication.schema_version);
    }
    validate_sha256(&publication.catalog_sha256, "inventory catalog")?;
    validate_git_commit(&publication.repository_commit, "inventory repository")?;
    validate_sha256(&publication.publication.raw_sha256, "raw inventory")?;
    if publication.publication.raw_schema_version.trim().is_empty() {
        bail!("raw inventory schema version is empty");
    }

    let mut tool_ids = BTreeSet::new();
    for tool in &publication.tools {
        if !tool_ids.insert(tool.tool.as_str()) {
            bail!("image inventory duplicates tool {}", tool.tool);
        }
        if tool.tool.trim().is_empty() || tool.version.trim().is_empty() {
            bail!("image inventory contains an empty tool identity");
        }
        let mut candidate_digests = BTreeSet::new();
        let mut canonical_candidates = 0;
        for candidate in &tool.candidates {
            validate_sha256(&candidate.sha256, "image candidate")?;
            if !candidate_digests.insert(candidate.sha256.as_str()) {
                bail!(
                    "image inventory duplicates candidate {} for {}",
                    candidate.sha256,
                    tool.tool
                );
            }
            if candidate.path.trim().is_empty() {
                bail!("image candidate path is empty for {}", tool.tool);
            }
            if candidate.current_recipe && !candidate.admitted {
                bail!("current recipe candidate is not admitted for {}", tool.tool);
            }
            if matches!(candidate.disposition, ImageCandidateDisposition::Canonical) {
                canonical_candidates += 1;
            }
            for evidence in &candidate.build_evidence {
                validate_sha256(&evidence.definition_sha256, "recipe definition")?;
                validate_git_commit(&evidence.repository_commit, "build evidence repository")?;
                for context in evidence.build_context.iter().flatten() {
                    if context.path.trim().is_empty() {
                        bail!("build-context path is empty for {}", tool.tool);
                    }
                    validate_git_commit(&context.git_object, "build-context object")?;
                }
            }
        }

        match (&tool.support_disposition, &tool.selected_image_sha256) {
            (ContainerSupportDisposition::Retained, Some(selected)) => {
                validate_sha256(selected, "selected image")?;
                let selected_candidate = tool
                    .candidates
                    .iter()
                    .find(|candidate| candidate.admitted && candidate.sha256 == *selected);
                let Some(selected_candidate) = selected_candidate else {
                    bail!("selected image for {} is not an admitted candidate", tool.tool);
                };
                if !matches!(selected_candidate.disposition, ImageCandidateDisposition::Canonical)
                    || canonical_candidates != 1
                {
                    bail!("selected image for {} is not the unique canonical candidate", tool.tool);
                }
                match tool.state {
                    ContainerImageState::Current if !selected_candidate.current_recipe => {
                        bail!("current image state lacks a current recipe for {}", tool.tool);
                    }
                    ContainerImageState::Historical if selected_candidate.current_recipe => {
                        bail!("historical image state selects a current recipe for {}", tool.tool);
                    }
                    ContainerImageState::Current | ContainerImageState::Historical => {}
                    _ => bail!("retained tool {} has an invalid image state", tool.tool),
                }
            }
            (ContainerSupportDisposition::Retained, None) => {
                bail!("retained tool {} has no selected image", tool.tool);
            }
            (_, Some(selected)) => validate_sha256(selected, "selected image")?,
            (_, None) => {}
        }
        if !matches!(tool.support_disposition, ContainerSupportDisposition::Retained)
            && (tool.selected_image_sha256.is_some()
                || canonical_candidates != 0
                || !matches!(tool.state, ContainerImageState::Unsupported))
        {
            bail!("non-retained tool {} exposes a selected image lifecycle", tool.tool);
        }
    }
    Ok(())
}

/// Validate immutable source and per-tool image identities in a runtime publication.
///
/// # Errors
/// Returns an error when schema identity or evidence identity is invalid.
pub fn validate_runtime_verification_publication(
    publication: &RuntimeVerificationPublication,
) -> Result<()> {
    if publication.schema_version != CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V1 {
        bail!(
            "unsupported runtime-verification publication schema: {}",
            publication.schema_version
        );
    }
    validate_sha256(&publication.catalog.sha256, "runtime catalog")?;
    validate_sha256(&publication.inventory.sha256, "runtime inventory")?;
    validate_git_commit(&publication.repository_commit, "runtime repository")?;
    if publication.catalog.path.trim().is_empty() || publication.inventory.path.trim().is_empty() {
        bail!("runtime evidence source path is empty");
    }
    for (tool_id, evidence) in &publication.tools {
        if tool_id.trim().is_empty() || evidence.version.trim().is_empty() {
            bail!("runtime publication contains an empty tool identity");
        }
        validate_sha256(&evidence.image_sha256, "runtime image")?;
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        || value.bytes().all(|byte| byte == b'0')
    {
        bail!("{label} identity is not a SHA-256 digest: {value}");
    }
    Ok(())
}

fn validate_git_commit(value: &str, label: &str) -> Result<()> {
    if value.len() != 40
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        || value.bytes().all(|byte| byte == b'0')
    {
        bail!("{label} identity is not a full Git object: {value}");
    }
    Ok(())
}

fn validate_portable_path(value: &str, label: &str) -> Result<()> {
    let relative = value.strip_prefix("study://").or_else(|| value.strip_prefix("repository://"));
    let Some(relative) = relative else {
        bail!("{label} path is not governed and portable: {value}");
    };
    if relative.is_empty()
        || Path::new(relative).is_absolute()
        || Path::new(relative).components().any(|component| {
            matches!(component, Component::CurDir | Component::ParentDir | Component::RootDir)
        })
    {
        bail!("{label} path escapes its governed root: {value}");
    }
    Ok(())
}
