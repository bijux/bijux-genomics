use std::collections::BTreeSet;

use anyhow::{bail, Result};

use super::model::{
    ContainerImageInventoryPublication, ContainerSupportDisposition,
    RuntimeVerificationPublication, CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V1,
    CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V1,
};

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
                if !tool
                    .candidates
                    .iter()
                    .any(|candidate| candidate.admitted && candidate.sha256 == *selected)
                {
                    bail!("selected image for {} is not an admitted candidate", tool.tool);
                }
            }
            (ContainerSupportDisposition::Retained, None) => {
                bail!("retained tool {} has no selected image", tool.tool);
            }
            (_, Some(selected)) => validate_sha256(selected, "selected image")?,
            (_, None) => {}
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
