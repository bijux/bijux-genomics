//! Pure qualification source identity, scope and selection admission.
use super::model::{
    QualificationSourceSelectionV1, QualificationSpecificationSourceV1,
    QUALIFICATION_SOURCE_SELECTION_SCHEMA_V1, QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1,
};
use anyhow::{bail, Result};
use std::path::{Component, Path};

pub(crate) fn validate_selection(selection: &QualificationSourceSelectionV1) -> Result<()> {
    if selection.schema_version != QUALIFICATION_SOURCE_SELECTION_SCHEMA_V1 {
        bail!("unsupported qualification source selection schema");
    }
    for (value, label) in [
        (&selection.tool_id, "tool"),
        (&selection.campaign, "campaign"),
        (&selection.corpus, "corpus"),
        (&selection.corpus_role, "corpus role"),
        (&selection.sample, "sample"),
    ] {
        if value.trim().is_empty() || value != value.trim() {
            bail!("qualification source selection {label} is empty or malformed");
        }
    }
    if selection.inputs.is_empty() {
        bail!("qualification source selection has no governed inputs");
    }
    for input in &selection.inputs {
        if !is_lower_snake_identifier(&input.id) {
            bail!("qualification source selection input ID must use lowercase snake case");
        }
    }
    if selection.negative_arguments.is_empty()
        || selection.negative_arguments.iter().any(String::is_empty)
    {
        bail!("qualification source selection requires a nonempty negative invocation");
    }
    Ok(())
}

fn is_lower_snake_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

pub(crate) fn validate_specification_header(
    source: &QualificationSpecificationSourceV1,
) -> Result<()> {
    if source.schema_version != QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1 {
        bail!("unsupported qualification specification source schema");
    }
    if !source.governed_study_root.is_absolute()
        || !source.repository_path.is_absolute()
        || !source.result_root.is_absolute()
    {
        bail!("qualification STUDY, repository, and result roots must be absolute");
    }
    require_path_under(
        &source.result_root,
        &source.governed_study_root,
        "qualification result root",
    )?;
    if source.repository_commit.len() != 40
        || source
            .repository_commit
            .bytes()
            .any(|byte| !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
    {
        bail!("qualification repository commit must be lowercase hexadecimal");
    }
    for (digest, label) in [
        (&source.catalog_sha256, "catalog"),
        (&source.registry_sha256, "qualification registry"),
        (&source.inventory_sha256, "image inventory"),
    ] {
        require_sha256(digest, label)?;
    }
    for (value, label) in [
        (&source.campaign, "campaign"),
        (&source.corpus, "corpus"),
        (&source.corpus_role, "corpus role"),
        (&source.sample, "sample"),
        (&source.operator, "operator"),
    ] {
        if value.trim().is_empty() {
            bail!("qualification {label} must not be empty");
        }
    }
    Ok(())
}

pub(crate) fn require_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || value.bytes().any(|byte| !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
    {
        bail!("{label} identity must be lowercase SHA-256");
    }
    Ok(())
}

pub(crate) fn require_safe_relative(path: &Path, label: &str) -> Result<()> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("{label} must be a safe relative path");
    }
    Ok(())
}

pub(crate) fn require_safe_component(value: &str, label: &str) -> Result<()> {
    let mut components = Path::new(value).components();
    if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
        bail!("{label} must be a safe path component");
    }
    Ok(())
}

pub(crate) fn require_path_under(path: &Path, root: &Path, label: &str) -> Result<()> {
    if !path.starts_with(root)
        || path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        bail!("{label} must remain under the governed STUDY root");
    }
    Ok(())
}
